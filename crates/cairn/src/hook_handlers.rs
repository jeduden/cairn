//! The hook handlers (§9.1). `cairn hook <hook>` reads the hook input,
//! decodes it (SEC-16, REC-26) and holds what it observed at the
//! transcript's ingest marker (REC-19), within its hook budget (NFR-02).
//! It writes nothing to stderr and exits 0 on every path: a failure, a
//! panic or a handler past its budget among them, is audited and the
//! agent goes on (ENG-04, I9). A home it must refuse is left untouched
//! (I8).
//!
//! - Each handler leaves a start marker it clears when it finishes; one
//!   left past its hook's harness timeout is counted, once, by the next
//!   hook handler, the MCP server as it starts, or `cairn status`.
//! - A `SessionStart` hook observation is the harness session's session
//!   binding; one no MCP server took within 30 s is counted, once, by the
//!   next hook handler, `cairn status` or `cairn ingest`.
//! - The handler's work runs on a thread of its own, holding only what
//!   it owns, so the entry point answers within the hook budget whatever
//!   the work waits on.

use std::fs;
use std::io::{self, Write};
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

use administration::home::Home;
use allowed_root::{Access, AllowedRoots};
use audit::entry::{Counter, Kind, Occurrence};
use audit::log::Log;
use configuration::Sources;
use hook::input::HookInput;
use hook::{Decoded, Failure, Hook};
use key::random::{OsEntropy, draw};
use record::commitment::{CommitmentKey, commit};
use redaction::Redactor;
use serde_json::json;
use store::{Observation, PromptCommitment, Store};

use crate::{Context, EXIT_OK, settings};

/// How long a `SessionStart` hook observation's session binding waits
/// for an MCP server to take it before it counts as a failure (REC-19).
pub(crate) const BINDING_WAIT_SECS: u64 = 30;

/// How long a hook handler the development build's seam hangs.
const HANG: Duration = Duration::from_secs(60);

/// The directory of start markers, in the home.
const MARKERS: &str = "hooks";

/// How the harness may say a harness session started; any other
/// `source` is recorded as `other`, so only these reach a structural
/// field.
const SOURCES: [&str; 5] = ["startup", "resume", "clear", "compact", "fork"];

/// `cairn hook <hook>`: runs the hook handler `name` names on the hook
/// input on stdin, writing at most its output to `stdout`.
pub(crate) fn hook_command(name: Option<&str>, ctx: &Context, stdout: &mut dyn Write) -> u8 {
    // A panic anywhere, the clock's or an audit's among them, stops
    // here (ENG-04).
    let _ = panic::catch_unwind(AssertUnwindSafe(|| run(name, ctx, stdout)));

    EXIT_OK
}

/// What `cairn hook` does, short of catching a panic.
#[expect(
    clippy::disallowed_methods,
    reason = "the hook budget is wall-clock time from the handler's start (NFR-02); \
              no derived artifact reads it (I10)"
)]
fn run(name: Option<&str>, ctx: &Context, stdout: &mut dyn Write) {
    let started = Instant::now();
    let Some(hook) = name.and_then(Hook::parse) else {
        audit_failure(ctx, "hook", Failure::UnknownHook);
        return;
    };
    let budget = ctx.hook_budget.unwrap_or_else(|| hook.budget());
    let deadline = started + budget;
    let home = ctx.home().ok().filter(|h| h.create().is_ok());
    let now = (ctx.clock)();
    let marker = home
        .as_ref()
        .and_then(|h| StartMarker::leave(h, hook, now).ok());
    if let Some(h) = &home {
        count_unfinished(h, now, marker.as_ref().map(|m| m.path.as_path()));
    }
    let wait = deadline.min(started + crate::HOOK_INPUT_WAIT);
    let read = match crate::hook_input(ctx, wait) {
        // An input still unread as the hook budget ends is the budget's.
        Err(Failure::Malformed) if now_instant() >= deadline => {
            return budget_exceeded(ctx, hook, marker);
        }
        read => read,
    };
    let answered = read.and_then(|text| {
        let job = Job {
            hook,
            text,
            home: home.clone().map(|h| {
                let sources = settings::sources(ctx, &h);
                (h, sources)
            }),
            os_home: ctx.home.as_deref().map(PathBuf::from),
            now,
            attended: ctx.attended,
            ancestry: ctx.ancestry,
            panics: ctx.hook_panics,
            hangs: ctx.hook_hangs,
        };
        job.within(deadline)
    });
    match answered {
        Ok(Some(out)) => {
            let _ = stdout.write_all(out.as_bytes());
        }
        Ok(None) => return budget_exceeded(ctx, hook, marker),
        Err(failure) => audit_failure(ctx, hook.operation(), failure),
    }
    if let Some(marker) = marker {
        marker.clear();
    }
}

/// Audits a `hook` handler that ran past its hook budget, answering
/// empty, and clears its start marker.
fn budget_exceeded(ctx: &Context, hook: Hook, marker: Option<StartMarker>) {
    let occurrence = Occurrence::Degraded {
        kind: Kind::TimedOut,
        counter: Counter::HookBudgetExceeded,
        operation: hook.operation(),
        detail: "hook handler ran past its hook budget and answered empty",
    };
    audit(ctx, &occurrence);
    if let Some(marker) = marker {
        marker.clear();
    }
}

/// Audits `failure` under `operation`. Nowhere to report an audit log
/// that takes no entry: the hook writes nothing to stderr. `cairn
/// status` and `cairn doctor` name what keeps the home from taking it.
fn audit_failure(ctx: &Context, operation: &'static str, failure: Failure) {
    audit(ctx, &failure.occurrence(operation));
}

/// Appends `occurrence` to the audit log, a panic while doing so caught.
fn audit(ctx: &Context, occurrence: &Occurrence) {
    let _ = panic::catch_unwind(AssertUnwindSafe(|| ctx.audit(occurrence)));
}

/// Everything a hook handler's work needs, owned, so it runs on a
/// thread of its own.
struct Job {
    hook: Hook,
    text: String,
    /// The checked home, with where its settings layers are; none for a
    /// home the core must refuse.
    home: Option<(Home, Sources)>,
    /// The OS user's home directory, which `~` in `transcript.roots`
    /// names.
    os_home: Option<PathBuf>,
    now: u64,
    /// Whether the attended mark is in the handler's environment.
    attended: bool,
    /// Reads the handler's process ancestry.
    ancestry: fn() -> Option<hook::ancestry::Ancestry>,
    /// A development build's seam: the handler panics.
    panics: bool,
    /// A development build's seam: the handler hangs.
    hangs: bool,
}

impl Job {
    /// Runs the job on a thread of its own: its answer by `deadline`,
    /// or none once the deadline passes first.
    fn within(self, deadline: Instant) -> Result<Option<String>, Failure> {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(hook::guarded(|| self.run()));
        });
        match rx.recv_timeout(deadline.saturating_duration_since(now_instant())) {
            Ok(answer) => answer.map(Some),
            Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => Ok(None),
        }
    }

    /// Decodes the hook input and records what it observed; its answer
    /// is empty, as no handler injects yet.
    fn run(self) -> Result<String, Failure> {
        if self.panics {
            crate::seam_panic();
        }
        if self.hangs {
            // Longer than any harness timeout: the harness kills it first.
            std::thread::sleep(HANG);
        }
        let decoded = hook::decode(self.hook, &self.text)?;
        if let Some((home, sources)) = &self.home {
            Recording::new(&self, home, sources).record(&decoded);
        }

        Ok(String::new())
    }
}

/// Now, for the hook budget's deadline.
#[expect(
    clippy::disallowed_methods,
    reason = "the hook budget is wall-clock time (NFR-02); no derived artifact reads it (I10)"
)]
fn now_instant() -> Instant {
    Instant::now()
}

/// One hook handler's recording, in a checked home.
struct Recording<'j> {
    job: &'j Job,
    home: &'j Home,
    sources: &'j Sources,
    log: Log,
}

impl<'j> Recording<'j> {
    fn new(job: &'j Job, home: &'j Home, sources: &'j Sources) -> Self {
        Self {
            job,
            home,
            sources,
            log: Log::new(home.audit()),
        }
    }

    fn audit(&self, occurrence: &Occurrence) {
        let _ = self.log.append(occurrence, self.job.now);
    }

    fn failed(&self, detail: &'static str) {
        self.audit(&Occurrence::Degraded {
            kind: Kind::Failed,
            counter: Counter::HookFailed,
            operation: self.job.hook.operation(),
            detail,
        });
    }

    /// Records what `decoded` observed: its format drift audited, every
    /// session binding left untaken counted, then the hook's own
    /// observation held at its transcript's ingest marker.
    fn record(&self, decoded: &Decoded) {
        let mut store = match Store::open(&self.home.store()) {
            Ok(store) => store,
            Err(_) => return self.failed("store could not be opened"),
        };
        let acts = settings::acts_in(self.home, &store).unwrap_or_default();
        let in_force = settings::in_force_from(self.sources, &acts);
        let settings = &in_force.resolved.settings;
        let patterns = settings.texts("redaction.extra_patterns");
        // The rule set always compiles (the redaction crate's tests prove
        // it); were it not to, nothing could be written.
        let Ok((redactor, _)) = Redactor::new(patterns) else {
            return self.failed("redaction rules could not be built");
        };
        let redact = |t: &str| redactor.redact_text(t).text;
        if let Some(drift) = &decoded.drift {
            let described = drift.describe(&redact);
            self.audit(&Occurrence::Keyed {
                kind: Kind::Rejected,
                counter: Counter::FormatDrift,
                operation: self.job.hook.operation(),
                detail: "format drift",
                key: &described,
            });
        }
        count_untaken(&mut store, self.job.now, &|o| self.audit(o));
        let input = &decoded.input;
        let (Some(session), Some(path)) = (&input.session_id, &input.transcript_path) else {
            return;
        };
        let roots = settings.texts("transcript.roots");
        let observation = Observation {
            hook: self.job.hook.name().to_owned(),
            harness_session: session.clone(),
            transcript_path: redact(path),
            cwd: redact(input.cwd.as_deref().unwrap_or_default()),
            offset: self.offset(Path::new(path), roots),
            source: input.source.as_deref().map(source_name),
            prompt_id: input.prompt_id.clone(),
            attended: self.job.attended,
            ancestry: (self.job.hook == Hook::SessionStart)
                .then(self.job.ancestry)
                .flatten()
                .map(|a| (a.harnesses as u64, a.nested)),
            at: self.job.now,
        };
        let usable = !in_force.loaded.principal_layer_failed();
        let commitment = match (self.job.hook, usable) {
            (Hook::UserPromptSubmit, true) => committed(input, &redactor),
            (Hook::UserPromptSubmit, false) if input.prompt_id.is_some() => {
                self.audit(&Occurrence::Degraded {
                    kind: Kind::Dropped,
                    counter: Counter::PromptCommitmentDropped,
                    operation: self.job.hook.operation(),
                    detail: "prompt commitment dropped while a settings layer fails",
                });
                None
            }
            _ => None,
        };
        if self.job.hook == Hook::PermissionRequest {
            return;
        }
        if store
            .hold_observation(&observation, commitment.as_ref())
            .is_err()
        {
            self.failed("hook observation could not be held");
        }
    }

    /// The size of the transcript at `path` when its hook fired: where
    /// the observation is held. None, audited, when it lies outside the
    /// transcript roots or cannot be read; nothing is read of it.
    fn offset(&self, path: &Path, roots: &[String]) -> Option<u64> {
        let os_home = self.job.os_home.as_deref();
        let allowed = roots.iter().fold(AllowedRoots::default(), |allowed, root| {
            allowed.with(&allowed_root::expand_home(root, os_home), Access::Read)
        });
        let Ok(resolved) = allowed.resolve(path, Access::Read) else {
            self.audit(&Occurrence::Degraded {
                kind: Kind::Rejected,
                counter: Counter::PathOutsideAllowedRoots,
                operation: self.job.hook.operation(),
                detail: "path outside allowed roots",
            });
            return None;
        };
        self.size(fs::metadata(resolved.path()).map(|meta| meta.len()))
    }

    /// The transcript size `read` found: none, audited, when it could
    /// not be read; 0 for one not written yet.
    fn size(&self, read: io::Result<u64>) -> Option<u64> {
        match read {
            Ok(len) => Some(len),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Some(0),
            Err(_) => {
                self.failed("transcript path could not be read");
                None
            }
        }
    }
}

/// The prompt commitment of `input`'s prompt, redacted by `redactor`,
/// under its own random 256-bit key; none without a `prompt_id`, a
/// prompt or the key's randomness (PRV-08).
fn committed(input: &HookInput, redactor: &Redactor) -> Option<PromptCommitment> {
    let (Some(session), Some(prompt_id), Some(prompt)) =
        (&input.session_id, &input.prompt_id, &input.prompt)
    else {
        return None;
    };
    let key = CommitmentKey::from_bytes(draw(&mut OsEntropy).ok()?);
    let commitment = commit(&key, &json!(redactor.redact_text(prompt).text)).ok()?;

    Some(PromptCommitment {
        harness_session: session.clone(),
        prompt_id: prompt_id.clone(),
        key,
        commitment,
        used: false,
    })
}

/// `source` as a structural field: one of [`SOURCES`], else `other`.
fn source_name(source: &str) -> String {
    if SOURCES.contains(&source) {
        source.to_owned()
    } else {
        "other".to_owned()
    }
}

/// Counts, once each, every `SessionStart` hook observation in `store`
/// whose session binding no MCP server took within 30 s of `now`,
/// auditing each through `audit` (REC-19, I6).
pub(crate) fn count_untaken(store: &mut Store, now: u64, audit: &dyn Fn(&Occurrence)) {
    let Ok(untaken) = store.untaken_bindings(now.saturating_sub(BINDING_WAIT_SECS)) else {
        return;
    };
    for held in untaken {
        if store.count_binding(held.id).unwrap_or(false) {
            audit(&Occurrence::Degraded {
                kind: Kind::Failed,
                counter: Counter::SessionBindingNotTaken,
                operation: "hook SessionStart",
                detail: "no MCP server took the session binding within 30 s",
            });
        }
    }
}

/// A start marker (NFR-02): a file in the home's `hooks` directory,
/// named for its hook and a random nonce, holding when its handler
/// started.
#[derive(Debug)]
pub(crate) struct StartMarker {
    path: PathBuf,
}

impl StartMarker {
    /// Leaves the start marker of a `hook` handler started at `now`.
    fn leave(home: &Home, hook: Hook, now: u64) -> io::Result<Self> {
        let dir = home.root().join(MARKERS);
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
        builder.create(&dir)?;
        let nonce: [u8; 8] = draw(&mut OsEntropy).map_err(io::Error::other)?;
        let name: String = nonce.iter().map(|b| format!("{b:02x}")).collect();
        let path = dir.join(format!("{}.{name}", hook.name()));
        // Written whole under a name readers skip, then renamed, so no
        // reader finds a marker without its time.
        let staged = dir.join(format!(".{}.{name}", hook.name()));
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        options
            .open(&staged)?
            .write_all(now.to_string().as_bytes())?;
        fs::rename(&staged, &path)?;

        Ok(Self { path })
    }

    /// Clears it, as its handler finishes.
    fn clear(self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Counts and audits, once each, every start marker in `home` but
/// `own` left past its hook's harness timeout at `now` (NFR-02): the
/// reader that removes one counts it, so no two readers count one.
pub(crate) fn count_unfinished(home: &Home, now: u64, own: Option<&Path>) -> usize {
    let Ok(listing) = fs::read_dir(home.root().join(MARKERS)) else {
        return 0;
    };
    let log = Log::new(home.audit());
    let mut counted = 0;
    for entry in listing.flatten() {
        let path = entry.path();
        let staged = entry.file_name().to_string_lossy().starts_with('.');
        if staged || Some(path.as_path()) == own || !left_behind(&path, now) {
            continue;
        }
        if fs::remove_file(&path).is_ok() {
            counted += 1;
            let occurrence = Occurrence::Degraded {
                kind: Kind::Failed,
                counter: Counter::HookUnfinished,
                operation: "hook",
                detail: "a hook handler the harness stopped left its start marker",
            };
            let _ = log.append(&occurrence, now);
        }
    }

    counted
}

/// Whether the start marker at `path` was left by a handler past its
/// hook's harness timeout at `now`: one whose name or time does not
/// parse counts as left.
fn left_behind(path: &Path, now: u64) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    let hook = name.split('.').next().and_then(Hook::parse);
    let started = fs::read_to_string(path)
        .ok()
        .and_then(|t| t.trim().parse::<u64>().ok());
    match hook.zip(started) {
        Some((hook, started)) => now.saturating_sub(started) > hook.harness_timeout().as_secs(),
        None => true,
    }
}

/// The hook budget the development build's seam sets, in milliseconds.
#[must_use]
pub fn budget_override(millis: Option<&str>) -> Option<Duration> {
    millis
        .and_then(|m| m.parse::<u64>().ok())
        .map(Duration::from_millis)
}

#[cfg(test)]
pub(crate) mod tests;
