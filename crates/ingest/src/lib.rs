//! `cairn ingest` (REC-01's first step): reads one transcript into the
//! record. Each `cairn ingest` starts an ingest seat in the personal room,
//! certified by the node's device key, and appends every line of the
//! transcript to that seat's writer as one event: kind `unparsed_line`,
//! provenance `unparsed`, its content the line's number and text. The
//! transcript parser that classes each line lands with phase 7 of plan
//! 2609292012; until then no line is classed or trusted, and the one
//! ingest seat stands in for all of the transcript's runs, which that
//! parser splits it by.
//!
//! The transcript is read, every line redacted (SEC-08) and every
//! commitment key drawn before anything is recorded, so a transcript
//! that cannot be read records nothing, and no secret the redactor
//! matches reaches the store. The harness account identifiers are
//! learnt from the whole transcript first, so a line before the one
//! that names them is redacted too.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use administration::AdminError;
use administration::home::Home;
use administration::node::Node;
use key::KeyError;
use key::random::{Entropy, draw};
use record::commitment::CommitmentKey;
use record::event::{EventKind, Provenance};
use record::id::{RoomId, SeatId, WriterId};
use redaction::Redactor;
use redaction::account::HarnessAccountIdentifiers;
use serde_json::{Value, json};
use store::{NewEvent, Store, StoreError};

/// What one ingest recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ingested {
    /// The ingest seat's writer.
    pub writer: WriterId,
    pub seat: SeatId,
    /// The personal room the seat is in.
    pub room: RoomId,
    /// How many events it appended.
    pub events: usize,
    /// The rule of each redaction made, line by line.
    pub redactions: Vec<String>,
    /// The transcript's path redacted the way its lines were, the
    /// harness account identifiers it taught among the rules: the path
    /// as it may be logged (SEC-08).
    pub redacted_path: String,
}

/// Why an ingest recorded nothing, or stopped part way.
#[derive(Debug)]
pub enum IngestError {
    /// The transcript cannot be read.
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    /// The transcript is not UTF-8.
    NotUtf8(PathBuf),
    /// The transcript holds no line.
    Empty(PathBuf),
    /// A harness account identifier is too long to redact.
    Redaction(redaction::PatternError),
    /// The random source failed.
    Key(KeyError),
    /// The home or the node failed.
    Admin(AdminError),
    /// The store failed.
    Store(StoreError),
    /// The store failed after `landed` of the transcript's `lines`
    /// lines were recorded in `writer`'s log; those stay.
    Partial {
        writer: WriterId,
        landed: usize,
        lines: usize,
        source: StoreError,
    },
}

impl fmt::Display for IngestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(f, "read the transcript {}: {source}", path.display())
            }
            Self::NotUtf8(path) => write!(f, "the transcript {} is not UTF-8", path.display()),
            Self::Empty(path) => write!(f, "the transcript {} holds no line", path.display()),
            Self::Redaction(e) => write!(f, "redact the transcript: {e}"),
            Self::Key(e) => write!(f, "{e}"),
            Self::Admin(e) => write!(f, "{e}"),
            Self::Store(e) => write!(f, "{e}"),
            Self::Partial {
                writer,
                landed,
                lines,
                source,
            } => write!(
                f,
                "writer {writer} recorded {landed} of {lines} lines before the store failed: {source}"
            ),
        }
    }
}

impl std::error::Error for IngestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::Redaction(e) => Some(e),
            Self::Key(e) => Some(e),
            Self::Admin(e) => Some(e),
            Self::Store(e) | Self::Partial { source: e, .. } => Some(e),
            Self::NotUtf8(_) | Self::Empty(_) => None,
        }
    }
}

/// Ingests the transcript at `path` into a new ingest seat of the node
/// in `home`, starting the node first when it has never started, each
/// line redacted by `redactor` and the harness account identifiers the
/// transcript names.
///
/// # Errors
///
/// Fails when the transcript cannot be read, is not UTF-8 or holds no
/// line, or when an identifier cannot be redacted, or the random
/// source, the home, the node or the store fails.
pub fn ingest_path(
    home: &Home,
    path: &Path,
    redactor: &Redactor,
    entropy: &mut dyn Entropy,
) -> Result<Ingested, IngestError> {
    let bytes = fs::read(path).map_err(|source| IngestError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let text = String::from_utf8(bytes).map_err(|_| IngestError::NotUtf8(path.to_path_buf()))?;
    let lines = lines(&text);
    if lines.is_empty() {
        return Err(IngestError::Empty(path.to_path_buf()));
    }
    let mut ids = HarnessAccountIdentifiers::default();
    lines.iter().for_each(|l| ids.learn(l));
    let redactor = redactor
        .with_identifiers(&ids)
        .map_err(IngestError::Redaction)?;
    let mut new_events = Vec::with_capacity(lines.len());
    let mut redactions = Vec::new();
    for (n, line) in (1u64..).zip(&lines) {
        let redacted = redactor.redact_line(line);
        new_events.push(NewEvent {
            kind: EventKind::UnparsedLine,
            provenance: Provenance::Unparsed,
            content: content(n, &redacted.text, redacted.rules.len()),
            commitment_key: CommitmentKey::from_bytes(draw(entropy).map_err(IngestError::Key)?),
        });
        redactions.extend(redacted.rules);
    }
    home.create().map_err(IngestError::Admin)?;
    let mut store = Store::open(&home.store()).map_err(IngestError::Store)?;
    let node = Node::open(home, &mut store, entropy).map_err(IngestError::Admin)?;
    let seat = node
        .certify_ingest_seat(home, &mut store, entropy)
        .map_err(IngestError::Admin)?;
    // A transaction at a time, so a failure can say what landed.
    let mut landed = 0;
    for chunk in new_events.chunks(store::BATCH) {
        store
            .append(seat.writer, chunk)
            .map_err(|source| IngestError::Partial {
                writer: seat.writer,
                landed,
                lines: new_events.len(),
                source,
            })?;
        landed += chunk.len();
    }

    Ok(Ingested {
        writer: seat.writer,
        seat: seat.seat,
        room: seat.room,
        events: new_events.len(),
        redactions,
        redacted_path: redactor.redact_text(&path.to_string_lossy()).text,
    })
}

/// An unparsed line's content: its number and redacted text, and how
/// many redactions the text holds when it holds any.
fn content(line: u64, text: &str, redactions: usize) -> Value {
    let mut content = json!({"line": line, "text": text});
    if redactions > 0 {
        content["redactions"] = json!(redactions);
    }

    content
}

/// A transcript's lines: its text split at each newline, without the
/// empty piece after a final one.
#[must_use]
pub fn lines(text: &str) -> Vec<&str> {
    let mut out: Vec<&str> = text.split('\n').collect();
    if out.last() == Some(&"") {
        out.pop();
    }

    out
}

#[cfg(test)]
mod tests;
