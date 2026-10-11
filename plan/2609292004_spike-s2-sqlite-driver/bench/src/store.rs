//! The store: SRS §8.2's events record and its FTS5 projection, the
//! same schema, settings and statements as the Go harness.

use std::path::Path;
use std::time::Duration;

use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction, params};
use sha2::{Digest, Sha256};

use crate::corpus::Event;

/// The spike's fixed 256-bit key for the OQ-04 probe; key management
/// is out of its scope.
pub const TEST_KEY: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Opens the store with ADR-01's settings: WAL, NORMAL sync, a busy
/// timeout, and BEGIN IMMEDIATE for every read-write transaction
/// (`begin`). With `key` it keys SQLCipher first, before the file is
/// read; that needs the `cipher` build.
pub fn open(path: &Path, key: bool, query_only: bool) -> Result<Connection> {
    let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
        | OpenFlags::SQLITE_OPEN_CREATE
        | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let db = Connection::open_with_flags(path, flags)?;
    if key {
        db.pragma_update(None, "key", format!("x'{TEST_KEY}'"))?;
    }
    db.busy_timeout(Duration::from_secs(20))?;
    db.pragma_update(None, "journal_mode", "WAL")?;
    db.pragma_update(None, "synchronous", "NORMAL")?;
    if query_only {
        db.pragma_update(None, "query_only", 1)?;
    }
    db.set_prepared_statement_cache_capacity(32);
    Ok(db)
}

/// BEGIN IMMEDIATE, the write lock taken up front (ADR-01).
pub fn begin(db: &mut Connection) -> Result<Transaction<'_>> {
    Ok(db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?)
}

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS meta(k TEXT PRIMARY KEY, v INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS events(
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
);
CREATE VIRTUAL TABLE IF NOT EXISTS events_fts USING fts5(body, content='events', content_rowid='seq');
";

pub fn create_schema(db: &Connection) -> Result<()> {
    db.execute_batch(SCHEMA)?;
    Ok(())
}

const INSERT_EVENT: &str = "INSERT INTO events(seq, session, source, generation, line, kind, trust, tool, body,
    payload_hash, payload_size, tokens, ts, prev_hash, hash)
    VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)";

const INSERT_FTS: &str = "INSERT INTO events_fts(rowid, body) VALUES(?, ?)";

/// Appends one event inside `tx`, chaining its hash onto `prev`, and
/// returns the new tail. The statements come from the connection's
/// cache, prepared once per connection.
pub fn append(tx: &Transaction, seq: i64, e: &Event, prev: &[u8]) -> Result<Vec<u8>> {
    let mut h = Sha256::new();
    h.update(prev);
    h.update(e.body.as_bytes());
    let sum = h.finalize().to_vec();
    let (ph, ps) = if e.payload {
        let p = Sha256::digest(e.body.as_bytes()).to_vec();
        (Some(p), Some(e.body.len() as i64 * 8))
    } else {
        (None, None)
    };
    tx.prepare_cached(INSERT_EVENT)?.execute(params![
        seq,
        e.session,
        e.source,
        0,
        e.line,
        e.kind,
        e.trust,
        e.tool,
        e.body,
        ph,
        ps,
        e.body.len() as i64 / 3 + 1,
        e.ts,
        prev,
        sum
    ])?;
    tx.prepare_cached(INSERT_FTS)?.execute(params![seq, e.body])?;
    Ok(sum)
}

pub fn set_meta(db: &Connection, k: &str, v: i64) -> Result<()> {
    db.execute(
        "INSERT INTO meta(k, v) VALUES(?, ?) ON CONFLICT(k) DO UPDATE SET v = excluded.v",
        params![k, v],
    )?;
    Ok(())
}

pub fn get_meta(db: &Connection, k: &str) -> Result<i64> {
    Ok(db.query_row("SELECT v FROM meta WHERE k = ?", [k], |r| r.get(0))?)
}

/// The tail of the chain and its seq.
pub fn last_hash(db: &Connection) -> Result<(i64, Vec<u8>)> {
    let row = db
        .query_row("SELECT seq, hash FROM events ORDER BY seq DESC LIMIT 1", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .optional()?;
    Ok(row.unwrap_or((0, vec![0; 32])))
}

/// Adds appended events and their text to the store's counters.
pub fn add_meta(db: &Connection, events: i64, text: i64) -> Result<()> {
    let n = get_meta(db, "events")?;
    let tb = get_meta(db, "text_bytes")?;
    set_meta(db, "events", n + events)?;
    set_meta(db, "text_bytes", tb + text)?;
    Ok(())
}
