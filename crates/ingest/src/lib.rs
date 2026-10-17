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

/// The settings an ingest writes under.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Content whose canonical encoding is longer goes to the payload
    /// store (`payload.threshold_bytes`, REC-09).
    pub payload_threshold: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            payload_threshold: store::DEFAULT_PAYLOAD_THRESHOLD,
        }
    }
}

/// Ingests the transcript at `path` as [`ingest_with`] does, under the
/// default [`Limits`].
///
/// # Errors
///
/// As [`ingest_with`].
pub fn ingest_path(
    home: &Home,
    path: &Path,
    redactor: &Redactor,
    entropy: &mut dyn Entropy,
) -> Result<Ingested, IngestError> {
    ingest_with(home, path, redactor, &Limits::default(), entropy)
}

/// Ingests the transcript at `path` into a new ingest seat of the node
/// in `home`, starting the node first when it has never started, each
/// line redacted by `redactor` and the harness account identifiers the
/// transcript names. The seat's writer closes a segment at each turn
/// end and subagent end the transcript records, and at its end
/// (REC-19).
///
/// # Errors
///
/// Fails when the transcript cannot be read, is not UTF-8 or holds no
/// line, or when an identifier cannot be redacted, or the random
/// source, the home, the node or the store fails.
pub fn ingest_with(
    home: &Home,
    path: &Path,
    redactor: &Redactor,
    limits: &Limits,
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
        ends,
        redactions,
        unparsed,
        drifts,
    } = prepare(&lines, &redactor, entropy)?;
    home.create().map_err(IngestError::Admin)?;
    let mut store = Store::open(&home.store())
        .map_err(IngestError::Store)?
        .with_payload_threshold(limits.payload_threshold);
    let node = Node::open(home, &mut store, entropy).map_err(IngestError::Admin)?;
    let seat = node
        .certify_ingest_seat(home, &mut store, entropy)
        .map_err(IngestError::Admin)?;
    append_segments(&mut store, seat.writer, &new_events, &ends)?;

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

/// Appends `new_events` to `writer`, a segment to each run of them that
/// `ends` closes, each `true` closing the segment after its event, and
/// the last segment closed at the end. A transaction at a time, so a
/// failure can say what landed.
fn append_segments(
    store: &mut Store,
    writer: WriterId,
    new_events: &[NewEvent],
    ends: &[bool],
) -> Result<(), IngestError> {
    let mut landed = 0;
    for segment in segments(ends) {
        let chunks: Vec<&[NewEvent]> = new_events[segment].chunks(store::BATCH).collect();
        for (n, chunk) in chunks.iter().enumerate() {
            let appended = if n + 1 == chunks.len() {
                store.append(writer, chunk)
            } else {
                store.append_open(writer, chunk)
            };
            appended.map_err(|source| IngestError::Partial {
                writer,
                landed,
                lines: new_events.len(),
                source,
            })?;
            landed += chunk.len();
        }
    }

    Ok(())
}

/// The ranges of events each segment holds: each closes after an event
/// `ends` marks, and the last at the end.
fn segments(ends: &[bool]) -> Vec<std::ops::Range<usize>> {
    let mut out = Vec::new();
    let mut start = 0;
    for (i, end) in ends.iter().enumerate() {
        if *end || i + 1 == ends.len() {
            out.push(start..i + 1);
            start = i + 1;
        }
    }

    out
}

/// A transcript's events, ready to append, with what reading it found.
struct Prepared {
    new_events: Vec<NewEvent>,
    /// For each event, whether a segment closes after it: a turn end or
    /// a subagent end (REC-19).
    ends: Vec<bool>,
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
        ends: Vec::with_capacity(lines.len()),
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
        // Read from the line's own structure, as its class is; the
        // timestamp only when well formed, so it carries no secret.
        let parsed: Value = serde_json::from_str(line).unwrap_or(Value::Null);
        prepared
            .ends
            .push(turn_end(&parsed) || provenance == Provenance::SubagentResult);
        let redacted = redactor.redact_line(line);
        prepared.new_events.push(NewEvent {
            kind,
            provenance,
            content: content(n, &redacted.text, redacted.rules.len()),
            commitment_key: CommitmentKey::from_bytes(draw(entropy).map_err(IngestError::Key)?),
            transcript_timestamp: parsed["timestamp"]
                .as_str()
                .filter(|t| well_formed(t))
                .map(str::to_owned),
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

/// Whether the parsed transcript line `line` is the model reply that
/// ends a turn: an `assistant` line that stopped for any reason but a
/// tool call or a pause.
fn turn_end(line: &Value) -> bool {
    line["type"] == "assistant"
        && line["message"]["stop_reason"]
            .as_str()
            .is_some_and(|r| !matches!(r, "tool_use" | "pause_turn"))
}

/// Whether `text` is an RFC 3339 date and time, as a transcript
/// timestamp is: `YYYY-MM-DDTHH:MM:SS`, at most nine fraction digits,
/// then `Z` or an offset `+HH:MM`.
fn well_formed(text: &str) -> bool {
    let b = text.as_bytes();
    let digits =
        |r: std::ops::Range<usize>| b.get(r).is_some_and(|d| d.iter().all(u8::is_ascii_digit));
    let at = |i: usize, c: u8| b.get(i) == Some(&c);
    let date_time = digits(0..4)
        && at(4, b'-')
        && digits(5..7)
        && at(7, b'-')
        && digits(8..10)
        && at(10, b'T')
        && digits(11..13)
        && at(13, b':')
        && digits(14..16)
        && at(16, b':')
        && digits(17..19);
    let mut rest = b.get(19..).unwrap_or_default();
    if let Some(fraction) = rest.strip_prefix(b".") {
        let n = fraction.iter().take_while(|d| d.is_ascii_digit()).count();
        if n == 0 || n > 9 {
            return false;
        }
        rest = &fraction[n..];
    }
    let zone = rest == b"Z"
        || (rest.len() == 6
            && matches!(rest[0], b'+' | b'-')
            && rest[1..3].iter().all(u8::is_ascii_digit)
            && rest[3] == b':'
            && rest[4..6].iter().all(u8::is_ascii_digit));

    date_time && zone
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
