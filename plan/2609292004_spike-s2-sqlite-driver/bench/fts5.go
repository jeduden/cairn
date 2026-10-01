package main

import (
	"context"
	"database/sql"
	"fmt"
	"os"
	"strings"
	"time"
)

// cmdFTS5 probes the FTS5 features Cairn's recall relies on (ADR-05,
// RCL-01) on a small fresh store: BM25 ranking, snippet, phrase,
// prefix and NEAR queries, the unicode61 and trigram tokenizers, and
// rebuilding the projection from the record (I10).
func cmdFTS5(ctx context.Context, o opts) error {
	for _, suf := range []string{"", "-wal", "-shm"} {
		os.Remove(o.db + suf)
	}
	db, err := open(o.driver, o.db)
	if err != nil {
		return err
	}
	defer db.Close()
	db.SetMaxOpenConns(1)
	var version string
	db.QueryRowContext(ctx, `SELECT sqlite_version()`).Scan(&version)
	fmt.Printf("RESULT fts5 driver=%s sqlite_version=%s\n", o.driver, version)
	if err := createSchema(ctx, db); err != nil {
		return err
	}
	docs := []string{
		"the migration failed because the sqlite driver was missing FTS5",
		"we pinned the constraint: never touch the production database",
		"driver benchmark: modernc versus ncruces at ten million events",
		"Ünïcödé naïve café résumé text with diacritics",
		"the driver the driver the driver repeated term frequency",
	}
	tx, _ := db.BeginTx(ctx, nil)
	a, err := newAppender(ctx, tx)
	if err != nil {
		return err
	}
	prev := make([]byte, 32)
	for i, d := range docs {
		if prev, err = a.append(ctx, int64(i+1), event{session: "s", source: 1, line: int64(i), kind: "user", body: d}, prev); err != nil {
			return err
		}
	}
	a.close()
	if err := tx.Commit(); err != nil {
		return err
	}
	probes := []struct{ name, q string }{
		{"bm25_order", `driver`},
		{"phrase", `"sqlite driver"`},
		{"prefix", `bench*`},
		{"near", `NEAR(migration FTS5, 8)`},
		{"diacritics_folded", `cafe`},
		{"boolean_not", `driver NOT benchmark`},
	}
	for _, p := range probes {
		rows, err := db.QueryContext(ctx, `SELECT rowid, round(bm25(events_fts), 3), snippet(events_fts, 0, '[', ']', '…', 6)
			FROM events_fts WHERE events_fts MATCH ? ORDER BY rank`, p.q)
		if err != nil {
			fmt.Printf("RESULT fts5 driver=%s probe=%s ok=false err=%q\n", o.driver, p.name, err)
			continue
		}
		var got []string
		for rows.Next() {
			var id int64
			var score float64
			var snip string
			rows.Scan(&id, &score, &snip)
			got = append(got, fmt.Sprintf("%d(%.3f)%s", id, score, snip))
		}
		rows.Close()
		fmt.Printf("RESULT fts5 driver=%s probe=%s ok=true hits=%q\n", o.driver, p.name, strings.Join(got, " | "))
	}
	for _, tok := range []string{"unicode61 remove_diacritics 2", "trigram", "porter unicode61"} {
		_, err := db.ExecContext(ctx, `CREATE VIRTUAL TABLE t_`+strings.Fields(tok)[0]+` USING fts5(x, tokenize='`+tok+`')`)
		fmt.Printf("RESULT fts5 driver=%s tokenizer=%q ok=%v err=%q\n", o.driver, tok, err == nil, errString(err))
	}
	_, err = db.ExecContext(ctx, `INSERT INTO events_fts(events_fts) VALUES('delete-all')`)
	if err == nil {
		_, err = db.ExecContext(ctx, `INSERT INTO events_fts(events_fts) VALUES('rebuild')`)
	}
	var n int64
	db.QueryRowContext(ctx, `SELECT count(*) FROM events_fts WHERE events_fts MATCH 'driver'`).Scan(&n)
	fmt.Printf("RESULT fts5 driver=%s probe=rebuild_from_record ok=%v hits_after=%d err=%q\n", o.driver, err == nil && n == 3, n, errString(err))
	var opts []string
	rows, err := db.QueryContext(ctx, `PRAGMA compile_options`)
	if err == nil {
		for rows.Next() {
			var s string
			rows.Scan(&s)
			if strings.Contains(s, "FTS") || strings.Contains(s, "THREADSAFE") || strings.Contains(s, "DQS") || strings.Contains(s, "DBSTAT") {
				opts = append(opts, s)
			}
		}
		rows.Close()
	}
	fmt.Printf("RESULT fts5 driver=%s compile_options=%q\n", o.driver, strings.Join(opts, ","))
	_ = sql.ErrNoRows

	return nil
}

// cmdOptimize reports the FTS5 index's segment count, merges it into
// one segment with the 'optimize' command, and reports the time and
// the store's size afterwards. Bulk loading leaves many segments, and
// every query reads each one.
func cmdOptimize(ctx context.Context, o opts) error {
	db, err := open(o.driver, o.db)
	if err != nil {
		return err
	}
	defer db.Close()
	db.SetMaxOpenConns(1)
	segs := func() int64 {
		var n int64
		db.QueryRowContext(ctx, `SELECT count(DISTINCT segid) FROM events_fts_idx`).Scan(&n)
		return n
	}
	before := segs()
	t := time.Now()
	if _, err := db.ExecContext(ctx, `INSERT INTO events_fts(events_fts) VALUES('optimize')`); err != nil {
		return err
	}
	el := time.Since(t)
	if _, err := db.ExecContext(ctx, `PRAGMA wal_checkpoint(TRUNCATE)`); err != nil {
		return err
	}
	fmt.Printf("RESULT optimize driver=%s segments_before=%d segments_after=%d elapsed=%s\n", o.driver, before, segs(), el.Round(time.Second))

	return reportSize(ctx, db, o)
}
