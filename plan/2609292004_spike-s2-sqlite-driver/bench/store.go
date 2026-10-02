package main

import (
	"context"
	"crypto/sha256"
	"database/sql"
	"fmt"
	"net/url"
	"strings"

	"github.com/ncruces/go-sqlite3/driver"
	"github.com/ncruces/go-sqlite3/ext/fts5"
	_ "github.com/ncruces/go-sqlite3/vfs/adiantum"
	_ "github.com/ncruces/go-sqlite3/vfs/xts"
	_ "modernc.org/sqlite"
)

// dsn builds the same URI for both drivers: WAL, NORMAL sync, a busy
// timeout, and BEGIN IMMEDIATE for every read-write transaction, the
// settings ADR-01 names. vfs names an encrypting VFS (ncruces only)
// for the OQ-04 probe, empty for plain files. extra carries further
// _pragma values.
func dsn(path, vfs string, extra ...string) string {
	q := url.Values{}
	if vfs != "" {
		// The key PRAGMA must run first, before the file is read.
		q.Set("vfs", vfs)
		q.Add("_pragma", "hexkey('"+testKey+"')")
	}
	q.Add("_pragma", "busy_timeout(20000)")
	q.Add("_pragma", "journal_mode(WAL)")
	q.Add("_pragma", "synchronous(NORMAL)")
	for _, p := range extra {
		q.Add("_pragma", p)
	}
	q.Set("_txlock", "immediate")

	return "file:" + uriPath.Replace(path) + "?" + q.Encode()
}

// uriPath escapes the characters a file: URI gives meaning to, so a
// path holding '?', '#' or '%' names the file it spells.
var uriPath = strings.NewReplacer("%", "%25", "?", "%3f", "#", "%23")

// testKey is the spike's fixed 256-bit key; key management is out of
// the probe's scope (OQ-04 records it).
const testKey = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"

// open opens o.db with o.driver. modernc compiles FTS5 in; ncruces
// ships it as a separate module, registered here on each connection
// through driver.Open's init hook rather than the process-global
// sqlite3.AutoExtension.
func open(o opts, extra ...string) (*sql.DB, error) {
	name := dsn(o.db, o.vfs, extra...)
	switch o.driver {
	case "modernc":
		return sql.Open("sqlite", name)
	case "ncruces":
		return driver.Open(name, fts5.Register)
	}

	return nil, fmt.Errorf("unknown driver %q", o.driver)
}

// schema approximates SRS §8.2's events record and its FTS5
// projection. events_fts is external-content, so the text is stored
// once, in events.
var schema = []string{
	`CREATE TABLE IF NOT EXISTS meta(k TEXT PRIMARY KEY, v INTEGER NOT NULL)`,
	`CREATE TABLE IF NOT EXISTS events(
		seq INTEGER PRIMARY KEY,
		session TEXT NOT NULL,
		source INTEGER NOT NULL,
		generation INTEGER NOT NULL,
		line INTEGER NOT NULL,
		kind TEXT NOT NULL,
		trust INTEGER NOT NULL,
		tool TEXT,
		body TEXT NOT NULL,
		payload_hash BLOB,
		payload_size INTEGER,
		tokens INTEGER NOT NULL,
		ts INTEGER NOT NULL,
		prev_hash BLOB NOT NULL,
		hash BLOB NOT NULL,
		UNIQUE(source, generation, line)
	)`,
	`CREATE VIRTUAL TABLE IF NOT EXISTS events_fts USING fts5(body, content='events', content_rowid='seq')`,
}

func createSchema(ctx context.Context, db *sql.DB) error {
	for _, s := range schema {
		if _, err := db.ExecContext(ctx, s); err != nil {
			return fmt.Errorf("schema: %w", err)
		}
	}

	return nil
}

const insertEvent = `INSERT INTO events(seq, session, source, generation, line, kind, trust, tool, body,
	payload_hash, payload_size, tokens, ts, prev_hash, hash)
	VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)`

const insertFTS = `INSERT INTO events_fts(rowid, body) VALUES(?, ?)`

// appender writes events inside one transaction through prepared
// statements, chaining each event's hash onto the previous one.
type appender struct {
	ev, fts *sql.Stmt
}

func newAppender(ctx context.Context, tx *sql.Tx) (*appender, error) {
	ev, err := tx.PrepareContext(ctx, insertEvent)
	if err != nil {
		return nil, err
	}
	fts, err := tx.PrepareContext(ctx, insertFTS)
	if err != nil {
		return nil, err
	}

	return &appender{ev: ev, fts: fts}, nil
}

func (a *appender) append(ctx context.Context, seq int64, e event, prev []byte) ([]byte, error) {
	h := sha256.New()
	h.Write(prev)
	h.Write([]byte(e.body))
	sum := h.Sum(nil)
	var ph any
	var ps any
	if e.payload {
		p := sha256.Sum256([]byte(e.body))
		ph, ps = p[:], int64(len(e.body))*8
	}
	_, err := a.ev.ExecContext(ctx, seq, e.session, e.source, 0, e.line, e.kind, e.trust, e.tool, e.body,
		ph, ps, len(e.body)/3+1, e.ts, prev, sum)
	if err != nil {
		return nil, fmt.Errorf("insert event %d: %w", seq, err)
	}
	if _, err := a.fts.ExecContext(ctx, seq, e.body); err != nil {
		return nil, fmt.Errorf("insert fts %d: %w", seq, err)
	}

	return sum, nil
}

func (a *appender) close() {
	a.ev.Close()
	a.fts.Close()
}

func setMeta(ctx context.Context, db *sql.DB, k string, v int64) error {
	_, err := db.ExecContext(ctx, `INSERT INTO meta(k, v) VALUES(?, ?) ON CONFLICT(k) DO UPDATE SET v = excluded.v`, k, v)
	return err
}

func getMeta(ctx context.Context, db *sql.DB, k string) (int64, error) {
	var v int64
	err := db.QueryRowContext(ctx, `SELECT v FROM meta WHERE k = ?`, k).Scan(&v)
	return v, err
}

// lastHash returns the tail of the chain and its seq.
func lastHash(ctx context.Context, q interface {
	QueryRowContext(context.Context, string, ...any) *sql.Row
}) (int64, []byte, error) {
	var seq int64
	var h []byte
	err := q.QueryRowContext(ctx, `SELECT seq, hash FROM events ORDER BY seq DESC LIMIT 1`).Scan(&seq, &h)
	if err == sql.ErrNoRows {
		return 0, make([]byte, 32), nil
	}

	return seq, h, err
}
