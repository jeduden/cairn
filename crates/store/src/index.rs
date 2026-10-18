//! The search index (§8.2, REC-11, RCL-02, SEC-04): an FTS5 table over
//! each event's text in `store.db`, beside the record it derives from.
//! It is a derived artifact (I10): a function of the writer logs the
//! store holds, whatever order they arrived in, and `Store::rebuild_index`
//! derives it again from the record alone.
//!
//! - It keeps no text of its own (contentless): the record stays the one
//!   place content lives. An event's text is its content's `text`, in the
//!   payload store too, up to the indexing cap of [`DEFAULT_CAP`] bytes.
//! - A row's id derives from the event's address, never from arrival
//!   (S12): its seq in the high bits, a slot taken from its writer id in
//!   the low ones. Newer events of a writer so have the larger ids.
//!   Where two writers' ids share a slot the smaller writer id keeps it
//!   and the other's events are left out, whichever arrived first; they
//!   are named by `Store::index_displaced`. A seq of [`SEQ_LIMIT`] or more is left
//!   out too.
//! - Search ranks bounded: BM25 with length normalization over the
//!   newest [`WINDOW`] matches after the filters (S2), damped per run,
//!   under a deadline (SEC-04).

use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::Duration;

use record::digest::Digest;
use record::id::{Address, RoomId, Seq, WriterId};
use rusqlite::types::Value as Sql;
use rusqlite::{Connection, ErrorCode, OptionalExtension, params, params_from_iter};
use search::Compiled;
use search::rank::{self, Candidate, Hit};
use serde_json::Value;

use crate::StoreError;

/// REC-11's indexing cap: the most bytes of one event's text indexed.
pub const DEFAULT_CAP: usize = 1 << 20;

/// How many of the newest matches a search ranks (S2).
pub const WINDOW: usize = 2_000;

/// The tokenizer the index is built with, part of its content (§8.3).
pub const TOKENIZER: &str = "unicode61 remove_diacritics 2";

/// The bits of a row id that hold the writer's slot.
pub const SLOT_BITS: u32 = 35;

/// The slot's mask.
const SLOT_MASK: i64 = (1 << SLOT_BITS) - 1;

/// A seq from here on is not indexed: it would not fit a row id.
pub const SEQ_LIMIT: u64 = 1 << 28;

/// How long a search waits for its deadline watcher to end once the
/// search has (ENG-03).
const WATCHER_GRACE: Duration = Duration::from_secs(5);

/// How many events a rebuild reads at a time.
pub(crate) const CHUNK: i64 = 100;

/// The tables of the search index.
pub(crate) const SCHEMA: &str = "
CREATE VIRTUAL TABLE IF NOT EXISTS events_fts USING fts5(
    text, content = '', contentless_delete = 1, tokenize = 'unicode61 remove_diacritics 2');
CREATE VIRTUAL TABLE IF NOT EXISTS events_vocab USING fts5vocab(events_fts, 'instance');
CREATE TABLE IF NOT EXISTS search_writers (
    slot INTEGER PRIMARY KEY,
    writer TEXT NOT NULL UNIQUE
) STRICT;
";

/// Gives a slot to a writer, takes it from the writer that held it, and
/// drops that writer's rows.
const CLAIM_SLOT: &str = "INSERT INTO search_writers (slot, writer) VALUES (?1, ?2)";
const TAKE_SLOT: &str = "UPDATE search_writers SET writer = ?2 WHERE slot = ?1";
const DROP_SLOT_ROWS: &str = "DELETE FROM events_fts WHERE (rowid & ?1) = ?2";

/// Empties the index.
const CLEAR: &str =
    "INSERT INTO events_fts (events_fts) VALUES ('delete-all'); DELETE FROM search_writers;";

/// The writers with events but no slot.
const DISPLACED: &str = "SELECT DISTINCT writer FROM events
    WHERE writer NOT IN (SELECT writer FROM search_writers) ORDER BY writer";

/// What the canonical digest reads of the index.
const SCHEMA_OBJECTS: &str = "SELECT sql FROM sqlite_master
    WHERE name IN ('events_fts', 'events_vocab', 'search_writers') ORDER BY name";
const POSTINGS: &str =
    "SELECT term, doc, col, offset FROM events_vocab ORDER BY term, doc, col, offset";

/// The settings the index is built under, kept in `meta` (§8.3).
pub(crate) fn settings() -> [(&'static str, String); 2] {
    [
        ("search_tokenizer", TOKENIZER.to_owned()),
        ("search_index_cap", DEFAULT_CAP.to_string()),
    ]
}

/// Whether the store has its search index yet.
pub(crate) fn exists(conn: &Connection) -> Result<bool, StoreError> {
    let found = conn.query_row(
        "SELECT count(*) FROM sqlite_master WHERE name = 'events_fts'",
        [],
        |r| r.get::<_, bool>(0),
    );

    Ok(found?)
}

/// The slot a writer takes in row ids: five bytes of its id.
fn slot_of(writer: WriterId) -> i64 {
    let bytes = writer.digest().as_bytes();
    let wide = bytes[..5]
        .iter()
        .fold(0_u64, |n, b| (n << 8) | u64::from(*b));

    i64::try_from(wide).map_or(0, |n| n & SLOT_MASK)
}

/// The row id of the event at `seq` of the writer in `slot`.
fn row_id(slot: i64, seq: Seq) -> i64 {
    i64::try_from(seq.get()).map_or(0, |n| (n << SLOT_BITS) | slot)
}

/// The row id the index gives the event at `address`; none for a seq
/// from [`SEQ_LIMIT`] on, which is not indexed.
#[must_use]
pub fn row_id_of(address: Address) -> Option<i64> {
    (address.seq.get() < SEQ_LIMIT).then(|| row_id(slot_of(address.writer), address.seq))
}

/// An event's text as the index takes it: the content's `text`, cut to
/// at most `cap` bytes at a character boundary; none for content with
/// none.
#[must_use]
pub fn text_of(content: &Value, cap: usize) -> Option<&str> {
    let text = content.get("text")?.as_str()?;
    let mut end = text.len().min(cap);
    while !text.is_char_boundary(end) {
        end -= 1;
    }

    Some(&text[..end])
}

/// The index as one append transaction fills it, for one writer.
pub(crate) struct Indexing<'t> {
    insert: Option<rusqlite::Statement<'t>>,
    slot: i64,
}

impl<'t> Indexing<'t> {
    /// Claims `writer`'s slot, or finds another writer holding it.
    pub(crate) fn open(conn: &'t Connection, writer: WriterId) -> Result<Self, StoreError> {
        let slot = slot_of(writer);
        let held: Option<String> = conn
            .query_row(
                "SELECT writer FROM search_writers WHERE slot = ?1",
                [slot],
                |r| r.get(0),
            )
            .optional()?;
        let mine = writer.to_string();
        let owner = match held {
            Some(held) if held < mine => false,
            Some(held) if held == mine => true,
            Some(_) => {
                // The smaller writer id keeps the slot: the one that
                // held it is left out, whichever arrived first.
                conn.execute(DROP_SLOT_ROWS, [SLOT_MASK, slot])?;
                conn.execute(TAKE_SLOT, params![slot, mine])?;
                true
            }
            None => {
                conn.execute(CLAIM_SLOT, params![slot, mine])?;
                true
            }
        };
        let insert = if owner {
            Some(conn.prepare("INSERT INTO events_fts (rowid, text) VALUES (?1, ?2)")?)
        } else {
            None
        };

        Ok(Self { insert, slot })
    }

    /// Indexes the event at `seq` whose content is `content`.
    pub(crate) fn add(&mut self, seq: Seq, content: &Value) -> Result<(), StoreError> {
        let (Some(insert), Some(text)) = (self.insert.as_mut(), text_of(content, DEFAULT_CAP))
        else {
            return Ok(());
        };
        if seq.get() < SEQ_LIMIT {
            insert.execute(params![row_id(self.slot, seq), text])?;
        }

        Ok(())
    }
}

/// Empties the index, leaving its tables.
pub(crate) fn clear(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch(CLEAR)?;

    Ok(())
}

/// The writers with events whose slot another writer holds, so their
/// events are not searchable (I6).
pub(crate) fn displaced(conn: &Connection) -> Result<Vec<String>, StoreError> {
    let mut statement = conn.prepare(DISPLACED)?;
    let rows = statement.query_map([], |r| r.get::<_, String>(0))?;

    Ok(rows.collect::<Result<_, _>>()?)
}

/// What a search asks of the index.
#[derive(Debug, Clone, Copy)]
pub struct Search<'a> {
    pub query: &'a Compiled,
    /// The room whose writers are searched.
    pub room: RoomId,
    /// One run, by its writer id.
    pub run: Option<WriterId>,
    /// Provenance classes, by name; none means any.
    pub provenance: &'a [String],
    /// Event kinds, by name; none means any.
    pub kinds: &'a [String],
    /// Transcript times from here on.
    pub since: Option<&'a str>,
    /// Transcript times up to here, a date taking its whole day.
    pub until: Option<&'a str>,
    /// How many hits to return, at most [`rank::MAX_HITS`].
    pub k: usize,
    /// How long the search may run (SEC-04).
    pub deadline: Duration,
}

/// The SQL and arguments of the window query for `search`.
fn window(search: &Search<'_>) -> (String, Vec<Sql>) {
    let mut sql = String::from(
        "SELECT events_fts.rowid, bm25(events_fts), w.writer
         FROM events_fts
         JOIN search_writers w ON w.slot = (events_fts.rowid & 34359738367)
         JOIN seats s ON s.writer = w.writer
         JOIN events e ON e.writer = w.writer AND e.seq = (events_fts.rowid >> 35)
         WHERE events_fts MATCH ? AND s.room = ?",
    );
    let mut args = vec![
        Sql::Text(search.query.expression().to_owned()),
        Sql::Text(search.room.to_string()),
    ];
    if let Some(run) = search.run {
        sql.push_str(" AND w.writer = ?");
        args.push(Sql::Text(run.to_string()));
    }
    for (column, values) in [("provenance", search.provenance), ("kind", search.kinds)] {
        if !values.is_empty() {
            let marks = vec!["?"; values.len()].join(", ");
            sql.push_str(&format!(" AND e.{column} IN ({marks})"));
            args.extend(values.iter().cloned().map(Sql::Text));
        }
    }
    if let Some(since) = search.since {
        sql.push_str(" AND e.transcript_timestamp >= ?");
        args.push(Sql::Text(since.to_owned()));
    }
    if let Some(until) = search.until {
        sql.push_str(" AND substr(e.transcript_timestamp, 1, length(?)) <= ?");
        args.extend([Sql::Text(until.to_owned()), Sql::Text(until.to_owned())]);
    }
    sql.push_str(&format!(" ORDER BY events_fts.rowid DESC LIMIT {WINDOW}"));

    (sql, args)
}

/// The newest [`WINDOW`] matches of `search`.
fn candidates(conn: &Connection, search: &Search<'_>) -> Result<Vec<Candidate>, StoreError> {
    let (sql, args) = window(search);
    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map(params_from_iter(args), |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, f64>(1)?,
            r.get::<_, String>(2)?,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (id, bm25, writer) = row?;
        let corrupt = || StoreError::Corrupt {
            address: None,
            what: "the search index names an event that is not there".to_owned(),
        };
        let writer = WriterId::parse(&writer).map_err(|_| corrupt())?;
        let seq = u64::try_from(id >> SLOT_BITS).ok().and_then(Seq::new);
        out.push(Candidate {
            address: Address::new(writer, seq.ok_or_else(corrupt)?),
            relevance: -bm25,
        });
    }

    Ok(out)
}

/// Runs `work` on `conn`, interrupting the statement it runs once
/// `deadline` has passed: a deadline already passed cancels it before it
/// starts (SEC-04, ENG-03).
fn within<T>(
    conn: &Connection,
    deadline: Duration,
    work: impl FnOnce() -> Result<T, StoreError>,
) -> Result<T, StoreError> {
    if deadline.is_zero() {
        return Err(StoreError::Cancelled);
    }
    let handle = conn.get_interrupt_handle();
    let (done, wait) = mpsc::channel::<()>();
    let (ended, watched) = mpsc::channel::<()>();
    thread::spawn(move || {
        if wait.recv_timeout(deadline) == Err(RecvTimeoutError::Timeout) {
            handle.interrupt();
        }
        let _ = ended.send(());
    });
    let out = work();
    // The watcher is done once it hears the work has ended, or at once
    // if it fired first; wait for it, so it never interrupts a
    // statement that starts after this one.
    let _ = done.send(());
    let _ = watched.recv_timeout(WATCHER_GRACE);
    match out {
        Err(StoreError::Sqlite(rusqlite::Error::SqliteFailure(e, _)))
            if e.code == ErrorCode::OperationInterrupted =>
        {
            Err(StoreError::Cancelled)
        }
        other => other,
    }
}

/// Searches the index for `search`, cancelled at its deadline.
pub(crate) fn run(conn: &Connection, search: &Search<'_>) -> Result<Vec<Hit>, StoreError> {
    if !exists(conn)? {
        return Err(StoreError::NoIndex);
    }
    let found = within(conn, search.deadline, || candidates(conn, search))?;

    Ok(rank::rank(&found, search.k))
}

/// The canonical digest of the index's content (§8.3): its settings,
/// its schema, the slot each writer holds and every posting (term,
/// event, position), in key order, folded into one SHA-256. A row id
/// derives from an address, so no storage layout reaches it.
pub(crate) fn digest(conn: &Connection) -> Result<Digest, StoreError> {
    let mut state = Fold(Digest::of(&[b"cairn search index digest v1"]));
    for (key, _) in settings() {
        let value: String =
            conn.query_row("SELECT value FROM meta WHERE key = ?1", [key], |r| r.get(0))?;
        state.add(&[key.as_bytes(), b"\0", value.as_bytes(), b"\0"]);
    }
    let mut schema = conn.prepare(SCHEMA_OBJECTS)?;
    for sql in schema.query_map([], |r| r.get::<_, String>(0))? {
        state.add(&[sql?.as_bytes(), b"\0"]);
    }
    let mut writers = conn.prepare("SELECT slot, writer FROM search_writers ORDER BY writer")?;
    for row in writers.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))? {
        let (slot, writer) = row?;
        state.add(&[&slot.to_be_bytes(), writer.as_bytes(), b"\0"]);
    }
    let mut postings = conn.prepare(POSTINGS)?;
    let rows = postings.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, i64>(3)?,
        ))
    })?;
    for row in rows {
        let (term, doc, col, offset) = row?;
        let (doc, offset) = (doc.to_be_bytes(), offset.to_be_bytes());
        state.add(&[term.as_bytes(), b"\0", &doc, col.as_bytes(), b"\0", &offset]);
    }

    Ok(state.0)
}

/// A digest folded over the parts of each row in turn.
struct Fold(Digest);

impl Fold {
    fn add(&mut self, parts: &[&[u8]]) {
        let mut all: Vec<&[u8]> = vec![self.0.as_bytes().as_slice()];
        all.extend(parts);
        let next = Digest::of(&all);
        self.0 = next;
    }
}

#[cfg(test)]
mod tests;
