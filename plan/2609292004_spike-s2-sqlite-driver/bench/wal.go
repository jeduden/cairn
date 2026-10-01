package main

import (
	"bytes"
	"context"
	"crypto/sha256"
	"fmt"
	"math/rand/v2"
	"os"
	"os/exec"
	"strconv"
	"strings"
	"sync"
	"sync/atomic"
	"time"
)

// cmdWAL checks WAL correctness under o.workers concurrent writer
// processes (NFR-05: >= 50), ADR-01's daemonless model: each writer is
// its own OS process opening the store directly. Readers search
// throughout. Afterwards seq must be gap-free, the hash chain intact,
// and both integrity checks clean.
func cmdWAL(ctx context.Context, o opts) error {
	for _, suf := range []string{"", "-wal", "-shm"} {
		os.Remove(o.db + suf)
	}
	db, err := open(o)
	if err != nil {
		return err
	}
	defer db.Close()
	if err := createSchema(ctx, db); err != nil {
		return err
	}
	if err := setMeta(ctx, db, "events", 0); err != nil {
		return err
	}

	stop := make(chan struct{})
	var reads, readErrs atomic.Int64
	var rwg sync.WaitGroup
	words := newCorpus(o.seed).words
	for r := range 4 {
		rwg.Add(1)
		go func() {
			defer rwg.Done()
			rdb, err := open(o, "query_only(1)")
			if err != nil {
				readErrs.Add(1)
				return
			}
			defer rdb.Close()
			rng := rand.New(rand.NewPCG(uint64(r), 3))
			for {
				select {
				case <-stop:
					return
				default:
				}
				if _, err := runSearch(ctx, rdb, `"`+words[20+rng.IntN(2000)]+`"`); err != nil {
					readErrs.Add(1)
				}
				reads.Add(1)
			}
		}()
	}

	start := time.Now()
	var wg sync.WaitGroup
	outs := make([]string, o.workers)
	fails := make([]error, o.workers)
	for w := range o.workers {
		wg.Add(1)
		go func() {
			defer wg.Done()
			// Workers must open the store the way this process does,
			// including the encrypting VFS when -vfs is set.
			cmd := exec.Command(os.Args[0], "walworker", "-driver", o.driver, "-db", o.db, "-vfs", o.vfs,
				"-n", strconv.FormatInt(o.n, 10), "-id", strconv.Itoa(w), "-seed", strconv.FormatUint(o.seed, 10))
			out, err := cmd.CombinedOutput()
			outs[w], fails[w] = string(out), err
		}()
	}
	wg.Wait()
	el := time.Since(start)
	close(stop)
	rwg.Wait()

	var failed, retries int
	var worst time.Duration
	for w := range o.workers {
		if fails[w] != nil {
			failed++
			fmt.Printf("worker %d failed: %v\n%s", w, fails[w], outs[w])
			continue
		}
		for _, f := range strings.Fields(outs[w]) {
			if v, ok := strings.CutPrefix(f, "retries="); ok {
				n, _ := strconv.Atoi(v)
				retries += n
			}
			if v, ok := strings.CutPrefix(f, "max="); ok {
				d, _ := time.ParseDuration(v)
				worst = max(worst, d)
			}
		}
	}
	fmt.Printf("RESULT wal driver=%s writers=%d tx_per_writer=%d elapsed=%s failed_writers=%d busy_retries=%d worst_commit=%s reads=%d read_errors=%d\n",
		o.driver, o.workers, o.n, el.Round(time.Millisecond), failed, retries, worst, reads.Load(), readErrs.Load())

	return verifyStore(ctx, o, int64(o.workers)*o.n)
}

// verifyStore checks the record after the concurrent run.
func verifyStore(ctx context.Context, o opts, wantTx int64) error {
	db, err := open(o)
	if err != nil {
		return err
	}
	defer db.Close()
	var n, lo, hi int64
	if err := db.QueryRowContext(ctx, `SELECT count(*), min(seq), max(seq) FROM events`).Scan(&n, &lo, &hi); err != nil {
		return err
	}
	var txs int64
	if err := db.QueryRowContext(ctx, `SELECT count(DISTINCT source * 1000000 + line / 3) FROM events`).Scan(&txs); err != nil {
		return err
	}

	rows, err := db.QueryContext(ctx, `SELECT seq, body, prev_hash, hash FROM events ORDER BY seq`)
	if err != nil {
		return err
	}
	defer rows.Close()
	prev := make([]byte, 32)
	chainOK := true
	var walked int64
	for rows.Next() {
		walked++
		var seq int64
		var body string
		var ph, h []byte
		if err := rows.Scan(&seq, &body, &ph, &h); err != nil {
			return err
		}
		s := sha256.New()
		s.Write(ph)
		s.Write([]byte(body))
		if !bytes.Equal(ph, prev) || !bytes.Equal(s.Sum(nil), h) {
			if chainOK {
				fmt.Printf("chain broken at seq %d\n", seq)
			}
			chainOK = false
		}
		prev = h
	}
	// A walk cut short by an error must not read as an intact chain.
	if err := rows.Err(); err != nil {
		return fmt.Errorf("walk chain: %w", err)
	}
	rows.Close()
	if walked != n {
		fmt.Printf("chain walk read %d of %d events\n", walked, n)
		chainOK = false
	}
	var integ string
	if err := db.QueryRowContext(ctx, `PRAGMA integrity_check`).Scan(&integ); err != nil {
		return err
	}
	_, ftsErr := db.ExecContext(ctx, `INSERT INTO events_fts(events_fts, rank) VALUES('integrity-check', 1)`)
	fmt.Printf("RESULT walverify driver=%s events=%d transactions=%d/%d seq_min=%d seq_max=%d gap_free=%v chain_ok=%v integrity=%s fts_integrity_err=%q\n",
		o.driver, n, txs, wantTx, lo, hi, lo == 1 && hi == n, chainOK, integ, errString(ftsErr))

	return nil
}

// cmdWALWorker is one writer process: o.n transactions of 1–3 events,
// each allocating seq and chaining onto the tail inside BEGIN
// IMMEDIATE.
func cmdWALWorker(ctx context.Context, o opts) error {
	c := newCorpus(o.seed + uint64(o.id)*7919)
	r := rand.New(rand.NewPCG(uint64(o.id), 5))
	var retries int
	var lat []time.Duration
	for i := int64(0); i < o.n; i++ {
		k := 1 + r.IntN(3)
		evs := make([]event, k)
		for j := range evs {
			evs[j] = c.next()
			evs[j].source = int64(1000 + o.id)
			evs[j].line = i*3 + int64(j)
		}
		for attempt := 0; ; attempt++ {
			t := time.Now()
			err := writeTx(ctx, o, evs)
			if err == nil {
				lat = append(lat, time.Since(t))
				break
			}
			if attempt >= 20 || !strings.Contains(strings.ToLower(err.Error()), "busy") && !strings.Contains(strings.ToLower(err.Error()), "locked") {
				return fmt.Errorf("worker %d tx %d: %w", o.id, i, err)
			}
			retries++
			time.Sleep(time.Duration(r.IntN(20)) * time.Millisecond)
		}
	}
	fmt.Printf("worker=%d retries=%d %s\n", o.id, retries, stats(lat))

	return nil
}

// writeTx opens its own connection per transaction, so every
// transaction pays the open a hook process pays.
func writeTx(ctx context.Context, o opts, evs []event) error {
	db, err := open(o)
	if err != nil {
		return err
	}
	defer db.Close()
	tx, err := db.BeginTx(ctx, nil)
	if err != nil {
		return err
	}
	defer tx.Rollback()
	seq, prev, err := lastHash(ctx, tx)
	if err != nil {
		return err
	}
	a, err := newAppender(ctx, tx)
	if err != nil {
		return err
	}
	defer a.close()
	for _, e := range evs {
		seq++
		if prev, err = a.append(ctx, seq, e, prev); err != nil {
			return err
		}
	}

	return tx.Commit()
}
