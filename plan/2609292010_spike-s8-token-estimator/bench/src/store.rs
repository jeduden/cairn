//! Spike S2's store: the same schema, settings and append, opened on a
//! store `s2bench load` built. S8 adds a pins table for the restore
//! block a `SessionStart` hook handler builds.

use std::path::Path;
use std::time::Duration;

use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction, params};
use sha2::{Digest, Sha256};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// ADR-01's settings, as S2 opened the store.
pub fn open(path: &Path) -> Result<Connection> {
    let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let db = Connection::open_with_flags(path, flags)?;
    db.busy_timeout(Duration::from_secs(20))?;
    db.pragma_update(None, "journal_mode", "WAL")?;
    db.pragma_update(None, "synchronous", "NORMAL")?;
    db.set_prepared_statement_cache_capacity(32);
    Ok(db)
}

/// BEGIN IMMEDIATE, the write lock taken up front (ADR-01).
pub fn begin(db: &mut Connection) -> Result<Transaction<'_>> {
    Ok(db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?)
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

/// One event to append.
pub struct Event<'a> {
    pub source: i64,
    pub line: i64,
    pub kind: &'a str,
    pub tool: Option<&'a str>,
    pub body: &'a str,
}

const INSERT_EVENT: &str = "INSERT INTO events(seq, session, source, generation, line, kind, trust, tool, body,
    payload_hash, payload_size, tokens, ts, prev_hash, hash)
    VALUES(?,?,?,0,?,?,0,?,?,NULL,NULL,?,?,?,?)";

const INSERT_FTS: &str = "INSERT INTO events_fts(rowid, body) VALUES(?, ?)";

/// Appends after the chain's tail inside `tx`, as S2's append does.
pub fn append(tx: &Transaction, e: &Event) -> Result<i64> {
    let (seq, prev) = last_hash(tx)?;
    let seq = seq + 1;
    let mut h = Sha256::new();
    h.update(&prev);
    h.update(e.body.as_bytes());
    let sum = h.finalize().to_vec();
    tx.prepare_cached(INSERT_EVENT)?.execute(params![
        seq,
        format!("ses-{}", e.source),
        e.source,
        e.line,
        e.kind,
        e.tool,
        e.body,
        crate::estimate::estimate(e.body) as i64,
        1_790_000_000_000i64 + seq,
        prev,
        sum
    ])?;
    tx.prepare_cached(INSERT_FTS)?.execute(params![seq, e.body])?;
    Ok(seq)
}

/// The next free line of a source.
pub fn next_line(db: &Connection, source: i64) -> Result<i64> {
    let max: Option<i64> = db.query_row(
        "SELECT max(line) FROM events WHERE source = ? AND generation = 0",
        [source],
        |r| r.get(0),
    )?;
    Ok(max.map_or(0, |m| m + 1))
}

const PINS: &str = "CREATE TABLE IF NOT EXISTS pins(
    id INTEGER PRIMARY KEY,
    room TEXT NOT NULL,
    priority INTEGER NOT NULL,
    active INTEGER NOT NULL,
    body TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS pins_room ON pins(room, active, priority);";

/// Adds `n` pins over five rooms, each up to 1,000 characters (PIN-03).
pub fn add_pins(db: &Connection, n: i64) -> Result<()> {
    db.execute_batch(PINS)?;
    let words = ["never", "push", "to", "main", "run", "the", "tests", "before", "commit", "keep",
        "error", "messages", "lowercase", "UTC", "timestamps", "budget", "ask", "first"];
    for i in 0..n {
        let len = 40 + (i * 7919 % 960) as usize;
        let mut body = String::new();
        let mut k = i as usize;
        while body.len() < len {
            body.push_str(words[k % words.len()]);
            body.push(' ');
            k = k * 31 + 7;
        }
        body.truncate(len);
        db.execute(
            "INSERT INTO pins(room, priority, active, body) VALUES(?, ?, 1, ?)",
            params![format!("r{}", i % 5), i % 3, body],
        )?;
    }
    Ok(())
}
