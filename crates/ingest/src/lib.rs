//! `cairn ingest` (REC-01's first step): reads one transcript into the
//! record. Each `cairn ingest` starts an ingest seat in the personal room,
//! certified by the node's device key, and appends every line of the
//! transcript to that seat's writer as one event, its content the line's
//! number and redacted text. The transcript parser classes each line
//! from its structure (PRV-08): a `transcript_line` of one provenance
//! class, or, for a line it cannot parse, an `unparsed_line` of
//! provenance `unparsed` (REC-05). A line that drifts from its recorded
//! shape is kept whole as `harness_text` (REC-26); each unparsed line
//! and each drift is reported for the caller to audit and count. No
//! hook observation records a harness session yet, so no line is
//! `user`; the one ingest seat stands in for all of the transcript's
//! runs, which phase 8 splits it by.
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
use provenance::{Classified, Classifier, Session};
use record::commitment::CommitmentKey;
use record::event::{EventKind, Provenance};
use record::id::{RoomId, SeatId, WriterId};
use redaction::Redactor;
use redaction::account::HarnessAccountIdentifiers;
use serde_json::{Value, json};
use store::{NewEvent, Store, StoreError};
use transcript::shape::Shapes;

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
    /// How many lines it stored as `unparsed` events (REC-05).
    pub unparsed: usize,
    /// What an audit entry says of each format drift (REC-26), line by
    /// line: plain names only, each passed through the redactor.
    pub drifts: Vec<String>,
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
    /// A directory under a transcript root cannot be listed.
    List {
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
            Self::List { path, source } => {
                write!(f, "list the transcripts in {}: {source}", path.display())
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
            Self::Read { source, .. } | Self::List { source, .. } => Some(source),
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
    let Prepared {
        new_events,
        redactions,
        unparsed,
        drifts,
    } = prepare(&lines, &redactor, entropy)?;
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
        unparsed,
        drifts,
        redacted_path: redactor.redact_text(&path.to_string_lossy()).text,
    })
}

/// A transcript's events, ready to append, with what reading it found.
struct Prepared {
    new_events: Vec<NewEvent>,
    redactions: Vec<String>,
    unparsed: usize,
    drifts: Vec<String>,
}

/// Classes each of `lines` from its structure, in order, then redacts
/// it and draws its commitment key. A line is classed from its own
/// text, so a key name or value redaction would change cannot change
/// its class; what it stores is the redacted text alone.
fn prepare(
    lines: &[&str],
    redactor: &Redactor,
    entropy: &mut dyn Entropy,
) -> Result<Prepared, IngestError> {
    let shapes = Shapes::supported();
    // No hook observation records a harness session's ancestry yet
    // (phase 9), so no line is `user` (PRV-08).
    let mut classifier = Classifier::new(&shapes, Session::default(), redactor);
    let describe = |text: &str| redactor.redact_text(text).text;
    let mut prepared = Prepared {
        new_events: Vec::with_capacity(lines.len()),
        redactions: Vec::new(),
        unparsed: 0,
        drifts: Vec::new(),
    };
    for (n, line) in (1u64..).zip(lines) {
        let (kind, provenance, drift) = match classifier.line(line) {
            Classified::Unparsed(drift) => {
                prepared.unparsed += 1;
                (EventKind::UnparsedLine, Provenance::Unparsed, drift)
            }
            Classified::Line(provenance, drift) => (EventKind::TranscriptLine, provenance, drift),
        };
        prepared.drifts.extend(drift.map(|d| d.describe(&describe)));
        let redacted = redactor.redact_line(line);
        prepared.new_events.push(NewEvent {
            kind,
            provenance,
            content: content(n, &redacted.text, redacted.rules.len()),
            commitment_key: CommitmentKey::from_bytes(draw(entropy).map_err(IngestError::Key)?),
        });
        prepared.redactions.extend(redacted.rules);
    }

    Ok(prepared)
}

/// The transcripts under `root`: every `.jsonl` file in it or a
/// directory below it, in path order, a subagent's among them. A
/// symbolic link to a transcript is listed as the link, for the caller
/// to resolve within the allowed roots (SEC-18); a linked directory,
/// which could loop, is not descended. A root that does not exist holds
/// none.
///
/// # Errors
///
/// Fails when `root` or a directory below it cannot be listed.
pub fn transcripts(root: &Path) -> Result<Vec<PathBuf>, IngestError> {
    let mut found = Vec::new();
    match fs::symlink_metadata(root) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        _ => walk(root, &mut found)?,
    }
    found.sort();

    Ok(found)
}

/// Adds each transcript in `dir` and the directories below it to
/// `found`.
fn walk(dir: &Path, found: &mut Vec<PathBuf>) -> Result<(), IngestError> {
    let failed = |source| IngestError::List {
        path: dir.to_path_buf(),
        source,
    };
    for entry in fs::read_dir(dir).map_err(failed)? {
        let entry = entry.map_err(failed)?;
        let path = entry.path();
        if entry.file_type().map_err(failed)?.is_dir() {
            walk(&path, found)?;
        } else if path.extension().is_some_and(|e| e == "jsonl") {
            found.push(path);
        }
    }

    Ok(())
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
