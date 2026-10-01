package main

import (
	"context"
	"database/sql"
	"errors"
	"fmt"
	"math"
	"math/rand/v2"
	"os"
	"runtime"
	"slices"
	"strings"
	"syscall"
	"time"
)

// cmdLoad builds a store of o.n synthetic events.
func cmdLoad(ctx context.Context, o opts) error {
	db, err := open(o)
	if err != nil {
		return err
	}
	defer db.Close()
	db.SetMaxOpenConns(1)
	if err := createSchema(ctx, db); err != nil {
		return err
	}
	c := newCorpus(o.seed)
	var textBytes int64
	prev := make([]byte, 32)
	start := time.Now()
	lap := start
	const batch = 10_000
	for seq := int64(1); seq <= o.n; {
		tx, err := db.BeginTx(ctx, nil)
		if err != nil {
			return err
		}
		a, err := newAppender(ctx, tx)
		if err != nil {
			return err
		}
		for j := 0; j < batch && seq <= o.n; j++ {
			e := c.next()
			textBytes += int64(len(e.body))
			if prev, err = a.append(ctx, seq, e, prev); err != nil {
				return err
			}
			seq++
		}
		a.close()
		if err := tx.Commit(); err != nil {
			return err
		}
		if (seq-1)%500_000 == 0 {
			now := time.Now()
			fmt.Printf("%d events  %.0f ev/s (lap)  %s elapsed\n", seq-1,
				500_000/now.Sub(lap).Seconds(), now.Sub(start).Round(time.Second))
			lap = now
		}
	}
	el := time.Since(start)
	if err := setMeta(ctx, db, "events", o.n); err != nil {
		return err
	}
	if err := setMeta(ctx, db, "text_bytes", textBytes); err != nil {
		return err
	}
	if _, err := db.ExecContext(ctx, `PRAGMA wal_checkpoint(TRUNCATE)`); err != nil {
		return err
	}
	fmt.Printf("RESULT load driver=%s events=%d text_bytes=%d elapsed=%s rate=%.0f ev/s\n",
		o.driver, o.n, textBytes, el.Round(time.Millisecond), float64(o.n)/el.Seconds())

	return reportSize(ctx, db, o)
}

// reportSize prints the store's on-disk footprint against the text it
// holds (NFR-09: at most 1.5x).
func reportSize(ctx context.Context, db *sql.DB, o opts) error {
	tb, err := getMeta(ctx, db, "text_bytes")
	if err != nil {
		return err
	}
	var size int64
	for _, suf := range []string{"", "-wal", "-shm"} {
		if st, err := os.Stat(o.db + suf); err == nil {
			size += st.Size()
		}
	}
	tables, err := dbstat(ctx, db)
	fmt.Printf("RESULT size driver=%s file_bytes=%d text_bytes=%d ratio=%.3f\n", o.driver, size, tb, float64(size)/float64(tb))
	if err == nil {
		for _, t := range tables {
			fmt.Printf("RESULT table driver=%s name=%s bytes=%d\n", o.driver, t.name, t.bytes)
		}
	} else {
		fmt.Printf("RESULT table driver=%s dbstat=unavailable (%v)\n", o.driver, err)
	}

	return nil
}

type tableSize struct {
	name  string
	bytes int64
}

func dbstat(ctx context.Context, db *sql.DB) ([]tableSize, error) {
	rows, err := db.QueryContext(ctx, `SELECT name, SUM(pgsize) FROM dbstat GROUP BY name ORDER BY 2 DESC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var out []tableSize
	for rows.Next() {
		var t tableSize
		if err := rows.Scan(&t.name, &t.bytes); err != nil {
			return nil, err
		}
		out = append(out, t)
	}

	return out, rows.Err()
}

// searchSQL ranks every match by BM25; the rowid floor, when above
// zero, bounds the candidates to the most recent events, a constraint
// FTS5 pushes into its doclist scan.
const searchSQL = `SELECT e.seq, e.session, snippet(events_fts, 0, '[', ']', '…', 16)
	FROM events_fts JOIN events e ON e.seq = events_fts.rowid
	WHERE events_fts MATCH ? AND events_fts.rowid > ? ORDER BY rank LIMIT 20`

// searchQueries draws realistic queries: terms lifted from a stored
// event, anchored on one term in the requested band, so every query
// has at least one hit, like an agent searching for something it saw.
func searchQueries(ctx context.Context, db *sql.DB, c *corpus, r *rand.Rand, n int, floor, events int64) ([]string, []string, error) {
	rank := make(map[string]int, len(c.words))
	for i, w := range c.words {
		rank[strings.ToLower(w)] = i
	}
	if events <= floor {
		return nil, nil, fmt.Errorf("no events to draw queries from: window (%d, %d]", floor, events)
	}
	var qs, qb []string
	// Every query lifts its anchor from a stored event; a window whose
	// events hold no term of a band would otherwise draw forever.
	for tries := 0; len(qs) < n; tries++ {
		if tries >= 1000*(n+1) {
			return nil, nil, fmt.Errorf("found %d of %d queries in %d events", len(qs), n, tries)
		}
		b := bands[len(qs)%len(bands)]
		var body string
		if err := db.QueryRowContext(ctx, `SELECT body FROM events WHERE seq = ?`, floor+1+r.Int64N(events-floor)).Scan(&body); err != nil {
			return nil, nil, err
		}
		var anchor []string
		var other []string
		for _, w := range strings.Fields(strings.ReplaceAll(body, ".", " ")) {
			k, ok := rank[strings.ToLower(w)]
			switch {
			case !ok:
			case k >= b.lo && k < b.hi:
				anchor = append(anchor, w)
			case k >= bands[0].lo && k < b.lo:
				other = append(other, w)
			}
		}
		if len(anchor) == 0 {
			continue
		}
		terms := []string{`"` + anchor[r.IntN(len(anchor))] + `"`}
		for extra := r.IntN(3); extra > 0 && len(other) > 0; extra-- {
			terms = append(terms, `"`+other[r.IntN(len(other))]+`"`)
		}
		qs = append(qs, strings.Join(terms, " "))
		qb = append(qb, b.name)
	}

	return qs, qb, nil
}

// cmdSearch measures FTS5 BM25 search latency (NFR-03: p95 <= 200 ms).
func cmdSearch(ctx context.Context, o opts) error {
	// The bounded shape ranks the newest matches of the whole store; it
	// takes no window, and a run naming both would report one it ignored.
	if o.recent > 0 && o.window < defaultWindow {
		return errors.New("-recent and -window do not combine")
	}
	db, err := open(o, "query_only(1)")
	if err != nil {
		return err
	}
	defer db.Close()
	db.SetMaxOpenConns(1)
	events, err := getMeta(ctx, db, "events")
	if err != nil {
		return err
	}
	c := newCorpus(o.seed)
	r := rand.New(rand.NewPCG(o.seed+7, 11))
	floor := max(0, events-o.window)
	qs, qb, err := searchQueries(ctx, db, c, r, int(o.n)+30, floor, events)
	if err != nil {
		return err
	}
	lat := map[string][]time.Duration{}
	var hits int
	for i, q := range qs {
		t := time.Now()
		var n int
		if o.recent > 0 {
			n, err = runSearchRecent(ctx, db, q, o.recent)
		} else {
			n, err = runSearchFrom(ctx, db, q, floor)
		}
		d := time.Since(t)
		if err != nil {
			return fmt.Errorf("query %q: %w", q, err)
		}
		if i < 30 { // warm-up
			continue
		}
		hits += n
		lat[qb[i]] = append(lat[qb[i]], d)
		lat["all"] = append(lat["all"], d)
	}
	for _, name := range []string{"common", "mid", "rare", "all"} {
		fmt.Printf("RESULT search driver=%s window=%d recent=%d band=%s %s\n", o.driver, o.window, o.recent, name, stats(lat[name]))
	}
	fmt.Printf("RESULT search driver=%s mean_hits=%.1f\n", o.driver, float64(hits)/float64(len(lat["all"])))

	return nil
}

func runSearch(ctx context.Context, db *sql.DB, q string) (int, error) {
	return runSearchFrom(ctx, db, q, 0)
}

// recentSQL ranks by BM25 only the most recent k matches. FTS5 walks
// a doclist in rowid order natively, so ORDER BY rowid DESC LIMIT k
// stops after k rows and bm25() is computed for those alone; the
// outer query keeps the best 20.
const recentSQL = `SELECT r FROM (
		SELECT rowid AS r, bm25(events_fts) AS s FROM events_fts
		WHERE events_fts MATCH ? ORDER BY rowid DESC LIMIT ?)
	ORDER BY s LIMIT 20`

const snippetSQL = `SELECT e.seq, e.session, snippet(events_fts, 0, '[', ']', '…', 16)
	FROM events_fts JOIN events e ON e.seq = events_fts.rowid
	WHERE events_fts MATCH ? AND events_fts.rowid = ?`

// runSearchRecent is the bounded query shape: rank the most recent k
// matches, then fetch snippets for the 20 kept.
func runSearchRecent(ctx context.Context, db *sql.DB, q string, k int64) (int, error) {
	rows, err := db.QueryContext(ctx, recentSQL, q, k)
	if err != nil {
		return 0, err
	}
	var ids []int64
	for rows.Next() {
		var r int64
		if err := rows.Scan(&r); err != nil {
			rows.Close()
			return 0, err
		}
		ids = append(ids, r)
	}
	rows.Close()
	if err := rows.Err(); err != nil {
		return 0, err
	}
	for _, id := range ids {
		var seq int64
		var ses, snip string
		if err := db.QueryRowContext(ctx, snippetSQL, q, id).Scan(&seq, &ses, &snip); err != nil {
			return 0, err
		}
	}

	return len(ids), nil
}

func runSearchFrom(ctx context.Context, db *sql.DB, q string, floor int64) (int, error) {
	rows, err := db.QueryContext(ctx, searchSQL, q, floor)
	if err != nil {
		return 0, err
	}
	defer rows.Close()
	n := 0
	for rows.Next() {
		var seq int64
		var ses, snip string
		if err := rows.Scan(&seq, &ses, &snip); err != nil {
			return 0, err
		}
		n++
	}

	return n, rows.Err()
}

func stats(ds []time.Duration) string {
	if len(ds) == 0 {
		return "n=0"
	}
	s := slices.Clone(ds)
	slices.Sort(s)
	// Nearest rank: the p95 of 300 samples is the 285th, s[284].
	p := func(q float64) time.Duration {
		return s[max(0, int(math.Ceil(q*float64(len(s))))-1)]
	}
	var sum time.Duration
	for _, d := range s {
		sum += d
	}

	return fmt.Sprintf("n=%d mean=%s p50=%s p95=%s p99=%s max=%s", len(s),
		(sum / time.Duration(len(s))).Round(10*time.Microsecond), p(0.5).Round(10*time.Microsecond),
		p(0.95).Round(10*time.Microsecond), p(0.99).Round(10*time.Microsecond), s[len(s)-1].Round(10*time.Microsecond))
}

// cmdExpand measures fetching a 50-event window by seq (NFR-03:
// expand p95 <= 100 ms).
func cmdExpand(ctx context.Context, o opts) error {
	db, err := open(o, "query_only(1)")
	if err != nil {
		return err
	}
	defer db.Close()
	db.SetMaxOpenConns(1)
	events, err := getMeta(ctx, db, "events")
	if err != nil {
		return err
	}
	r := rand.New(rand.NewPCG(o.seed+9, 13))
	var lat []time.Duration
	for i := int64(0); i < o.n+30; i++ {
		lo := 1 + r.Int64N(events-50)
		t := time.Now()
		rows, err := db.QueryContext(ctx, `SELECT seq, session, kind, trust, tool, body, tokens, ts, hash
			FROM events WHERE seq BETWEEN ? AND ? ORDER BY seq`, lo, lo+49)
		if err != nil {
			return err
		}
		n := 0
		for rows.Next() {
			var seq, trust, tokens, ts int64
			var ses, kind, body string
			var tool sql.NullString
			var h []byte
			if err := rows.Scan(&seq, &ses, &kind, &trust, &tool, &body, &tokens, &ts, &h); err != nil {
				return err
			}
			n++
		}
		rows.Close()
		if n != 50 {
			return fmt.Errorf("expand got %d rows", n)
		}
		if i >= 30 {
			lat = append(lat, time.Since(t))
		}
	}
	fmt.Printf("RESULT expand driver=%s %s\n", o.driver, stats(lat))

	return nil
}

// cmdIngest measures append throughput on one core against the
// existing store (NFR-04: >= 5,000 events/s), o.batch events per
// transaction, the unit one hook invocation commits.
func cmdIngest(ctx context.Context, o opts) error {
	runtime.GOMAXPROCS(1)
	db, err := open(o)
	if err != nil {
		return err
	}
	defer db.Close()
	db.SetMaxOpenConns(1)
	c := newCorpusFrom(o.seed, o.seed+1000)
	evs := make([]event, o.n)
	for i := range evs {
		evs[i] = c.next()
	}
	seq, prev, err := lastHash(ctx, db)
	if err != nil {
		return err
	}
	var text int64
	start := time.Now()
	for i := 0; i < len(evs); {
		tx, err := db.BeginTx(ctx, nil)
		if err != nil {
			return err
		}
		a, err := newAppender(ctx, tx)
		if err != nil {
			return err
		}
		for j := 0; j < o.batch && i < len(evs); j++ {
			seq++
			evs[i].source, evs[i].line = 1_000_000, seq
			text += int64(len(evs[i].body))
			if prev, err = a.append(ctx, seq, evs[i], prev); err != nil {
				return err
			}
			i++
		}
		a.close()
		if err := tx.Commit(); err != nil {
			return err
		}
	}
	el := time.Since(start)
	fmt.Printf("RESULT ingest driver=%s events=%d batch=%d gomaxprocs=1 elapsed=%s rate=%.0f ev/s\n",
		o.driver, o.n, o.batch, el.Round(time.Millisecond), float64(o.n)/el.Seconds())

	return addMeta(ctx, db, o.n, text)
}

// addMeta adds appended events and their text to the store's counters,
// so later size ratios and query draws see the events ingest and hook
// appended.
func addMeta(ctx context.Context, db *sql.DB, events, text int64) error {
	n, err := getMeta(ctx, db, "events")
	if err != nil {
		return fmt.Errorf("read events: %w", err)
	}
	tb, err := getMeta(ctx, db, "text_bytes")
	if err != nil {
		return fmt.Errorf("read text_bytes: %w", err)
	}
	if err := setMeta(ctx, db, "events", n+events); err != nil {
		return fmt.Errorf("write events: %w", err)
	}
	if err := setMeta(ctx, db, "text_bytes", tb+text); err != nil {
		return fmt.Errorf("write text_bytes: %w", err)
	}

	return nil
}

// cmdHook runs what one hook process does — open the store, append
// one batch, run one search, close — and reports its peak RSS
// (NFR-09: <= 50 MiB) and wall time.
func cmdHook(ctx context.Context, o opts) error {
	start := time.Now()
	c := newCorpusFrom(o.seed, o.seed+2000)
	evs := make([]event, o.batch)
	for i := range evs {
		evs[i] = c.next()
	}
	var ru0 syscall.Rusage
	syscall.Getrusage(syscall.RUSAGE_SELF, &ru0)
	db, err := open(o)
	if err != nil {
		return err
	}
	db.SetMaxOpenConns(1)
	tx, err := db.BeginTx(ctx, nil)
	if err != nil {
		return err
	}
	seq, prev, err := lastHash(ctx, tx)
	if err != nil {
		return err
	}
	a, err := newAppender(ctx, tx)
	if err != nil {
		return err
	}
	var text int64
	for _, e := range evs {
		seq++
		e.source, e.line = 2_000_000, seq
		text += int64(len(e.body))
		if prev, err = a.append(ctx, seq, e, prev); err != nil {
			return err
		}
	}
	a.close()
	if err := tx.Commit(); err != nil {
		return err
	}
	if _, err := runSearch(ctx, db, `"`+c.words[1200]+`" "`+c.words[80]+`"`); err != nil {
		return err
	}
	if err := db.Close(); err != nil {
		return err
	}
	var ru syscall.Rusage
	syscall.Getrusage(syscall.RUSAGE_SELF, &ru)
	fmt.Printf("RESULT hook driver=%s batch=%d wall=%s maxrss_before_open=%.1fMiB maxrss=%.1fMiB\n",
		o.driver, o.batch, time.Since(start).Round(time.Millisecond), float64(ru0.Maxrss)/1024, float64(ru.Maxrss)/1024)

	// The counters update after the measurement, on a fresh
	// connection, so they cost the measured hook nothing.
	mdb, err := open(o)
	if err != nil {
		return err
	}
	defer mdb.Close()

	return addMeta(ctx, mdb, int64(len(evs)), text)
}

// cmdCancel checks that a context deadline stops a running query
// promptly and leaves the connection usable.
func cmdCancel(ctx context.Context, o opts) error {
	db, err := open(o)
	if err != nil {
		return err
	}
	defer db.Close()
	db.SetMaxOpenConns(1)
	conn, err := db.Conn(ctx)
	if err != nil {
		return err
	}
	defer conn.Close()
	queries := []struct{ name, sql string }{
		{"cte", `WITH RECURSIVE c(x) AS (SELECT 1 UNION ALL SELECT x+1 FROM c) SELECT count(*) FROM (SELECT x FROM c LIMIT 2000000000)`},
		{"fts_common", `SELECT count(*), sum(s) FROM (SELECT rowid AS s FROM events_fts WHERE events_fts MATCH '"` + newCorpus(o.seed).words[21] + `"' ORDER BY rank LIMIT 20)`},
		{"scan", `SELECT count(*) FROM events WHERE body LIKE '%zzqq%'`},
	}
	for _, q := range queries {
		for _, deadline := range []time.Duration{50 * time.Millisecond, 250 * time.Millisecond} {
			qctx, cancel := context.WithTimeout(ctx, deadline)
			t := time.Now()
			var a, b sql.NullFloat64
			var err error
			if q.name == "fts_common" {
				err = conn.QueryRowContext(qctx, q.sql).Scan(&a, &b)
			} else {
				err = conn.QueryRowContext(qctx, q.sql).Scan(&a)
			}
			el := time.Since(t)
			cancel()
			var after int64
			aerr := conn.QueryRowContext(ctx, `SELECT count(*) FROM meta`).Scan(&after)
			fmt.Printf("RESULT cancel driver=%s query=%s deadline=%s returned_after=%s overrun=%s err=%q is_deadline=%v conn_usable=%v\n",
				o.driver, q.name, deadline, el.Round(time.Millisecond), (el - deadline).Round(time.Millisecond),
				errString(err), errors.Is(err, context.DeadlineExceeded), aerr == nil)
		}
	}
	// A write interrupted mid-way must leave nothing behind. The
	// transaction itself runs on the parent context: database/sql rolls
	// back any transaction whose BeginTx context ends, which would pass
	// this check for every driver. Only the statement gets the deadline,
	// so the rollback observed is the driver's and SQLite's. A sentinel
	// row written before the long statement tells a rolled-back
	// transaction from one where only the interrupted statement failed:
	// statements are atomic, so counting rows alone cannot.
	if _, err := conn.ExecContext(ctx, `DELETE FROM meta WHERE k = 'sentinel'`); err != nil {
		return err
	}
	before, berr := count(ctx, conn)
	tx, err := conn.BeginTx(ctx, nil)
	if err != nil {
		return err
	}
	if _, err := tx.ExecContext(ctx, `INSERT INTO meta(k, v) VALUES('sentinel', 1)`); err != nil {
		tx.Rollback()
		return err
	}
	wctx, cancel := context.WithTimeout(ctx, 100*time.Millisecond)
	_, werr := tx.ExecContext(wctx, `INSERT INTO meta(k, v) WITH RECURSIVE c(x) AS (SELECT 1 UNION ALL SELECT x+1 FROM c LIMIT 50000000) SELECT 'junk' || x, x FROM c`)
	cancel()
	var cerr error
	if werr == nil {
		// The deadline never fired; never commit the junk rows.
		cerr = errors.Join(errors.New("statement finished before its deadline"), tx.Rollback())
	} else {
		// Try to commit: if the interrupt left the transaction open,
		// the sentinel lands and the check below fails.
		cerr = tx.Commit()
	}
	after, aerr := count(ctx, conn)
	var sentinels int64
	serr := conn.QueryRowContext(ctx, `SELECT count(*) FROM meta WHERE k = 'sentinel'`).Scan(&sentinels)
	// A count that failed proves nothing either way.
	fmt.Printf("RESULT cancel driver=%s query=write_tx exec_err=%q commit_err=%q count_err=%q rows_before=%d rows_after=%d sentinel_rows=%d rolled_back=%v\n",
		o.driver, errString(werr), errString(cerr), errString(errors.Join(berr, aerr, serr)), before, after, sentinels,
		werr != nil && berr == nil && aerr == nil && serr == nil && before == after && sentinels == 0)

	return nil
}

func count(ctx context.Context, conn *sql.Conn) (int64, error) {
	var n int64
	err := conn.QueryRowContext(ctx, `SELECT count(*) FROM meta`).Scan(&n)
	return n, err
}

func errString(err error) string {
	if err == nil {
		return ""
	}

	return err.Error()
}
