//! The search index and room state in one SQLite file, derived only
//! from the events a node holds and its key set (I10, ADM-08).
//!
//! Built from nothing in (writer, seq) order, in one transaction, so
//! the same held writers and key set give the same bytes whatever order
//! their segments arrived in.

use std::path::Path;

use rusqlite::{Connection, OpenFlags, params};

use crate::node::Node;
use crate::record::Address;
use crate::room::RoomState;
use crate::{Result, hex};

const SCHEMA: &str = "
PRAGMA journal_mode = OFF;
PRAGMA synchronous = OFF;
CREATE TABLE events(
    id INTEGER PRIMARY KEY,
    writer TEXT NOT NULL,
    seq INTEGER NOT NULL,
    kind TEXT NOT NULL,
    provenance TEXT NOT NULL,
    origin TEXT NOT NULL,
    trust TEXT NOT NULL,
    commitment TEXT NOT NULL,
    hash TEXT NOT NULL,
    run TEXT,
    tool TEXT,
    ts INTEGER,
    text TEXT,
    UNIQUE(writer, seq)
);
CREATE VIRTUAL TABLE events_fts USING fts5(text, content = 'events', content_rowid = 'id');
CREATE TABLE branch_links(
    room TEXT NOT NULL,
    repository TEXT NOT NULL,
    branch TEXT NOT NULL,
    writer TEXT NOT NULL,
    seq INTEGER NOT NULL,
    commitment TEXT NOT NULL,
    PRIMARY KEY(room, repository, branch)
) WITHOUT ROWID;
CREATE TABLE conflicts(
    void_writer TEXT NOT NULL,
    void_seq INTEGER NOT NULL,
    stands_writer TEXT NOT NULL,
    stands_seq INTEGER NOT NULL,
    rule TEXT NOT NULL,
    PRIMARY KEY(void_writer, void_seq)
) WITHOUT ROWID;
";

/// Builds the index at `path`, which must not exist yet.
pub fn build(path: &Path, node: &Node, room: &RoomState) -> Result<()> {
    let mut conn = Connection::open(path)?;
    conn.execute_batch(SCHEMA)?;
    let tx = conn.transaction()?;
    {
        let mut event = tx.prepare(
            "INSERT INTO events(writer, seq, kind, provenance, origin, trust, commitment, hash, run, tool, ts, text)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        )?;
        let mut fts = tx.prepare("INSERT INTO events_fts(rowid, text) VALUES (?1, ?2)")?;
        for e in node.events() {
            let h = &e.header;
            let content = e.body.as_ref().map(|b| &b.content);
            let field = |k: &str| content.and_then(|c| c.get(k));
            let text = field("text").and_then(|v| v.as_str());
            event.execute(params![
                h.writer,
                h.seq as i64,
                h.kind,
                h.provenance,
                node.origin(&h.writer),
                node.trust(e),
                hex(&h.commitment),
                hex(&e.hash),
                field("run").and_then(|v| v.as_str()),
                field("tool").and_then(|v| v.as_str()),
                field("ts").and_then(|v| v.as_i64()),
                text,
            ])?;
            if let Some(t) = text {
                fts.execute(params![tx.last_insert_rowid(), t])?;
            }
        }
        let mut link = tx.prepare("INSERT INTO branch_links VALUES (?1, ?2, ?3, ?4, ?5, ?6)")?;
        for l in &room.links {
            link.execute(params![
                l.room,
                l.repository,
                l.branch,
                l.act.writer,
                l.act.seq as i64,
                hex(&l.commitment)
            ])?;
        }
        let mut conflict = tx.prepare("INSERT INTO conflicts VALUES (?1, ?2, ?3, ?4, ?5)")?;
        for c in &room.conflicts {
            conflict.execute(params![
                c.void.writer,
                c.void.seq as i64,
                c.stands.writer,
                c.stands.seq as i64,
                c.rule
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

fn open(path: &Path) -> Result<Connection> {
    Ok(Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?)
}

fn address(writer: String, seq: i64) -> Address {
    Address {
        writer,
        seq: seq as u64,
    }
}

/// One indexed event's identity and trust, as the index holds them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub address: Address,
    pub provenance: String,
    pub origin: String,
    pub trust: String,
}

/// Every indexed event, in (writer, seq) order.
pub fn rows(path: &Path) -> Result<Vec<Row>> {
    let conn = open(path)?;
    let mut q = conn.prepare(
        "SELECT writer, seq, provenance, origin, trust FROM events ORDER BY writer, seq",
    )?;
    let rows = q.query_map([], |r| {
        Ok(Row {
            address: address(r.get(0)?, r.get(1)?),
            provenance: r.get(2)?,
            origin: r.get(3)?,
            trust: r.get(4)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// The addresses whose text matches the phrase.
pub fn phrase_matches(path: &Path, phrase: &str) -> Result<Vec<Address>> {
    let conn = open(path)?;
    let mut q = conn.prepare(
        "SELECT e.writer, e.seq FROM events_fts f JOIN events e ON e.id = f.rowid
         WHERE events_fts MATCH ?1 ORDER BY e.writer, e.seq",
    )?;
    let quoted = format!("\"{}\"", phrase.replace('"', "\"\""));
    let rows = q.query_map([quoted], |r| Ok(address(r.get(0)?, r.get(1)?)))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// The standing branch links the index holds, as (branch, act).
pub fn branch_links(path: &Path) -> Result<Vec<(String, Address)>> {
    let conn = open(path)?;
    let mut q = conn.prepare(
        "SELECT branch, writer, seq FROM branch_links ORDER BY room, repository, branch",
    )?;
    let rows = q.query_map([], |r| Ok((r.get(0)?, address(r.get(1)?, r.get(2)?))))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}
