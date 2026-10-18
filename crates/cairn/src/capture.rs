//! The MCP server's live ingest (REC-13), on a thread of its own beside
//! the requests it serves.
//!
//! - It takes the session bindings the `SessionStart` hook handlers of
//!   its own harness process recorded, and ingests nothing as witnessed
//!   before. Each transcript path resolves within the transcript roots
//!   (SEC-18); a subagent's transcript under the main one's
//!   `subagents` directory resolves the same way as it appears.
//! - Every [`POLL`] it ingests what each transcript gained into its
//!   runs' run seats, whose keys it holds in memory alone (SEC-10), and
//!   reads the tool-use ids of the lines ingest read, by run, for the
//!   server to find each tool call's run by (PRV-14).
//! - When it stops, it audits each transcript it left short of its end
//!   under `ingest_remainder_left`, closes its run seats' open segments
//!   (REC-19) and removes its server lock. A server killed first leaves
//!   the lock, which the next MCP server or `cairn ingest` finds, and
//!   audits the remainder for it ([`count_stopped`]).
//!
//! A server lock is a file in the home's `mcp` directory naming the
//! transcripts its server ingests, locked by the server while it runs:
//! one whose lock a reader can take belongs to a server that stopped
//! without removing it.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use administration::home::Home;
use allowed_root::{Access, AllowedRoots};
use audit::entry::{Counter, Kind, Occurrence};
use audit::log::Log;
use ingest::live::{RunSeats, ingest_live};
use ingest::run::{Defaults, RunKey};
use ingest::{IngestError, Ingested};
use key::random::{OsEntropy, draw};
use redaction::Redactor;
use serde_json::Value;
use store::Store;

/// How often live ingest reads its transcripts again.
pub(crate) const POLL: Duration = Duration::from_millis(100);

/// How often a call waiting for its tool-use id looks again.
const LOOK: Duration = Duration::from_millis(10);

/// The directory of server locks, in the home.
const LOCKS: &str = "mcp";

/// What live ingest shares with the server: the runs of each tool-use
/// id its ingest read, whether it holds a session binding, and whether
/// it should stop.
#[derive(Debug, Default)]
pub(crate) struct Shared {
    tool_uses: Mutex<BTreeMap<String, BTreeSet<String>>>,
    serving: AtomicBool,
    stop: AtomicBool,
}

impl Shared {
    /// Tells live ingest to stop.
    pub(crate) fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }

    fn stopping(&self) -> bool {
        self.stop.load(Ordering::SeqCst)
    }

    fn runs_of(&self, tool_use_id: &str) -> Vec<String> {
        self.tool_uses
            .lock()
            .map(|t| t.get(tool_use_id).into_iter().flatten().cloned().collect())
            .unwrap_or_default()
    }

    fn read(&self, tool_use_id: String, run: String) {
        if let Ok(mut t) = self.tool_uses.lock() {
            t.entry(tool_use_id).or_default().insert(run);
        }
    }
}

/// Finds each tool call's run through what live ingest read (PRV-14).
#[derive(Debug, Clone)]
pub(crate) struct Finder(pub(crate) Arc<Shared>);

impl mcp::tool_call::Runs for Finder {
    fn serving(&self) -> bool {
        self.0.serving.load(Ordering::SeqCst)
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "a call waits for ingest at most its bound in wall-clock time (PRV-14); \
                  no derived artifact reads it (I10)"
    )]
    fn find(&self, tool_use_id: &str, wait: Duration) -> (Vec<String>, Duration) {
        let started = Instant::now();
        loop {
            let runs = self.0.runs_of(tool_use_id);
            let waited = started.elapsed();
            if !runs.is_empty() || waited >= wait {
                return (runs, waited);
            }
            std::thread::sleep(LOOK);
        }
    }
}

/// The name of the run `key` names, as the log and a tool call's
/// finding give it: its harness session, and the harness's agent id for
/// a subagent's run.
pub(crate) fn run_name(key: &RunKey) -> String {
    match &key.agent_id {
        Some(agent) => format!("{}/{agent}", key.harness_session),
        None => key.harness_session.clone(),
    }
}

/// One bound harness session: its main run's transcript, resolved.
#[derive(Debug, Clone)]
struct Bound {
    main: PathBuf,
}

impl Bound {
    /// The directory a subagent's transcript of the session lies in.
    fn subagents(&self) -> PathBuf {
        let stem = self.main.file_stem().unwrap_or_default();
        self.main.with_file_name(stem).join("subagents")
    }
}

/// What live ingest needs, owned, so it runs on a thread of its own.
pub(crate) struct Capture {
    pub(crate) home: Home,
    pub(crate) roots: AllowedRoots,
    pub(crate) redactor: Redactor,
    pub(crate) inputs: ingest::Inputs,
    /// The server's own harness process, whose session bindings it takes.
    pub(crate) harness: Option<u32>,
    /// A development build's seam: ingest no further than this many
    /// lines of a transcript.
    pub(crate) limit: Option<usize>,
    /// The wall clock, in seconds since the Unix epoch, for audit
    /// entries.
    pub(crate) clock: Box<dyn Fn() -> u64 + Send>,
    pub(crate) shared: Arc<Shared>,
}

/// Live ingest as it runs.
pub(crate) struct Running {
    capture: Capture,
    seats: RunSeats,
    sessions: Vec<Bound>,
    /// Each transcript read, by its resolved path, and how far its
    /// tool-use ids are read.
    transcripts: BTreeMap<PathBuf, u64>,
    /// The paths refused, each audited once.
    refused: BTreeSet<PathBuf>,
    lock: Option<ServerLock>,
}

impl Capture {
    /// Starts live ingest: takes the session bindings already recorded
    /// and reads what their transcripts hold, before the server answers
    /// its first call.
    pub(crate) fn start(self) -> Running {
        let mut running = Running {
            capture: self,
            seats: RunSeats::default(),
            sessions: Vec::new(),
            transcripts: BTreeMap::new(),
            refused: BTreeSet::new(),
            lock: None,
        };
        running.bind();
        running.poll();

        running
    }
}

impl Running {
    /// Runs live ingest until told to stop, then audits what it left,
    /// closes its run seats' segments and removes its server lock.
    pub(crate) fn run(mut self) {
        while !self.capture.shared.stopping() {
            std::thread::sleep(POLL);
            self.bind();
            self.poll();
        }
        self.stop();
    }
}

impl Running {
    fn audit(&self, occurrence: &Occurrence) {
        let _ = Log::new(self.capture.home.audit()).append(occurrence, (self.capture.clock)());
    }

    /// Takes every session binding of its harness process no MCP server
    /// took, each transcript path resolved within the transcript roots.
    fn bind(&mut self) {
        let Some(harness) = self.capture.harness else {
            return;
        };
        let Ok(taken) =
            Store::open(&self.capture.home.store()).and_then(|mut s| s.take_bindings(harness))
        else {
            return;
        };
        for held in taken {
            self.capture.shared.serving.store(true, Ordering::SeqCst);
            let path = PathBuf::from(&held.observation.transcript_path);
            if let Some(main) = self.resolve(&path) {
                self.sessions.push(Bound { main });
            }
        }
    }

    /// `path` resolved within the transcript roots; none, audited once,
    /// when it lies outside them (SEC-18).
    fn resolve(&mut self, path: &Path) -> Option<PathBuf> {
        match self.capture.roots.resolve(path, Access::Read) {
            Ok(allowed) => Some(allowed.path().to_path_buf()),
            Err(_) => {
                self.refuse(path);
                None
            }
        }
    }

    /// Audits, once, a transcript path refused: outside the transcript
    /// roots, or a link swapped in after its check.
    fn refuse(&mut self, path: &Path) {
        if self.refused.insert(path.to_path_buf()) {
            self.audit(&Occurrence::Degraded {
                kind: Kind::Rejected,
                counter: Counter::PathOutsideAllowedRoots,
                operation: "mcp",
                detail: "path outside allowed roots",
            });
        }
    }

    /// The transcripts of the bound harness sessions: each main run's,
    /// and each subagent's that has appeared, resolved.
    fn transcripts(&mut self) -> Vec<PathBuf> {
        let mut found = Vec::new();
        for bound in self.sessions.clone() {
            found.push(bound.main.clone());
            let Ok(listing) = fs::read_dir(bound.subagents()) else {
                continue;
            };
            let mut subagents: Vec<PathBuf> = listing
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|e| e == "jsonl"))
                .filter(|p| !self.refused.contains(p))
                .collect();
            subagents.sort();
            for path in subagents {
                if self.transcripts.contains_key(&path) {
                    found.push(path);
                } else if let Some(resolved) = self.resolve(&path) {
                    found.push(resolved);
                }
            }
        }

        found
    }

    /// Ingests what each transcript gained, and reads the tool-use ids
    /// of the lines it read.
    fn poll(&mut self) {
        let transcripts = self.transcripts();
        let new = transcripts
            .iter()
            .any(|t| !self.transcripts.contains_key(t));
        for path in &transcripts {
            self.transcripts.entry(path.clone()).or_insert(0);
        }
        if new {
            self.lock = ServerLock::hold(&self.capture.home, &transcripts, self.lock.take());
        }
        for path in transcripts {
            let read = ingest_live(
                &self.capture.home,
                &path,
                &self.capture.redactor,
                &self.capture.inputs,
                &mut self.seats,
                self.capture.limit,
                &mut OsEntropy,
            );
            match read {
                Ok(Some(ingested)) => self.audit_ingested(&ingested),
                Ok(None) => {}
                Err(e) => self.failed(&path, &e),
            }
            self.read_tool_uses(&path);
        }
    }

    /// Audits an ingest that failed: a link swapped in for a transcript
    /// is refused; one another ingest got ahead of, or not written yet,
    /// is read again next time.
    fn failed(&mut self, path: &Path, e: &IngestError) {
        match e {
            e if e.position_moved() => {}
            IngestError::Read { source, .. } if source.kind() == io::ErrorKind::NotFound => {}
            IngestError::Read { source, .. } if source.kind() == io::ErrorKind::InvalidInput => {
                self.transcripts.remove(path);
                self.sessions.retain(|b| b.main != path);
                self.refuse(path);
            }
            _ => self.audit(&Occurrence::Degraded {
                kind: Kind::Failed,
                counter: Counter::LiveIngestFailed,
                operation: "mcp",
                detail: "a transcript could not be ingested; the next read tries again",
            }),
        }
    }

    /// Audits what one live ingest found: each redaction, unparsed line,
    /// format drift, new generation and witnessed repeat (I6).
    fn audit_ingested(&self, ingested: &Ingested) {
        for rule in &ingested.redactions {
            self.audit(&keyed(
                Kind::Redacted,
                Counter::Redacted,
                "redacted by rule",
                rule,
            ));
        }
        for _ in 0..ingested.unparsed {
            self.audit(&Occurrence::Degraded {
                kind: Kind::Failed,
                counter: Counter::EventsUnparsed,
                operation: "mcp",
                detail: "stored a transcript line it cannot parse as an unparsed event",
            });
        }
        for drift in &ingested.drifts {
            self.audit(&keyed(
                Kind::Rejected,
                Counter::FormatDrift,
                "format drift",
                drift,
            ));
        }
        if ingested.new_generation {
            self.audit(&keyed(
                Kind::Truncated,
                Counter::NewTranscriptGeneration,
                "new transcript generation of",
                &ingested.name,
            ));
        }
        for _ in 0..ingested.witnessed_repeats {
            self.audit(&keyed(
                Kind::Coalesced,
                Counter::WitnessedRepeatRecorded,
                "witnessed line repeating another transcript recorded as its own event in",
                &ingested.name,
            ));
        }
    }

    /// Reads the tool-use ids of the lines of the transcript at `path`
    /// that ingest read since the last time, each with its run.
    fn read_tool_uses(&mut self, path: &Path) {
        let Some(from) = self.transcripts.get(path).copied() else {
            return;
        };
        let read = Store::open(&self.capture.home.store())
            .and_then(|s| s.source(&path.to_string_lossy()))
            .ok()
            .flatten()
            .map_or(0, |s| s.position.offset);
        if read <= from {
            return;
        }
        let Ok(bytes) = ingest::read::transcript(path) else {
            return;
        };
        let range = bytes
            .get(as_index(from)..as_index(read))
            .unwrap_or_default();
        let defaults = Defaults::of(path);
        for line in String::from_utf8_lossy(range).lines() {
            let parsed: Value = serde_json::from_str(line).unwrap_or(Value::Null);
            let run = run_name(&defaults.key(&parsed));
            for id in tool_use_ids(&parsed) {
                self.capture.shared.read(id, run.clone());
            }
        }
        self.transcripts.insert(path.to_path_buf(), read);
    }

    /// Audits each transcript left short of its end, closes the run
    /// seats' open segments, and removes the server lock.
    fn stop(self) {
        let home = &self.capture.home;
        for name in remainders(home, self.transcripts.keys()) {
            self.audit(&remainder(&name, "mcp"));
        }
        if let Ok(mut store) = Store::open(&home.store()) {
            for (_, writer) in self.seats.runs() {
                let _ = store.close_segment(writer);
            }
        }
        if let Some(lock) = self.lock {
            lock.remove();
        }
    }
}

/// An entry under `counter` naming `key`, of the MCP server.
fn keyed<'a>(kind: Kind, counter: Counter, detail: &'static str, key: &'a str) -> Occurrence<'a> {
    Occurrence::Keyed {
        kind,
        counter,
        operation: "mcp",
        detail,
        key,
    }
}

/// The entry auditing the remainder an MCP server left of the
/// transcript `name`, by the reader `operation` names.
fn remainder<'a>(name: &'a str, operation: &'static str) -> Occurrence<'a> {
    Occurrence::Keyed {
        kind: Kind::Truncated,
        counter: Counter::IngestRemainderLeft,
        operation,
        detail: "an MCP server stopped short of a transcript's end, at its ingest marker:",
        key: name,
    }
}

/// The name of each of `paths` whose ingest position stands short of
/// its complete lines: what an MCP server left of it.
fn remainders<'p>(home: &Home, paths: impl Iterator<Item = &'p PathBuf>) -> Vec<String> {
    let Ok(store) = Store::open(&home.store()) else {
        return Vec::new();
    };
    paths
        .filter(|path| {
            let read = store
                .source(&path.to_string_lossy())
                .ok()
                .flatten()
                .map_or(0, |s| s.position.offset);
            let complete = ingest::read::transcript(path)
                .map(|b| b.iter().rposition(|c| *c == b'\n').map_or(0, |i| i + 1))
                .unwrap_or(0);
            read < complete as u64
        })
        .map(|path| {
            path.file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default()
        })
        .collect()
}

/// The tool-use ids a parsed transcript line carries: each `tool_use`
/// block of an `assistant` line's message.
fn tool_use_ids(line: &Value) -> Vec<String> {
    if line["type"] != "assistant" {
        return Vec::new();
    }
    line["message"]["content"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|block| block["type"] == "tool_use")
        .filter_map(|block| block["id"].as_str().map(str::to_owned))
        .collect()
}

fn as_index(offset: u64) -> usize {
    usize::try_from(offset).unwrap_or(usize::MAX)
}

/// A server lock: a file naming the transcripts its MCP server
/// ingests, locked while the server runs.
#[derive(Debug)]
struct ServerLock {
    path: PathBuf,
    _file: fs::File,
}

impl ServerLock {
    /// A server lock in `home` naming `transcripts`, written anew in
    /// place of `old`; none, with `old` kept, when it cannot be.
    fn hold(home: &Home, transcripts: &[PathBuf], old: Option<Self>) -> Option<Self> {
        let path = match &old {
            Some(lock) => lock.path.clone(),
            None => new_lock_path(home).ok()?,
        };
        let staged = path.with_extension("staged");
        let written = write_owner_only(&staged, transcripts).and_then(|()| {
            let file = fs::OpenOptions::new().read(true).open(&staged)?;
            file.try_lock().map_err(io::Error::other)?;
            fs::rename(&staged, &path)?;
            Ok(file)
        });
        match written {
            Ok(file) => Some(Self { path, _file: file }),
            Err(_) => {
                let _ = fs::remove_file(&staged);
                old
            }
        }
    }

    fn remove(self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// A path for a new server lock in `home`, its directory made.
fn new_lock_path(home: &Home) -> io::Result<PathBuf> {
    let dir = home.root().join(LOCKS);
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(&dir)?;
    let nonce: [u8; 8] = draw(&mut OsEntropy).map_err(io::Error::other)?;
    let name: String = nonce.iter().map(|b| format!("{b:02x}")).collect();

    Ok(dir.join(format!("{name}.lock")))
}

/// Writes each of `transcripts` on its own line to a new file at `path`
/// only its owner can read.
fn write_owner_only(path: &Path, transcripts: &[PathBuf]) -> io::Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = options.open(path)?;
    for t in transcripts {
        writeln!(file, "{}", t.display())?;
    }
    file.sync_all()
}

/// Audits through `audit`, under `operation`, the remainder each MCP
/// server that stopped without removing its server lock left of its
/// transcripts, and removes the lock; one whose server still runs is
/// left (REC-13, I6). Returns how many remainders it audited.
pub(crate) fn count_stopped(
    home: &Home,
    operation: &'static str,
    audit: &dyn Fn(&Occurrence),
) -> usize {
    let Ok(listing) = fs::read_dir(home.root().join(LOCKS)) else {
        return 0;
    };
    let mut audited = 0;
    for entry in listing.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "lock") {
            continue;
        }
        let Ok(file) = fs::File::open(&path) else {
            continue;
        };
        // A server that still runs holds its lock.
        if file.try_lock().is_err() {
            continue;
        }
        let transcripts: Vec<PathBuf> = fs::read_to_string(&path)
            .unwrap_or_default()
            .lines()
            .map(PathBuf::from)
            .collect();
        for name in remainders(home, transcripts.iter()) {
            audit(&remainder(&name, operation));
            audited += 1;
        }
        let _ = fs::remove_file(&path);
    }

    audited
}

#[cfg(test)]
mod tests;
