//! The store (§8.1, §8.2): `store.db`, which holds the record's writer
//! logs and the seats that write them, through SQLite. Each writer's
//! events are appended inside an immediate transaction, a hundred at a
//! time, which alone assigns their seq and chains them (REC-06,
//! REC-10). The store records its format version, the header's field
//! set and its encoding in `meta` (§8.3), and refuses a store of a
//! newer version (ADM-05).

use std::fmt;
use std::path::Path;
use std::time::Duration;

use record::canonical;
use record::chain::Link;
pub use record::commitment::CommitmentKey;
use record::commitment::{Commitment, commit};
use record::digest::Digest;
use record::event::{EventKind, Provenance};
pub use record::header::{ENCODING, EventHeader, FIELD_SET, SeenHead};
pub use record::id::{Address, RoomId, SeatId, Seq, WriterId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use serde_json::Value;

/// The store's format version.
pub const SCHEMA_VERSION: u32 = 1;

/// How many events one append transaction commits.
pub const BATCH: usize = 100;

/// How long a write waits for another connection's lock (ENG-03).
const BUSY: Duration = Duration::from_secs(5);

/// Whether the store has its schema yet.
const HAS_META: &str = "SELECT count(*) FROM sqlite_master WHERE name = 'meta'";

/// The store's format version.
const VERSION: &str = "SELECT value FROM meta WHERE key = 'schema_version'";

/// The heads one event cites.
const CITED: &str = "SELECT other_writer, other_seq, other_hash FROM event_heads
     WHERE writer = ?1 AND seq = ?2 ORDER BY other_writer";

/// The schema of format version 1.
const SCHEMA: &str = "
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL) STRICT;
CREATE TABLE seats (
    writer TEXT PRIMARY KEY,
    seat TEXT NOT NULL UNIQUE,
    room TEXT NOT NULL,
    certificate TEXT NOT NULL
) STRICT;
CREATE TABLE events (
    writer TEXT NOT NULL,
    seq INTEGER NOT NULL,
    kind TEXT NOT NULL,
    provenance TEXT NOT NULL,
    prev_hash TEXT NOT NULL,
    commitment TEXT NOT NULL,
    hash TEXT NOT NULL,
    content TEXT,
    commitment_key TEXT,
    PRIMARY KEY (writer, seq)
) STRICT, WITHOUT ROWID;
CREATE TABLE event_heads (
    writer TEXT NOT NULL,
    seq INTEGER NOT NULL,
    other_writer TEXT NOT NULL,
    other_seq INTEGER NOT NULL,
    other_hash TEXT NOT NULL,
    PRIMARY KEY (writer, seq, other_writer)
) STRICT, WITHOUT ROWID;
";

/// What can go wrong with the store.
#[derive(Debug)]
pub enum StoreError {
    /// SQLite refused an operation.
    Sqlite(rusqlite::Error),
    /// The store file's permissions cannot be set.
    Io(std::io::Error),
    /// The store is of a newer format version than this cairn's.
    Newer { found: u32 },
    /// A stored value does not parse; the event it belongs to, if any.
    Corrupt {
        address: Option<Address>,
        what: String,
    },
    /// A draft's content has no canonical encoding.
    Encoding(canonical::Error),
    /// A writer has used every seq.
    Full(WriterId),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(e) => write!(f, "the store: {e}"),
            Self::Io(e) => write!(f, "the store file: {e}"),
            Self::Newer { found } => write!(
                f,
                "the store is format version {found}, newer than this cairn's {SCHEMA_VERSION}"
            ),
            Self::Corrupt {
                address: Some(a),
                what,
            } => write!(f, "the stored event {a} is corrupt: {what}"),
            Self::Corrupt {
                address: None,
                what,
            } => write!(f, "the store is corrupt: {what}"),
            Self::Encoding(e) => write!(f, "{e}"),
            Self::Full(w) => write!(f, "writer {w} has no seq left"),
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Sqlite(e) => Some(e),
            Self::Io(e) => Some(e),
            Self::Encoding(e) => Some(e),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sqlite(e)
    }
}

/// An event to append: everything but what its writer's append
/// transaction assigns.
#[derive(Debug, Clone)]
pub struct Draft {
    pub kind: EventKind,
    pub provenance: Provenance,
    pub content: Value,
    pub commitment_key: CommitmentKey,
}

/// A stored event: its header and hash, and its content and commitment
/// key, which purge removes (§8.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stored {
    pub link: Link,
    pub content: Option<Value>,
    pub commitment_key: Option<CommitmentKey>,
}

/// A seat that writes in this store: its writer, id, room and seat
/// certificate, kept as the certificate's own JSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatRecord {
    pub writer: WriterId,
    pub seat: SeatId,
    pub room: RoomId,
    pub certificate: Value,
}

/// An open store.
pub struct Store {
    conn: Connection,
}

impl fmt::Debug for Store {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Store")
            .field("path", &self.conn.path())
            .finish()
    }
}

/// A seq as SQLite stores it: every seq is at most 2^53 - 1, so it
/// fits.
fn sql(seq: Seq) -> i64 {
    seq.get() as i64
}

/// The seq SQLite holds as `n`, if it is one.
fn seq_of(n: i64) -> Option<Seq> {
    u64::try_from(n).ok().and_then(Seq::new)
}

impl Store {
    /// Opens the store at `path`, creating it, readable only by its
    /// owner, when it does not exist.
    ///
    /// # Errors
    ///
    /// Fails when SQLite cannot open it, when it is of a newer format
    /// version, or when its version does not parse.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        restrict(path)?;
        conn.busy_timeout(BUSY)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "FULL")?;
        let mut store = Self { conn };
        store.migrate()?;

        Ok(store)
    }

    /// Creates the schema in a new store, or checks an existing one's
    /// version.
    fn migrate(&mut self) -> Result<(), StoreError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let exists: bool = tx.query_row(HAS_META, [], |r| r.get(0))?;
        if exists {
            let version: String = tx.query_row(VERSION, [], |r| r.get(0))?;
            let found: u32 = version.parse().map_err(|_| StoreError::Corrupt {
                address: None,
                what: format!("schema_version {version:?}"),
            })?;
            if found > SCHEMA_VERSION {
                return Err(StoreError::Newer { found });
            }
            return Ok(());
        }
        tx.execute_batch(SCHEMA)?;
        for (key, value) in [
            ("schema_version", SCHEMA_VERSION.to_string()),
            ("header_field_set", FIELD_SET.to_owned()),
            ("header_encoding", ENCODING.to_owned()),
        ] {
            tx.execute(
                "INSERT INTO meta (key, value) VALUES (?1, ?2)",
                params![key, value],
            )?;
        }

        Ok(tx.commit()?)
    }

    /// The value `meta` holds under `key`.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn meta(&self, key: &str) -> Result<Option<String>, StoreError> {
        Ok(self
            .conn
            .query_row("SELECT value FROM meta WHERE key = ?1", [key], |r| r.get(0))
            .optional()?)
    }

    /// Records a seat.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does, its writer or seat already recorded.
    pub fn add_seat(&mut self, seat: &SeatRecord) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO seats (writer, seat, room, certificate) VALUES (?1, ?2, ?3, ?4)",
            params![
                seat.writer.to_string(),
                seat.seat.to_string(),
                seat.room.to_string(),
                seat.certificate.to_string()
            ],
        )?;

        Ok(())
    }

    /// The seat `writer` belongs to.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or the row does not parse.
    pub fn seat(&self, writer: WriterId) -> Result<Option<SeatRecord>, StoreError> {
        Ok(self
            .seat_rows("WHERE writer = ?1", &writer.to_string())?
            .pop())
    }

    /// Every seat, by writer id.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn seats(&self) -> Result<Vec<SeatRecord>, StoreError> {
        self.seat_rows("WHERE ?1 = ?1", "")
    }

    fn seat_rows(&self, filter: &str, arg: &str) -> Result<Vec<SeatRecord>, StoreError> {
        let mut statement = self.conn.prepare(&format!(
            "SELECT writer, seat, room, certificate FROM seats {filter} ORDER BY writer"
        ))?;
        let rows = statement.query_map([arg], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?;
        rows.map(|row| {
            let (writer, seat, room, certificate) = row?;
            let corrupt = |what: &str| StoreError::Corrupt {
                address: None,
                what: format!("seat {what}"),
            };
            Ok(SeatRecord {
                writer: WriterId::parse(&writer).map_err(|_| corrupt("writer"))?,
                seat: SeatId::parse(&seat).map_err(|_| corrupt("seat"))?,
                room: RoomId::parse(&room).map_err(|_| corrupt("room"))?,
                certificate: serde_json::from_str(&certificate)
                    .map_err(|_| corrupt("certificate"))?,
            })
        })
        .collect()
    }

    /// Every writer with an event or a seat, by writer id.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or an id does not parse.
    pub fn writers(&self) -> Result<Vec<WriterId>, StoreError> {
        let mut statement = self
            .conn
            .prepare("SELECT writer FROM events UNION SELECT writer FROM seats ORDER BY 1")?;
        let rows = statement.query_map([], |r| r.get::<_, String>(0))?;
        rows.map(|row| {
            let text = row?;
            WriterId::parse(&text).map_err(|_| StoreError::Corrupt {
                address: None,
                what: format!("writer id {text:?}"),
            })
        })
        .collect()
    }

    /// Appends `drafts` to `writer`'s log, a hundred to an immediate
    /// transaction. Each transaction reads the writer's newest event,
    /// numbers and chains its events after it, and has its first event
    /// cite every other writer's head that advanced since the writer
    /// last cited it. Returns the new events' addresses.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does, when a draft's content has no canonical
    /// encoding, or when the writer runs out of seqs. The transaction
    /// the failure falls in is rolled back; those before it stay.
    pub fn append(
        &mut self,
        writer: WriterId,
        drafts: &[Draft],
    ) -> Result<Vec<Address>, StoreError> {
        let mut out = Vec::with_capacity(drafts.len());
        for chunk in drafts.chunks(BATCH) {
            let tx = self
                .conn
                .transaction_with_behavior(TransactionBehavior::Immediate)?;
            out.extend(append_chunk(&tx, writer, chunk)?);
            tx.commit()?;
        }

        Ok(out)
    }

    /// The event at `address`.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or its row does not parse.
    pub fn event(&self, address: Address) -> Result<Option<Stored>, StoreError> {
        Ok(self
            .rows(address.writer, "AND seq = ?2", sql(address.seq))?
            .pop())
    }

    /// `writer`'s log, in seq order.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn log(&self, writer: WriterId) -> Result<Vec<Stored>, StoreError> {
        self.rows(writer, "AND ?2 = ?2", 0)
    }

    fn rows(&self, writer: WriterId, filter: &str, seq: i64) -> Result<Vec<Stored>, StoreError> {
        let mut statement = self.conn.prepare(&format!(
            "SELECT seq, kind, provenance, prev_hash, commitment, hash, content, commitment_key
             FROM events WHERE writer = ?1 {filter} ORDER BY seq"
        ))?;
        let rows = statement.query_map(params![writer.to_string(), seq], |r| {
            Ok(Row {
                seq: r.get(0)?,
                kind: r.get(1)?,
                provenance: r.get(2)?,
                prev_hash: r.get(3)?,
                commitment: r.get(4)?,
                hash: r.get(5)?,
                content: r.get(6)?,
                commitment_key: r.get(7)?,
            })
        })?;
        let rows: Vec<Row> = rows.collect::<Result<_, _>>()?;
        rows.into_iter()
            .map(|row| self.stored(writer, row))
            .collect()
    }

    fn stored(&self, writer: WriterId, row: Row) -> Result<Stored, StoreError> {
        let seq = seq_of(row.seq).ok_or_else(|| StoreError::Corrupt {
            address: None,
            what: format!("seq {} of writer {writer}", row.seq),
        })?;
        let address = Address::new(writer, seq);
        let corrupt = |what: &str| StoreError::Corrupt {
            address: Some(address),
            what: what.to_owned(),
        };
        let digest = |text: &str, what: &str| {
            Digest::from_hex(text).map_err(|e| corrupt(&format!("{what}: {e}")))
        };
        let header = EventHeader {
            address,
            kind: EventKind::parse(&row.kind).ok_or_else(|| corrupt("kind"))?,
            provenance: Provenance::parse(&row.provenance).ok_or_else(|| corrupt("provenance"))?,
            prev_hash: digest(&row.prev_hash, "prev_hash")?,
            heads: self.heads(address)?,
            commitment: Commitment::from_hex(&row.commitment)
                .map_err(|e| corrupt(&format!("commitment: {e}")))?,
        };
        let content = row
            .content
            .map(|text| serde_json::from_str(&text).map_err(|e| corrupt(&format!("content: {e}"))))
            .transpose()?;
        let commitment_key = row
            .commitment_key
            .map(|text| {
                CommitmentKey::from_hex(&text).map_err(|e| corrupt(&format!("commitment_key: {e}")))
            })
            .transpose()?;

        Ok(Stored {
            link: Link {
                header,
                hash: digest(&row.hash, "hash")?,
            },
            content,
            commitment_key,
        })
    }

    /// The heads the event at `address` cites.
    fn heads(&self, address: Address) -> Result<Vec<SeenHead>, StoreError> {
        let mut statement = self.conn.prepare(CITED)?;
        let rows =
            statement.query_map(params![address.writer.to_string(), sql(address.seq)], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })?;
        rows.map(|row| {
            let (writer, seq, hash) = row?;
            let corrupt = |what: &str| StoreError::Corrupt {
                address: Some(address),
                what: format!("cited head {what}"),
            };
            let writer = WriterId::parse(&writer).map_err(|_| corrupt("writer"))?;
            let seq = seq_of(seq).ok_or_else(|| corrupt("seq"))?;
            let head = Digest::from_hex(&hash).map_err(|_| corrupt("hash"))?;
            Ok(SeenHead::new(Address::new(writer, seq), head))
        })
        .collect()
    }
}

/// One event row as SQLite returns it.
struct Row {
    seq: i64,
    kind: String,
    provenance: String,
    prev_hash: String,
    commitment: String,
    hash: String,
    content: Option<String>,
    commitment_key: Option<String>,
}

/// Appends one transaction's drafts to `writer`.
fn append_chunk(
    tx: &Transaction<'_>,
    writer: WriterId,
    drafts: &[Draft],
) -> Result<Vec<Address>, StoreError> {
    let id = writer.to_string();
    let newest: Option<(i64, String)> = tx
        .query_row(
            "SELECT seq, hash FROM events WHERE writer = ?1 ORDER BY seq DESC LIMIT 1",
            [&id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let (mut next, mut prev_hash) = match newest {
        Some((seq, hash)) => (
            u64::try_from(seq).unwrap_or(0) + 1,
            Digest::from_hex(&hash).map_err(|e| StoreError::Corrupt {
                address: seq_of(seq).map(|s| Address::new(writer, s)),
                what: format!("hash: {e}"),
            })?,
        ),
        None => (1, Digest::ZERO),
    };
    let mut heads = advanced_heads(tx, writer)?;
    let mut out = Vec::with_capacity(drafts.len());
    for draft in drafts {
        let seq = Seq::new(next).ok_or(StoreError::Full(writer))?;
        let header = EventHeader {
            address: Address::new(writer, seq),
            kind: draft.kind,
            provenance: draft.provenance.clone(),
            prev_hash,
            heads: std::mem::take(&mut heads),
            commitment: commit(&draft.commitment_key, &draft.content)
                .map_err(StoreError::Encoding)?,
        };
        let hash = header.hash();
        tx.execute(
            "INSERT INTO events (writer, seq, kind, provenance, prev_hash, commitment, hash, content, commitment_key)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                id,
                sql(seq),
                header.kind.as_str(),
                header.provenance.name(),
                prev_hash.to_hex(),
                header.commitment.to_string(),
                hash.to_hex(),
                draft.content.to_string(),
                draft.commitment_key.to_hex()
            ],
        )?;
        for seen in &header.heads {
            tx.execute(
                "INSERT INTO event_heads (writer, seq, other_writer, other_seq, other_hash) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, sql(seq), seen.address.writer.to_string(), sql(seen.address.seq), seen.head.to_hex()],
            )?;
        }
        out.push(header.address);
        prev_hash = hash;
        next += 1;
    }

    Ok(out)
}

/// Every other writer's newest event that lies beyond what `writer`
/// last cited of it.
fn advanced_heads(tx: &Transaction<'_>, writer: WriterId) -> Result<Vec<SeenHead>, StoreError> {
    let mut statement = tx.prepare(
        "SELECT e.writer, e.seq, e.hash FROM events e
         JOIN (SELECT writer, max(seq) AS seq FROM events WHERE writer != ?1 GROUP BY writer) n
           ON e.writer = n.writer AND e.seq = n.seq
         WHERE e.seq > coalesce(
           (SELECT max(other_seq) FROM event_heads h WHERE h.writer = ?1 AND h.other_writer = e.writer), 0)
         ORDER BY e.writer",
    )?;
    let rows = statement.query_map([writer.to_string()], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, String>(2)?,
        ))
    })?;
    rows.map(|row| {
        let (other, seq, hash) = row?;
        let address = WriterId::parse(&other)
            .ok()
            .zip(seq_of(seq))
            .map(|(w, s)| Address::new(w, s));
        let corrupt = || StoreError::Corrupt {
            address,
            what: "the head it would cite".to_owned(),
        };
        let head = Digest::from_hex(&hash).map_err(|_| corrupt())?;
        Ok(SeenHead::new(address.ok_or_else(corrupt)?, head))
    })
    .collect()
}

/// Makes the store file readable only by its owner (§8.1).
fn restrict(path: &Path) -> Result<(), StoreError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .map_err(StoreError::Io)?;
    }
    #[cfg(not(unix))]
    let _ = path;

    Ok(())
}

#[cfg(test)]
mod tests;
