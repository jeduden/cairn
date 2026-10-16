//! The Cairn core executable, specified in docs/srs: the hook
//! handlers, the MCP server and the CLI so far.
//!
//! It runs `version`, `ingest --path`, `mcp`, `verify`, `hook`,
//! `status`, `doctor`, and the principal acts `counter ack`,
//! `configuration accept` and `risk-acceptance`. Every other
//! command in the SRS's §9.5 is a pending scenario under features/, and
//! lands with the plan that makes its scenario pass.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use administration::act as act_record;
use administration::home::Home;
use administration::node::Node;
use administration::verify::{VerifyError, verify};
use allowed_root::{Access, AllowedPath, AllowedRoots};
use audit::AuditError;
use audit::counters::{Counts, counts};
use audit::entry::{Counter, Entry, Kind, Occurrence};
use audit::log::Log;
use configuration::source::OsUser;
use configuration::{Loaded, SettingsLayer};
use hook::{Failure, Hook};
use key::random::OsEntropy;
use key::signing::DeviceKey;
use redaction::Redactor;
use serde_json::{Value, json};
use store::Store;

/// The command did what it was asked.
pub const EXIT_OK: u8 = 0;
/// The command failed.
pub const EXIT_FAILURE: u8 = 1;
/// The command line is wrong.
pub const EXIT_USAGE: u8 = 2;
/// The record failed an integrity check.
pub const EXIT_INTEGRITY: u8 = 3;

/// How long `cairn mcp` waits for a line before it waits again.
const IDLE: Duration = Duration::from_secs(1);

/// The longest line the process's input may carry, the hook input's
/// limit (SEC-05); the MCP server's requests are held to it too.
pub const INPUT_LINE_LIMIT: usize = hook::HOOK_INPUT_LIMIT;

/// How long `cairn hook` waits for its whole hook input, however its
/// lines arrive. The hook budgets of NFR-02 replace it with phase 9's
/// handlers.
const HOOK_INPUT_WAIT: Duration = Duration::from_millis(500);

/// What a build stamps into the executable. The release build sets
/// `CAIRN_VERSION` and `CAIRN_COMMIT` (ENG-01); a local build reports
/// version "dev" and no commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Build<'a> {
    pub version: &'a str,
    pub commit: Option<&'a str>,
}

/// The version a build reports when the release pipeline stamps none.
const BUILD_DEV: &str = "dev";

/// This executable's build.
pub const BUILD: Build<'static> = Build {
    version: match option_env!("CAIRN_VERSION") {
        Some(v) => v,
        None => BUILD_DEV,
    },
    commit: option_env!("CAIRN_COMMIT"),
};

const USAGE: &str = "usage: cairn <command> [flags]

commands:
  configuration accept [--json]          accept the configuration (widening)
  counter ack [--json]                   acknowledge the counters (neutral)
  doctor [--json]                        fail while a counter rose since the last acknowledgement
  hook <hook>                            run the hook handler on the hook input on stdin
  ingest --path P [--json]               record the transcript at P
  mcp                                    serve the MCP tools over stdio
  risk-acceptance show [--json]          show the residual risks open on this node
  risk-acceptance record [--json]        accept the open residual risks (widening)
  risk-acceptance withdraw [--json]      withdraw the risk acceptance (cut)
  status [--json]                        show the home, the store, the settings and the counters
  verify [--json]                        check the record and the audit log
  version [--json]                       print the build version
  help                                   print this message
";

/// What an invocation runs in: the build, the home variables and the
/// process's input, each injected so a test can stand in for them.
pub struct Context<'a> {
    pub build: &'a Build<'a>,
    /// `CAIRN_HOME`, as the process found it.
    pub cairn_home: Option<OsString>,
    /// `HOME`, as the process found it.
    pub home: Option<OsString>,
    /// `CAIRN_HOME_ID`, the home id the environment supplies (SEC-03).
    pub home_id: Option<OsString>,
    /// Whether the hook handler panics: a development build's seam for
    /// the `@ENG-04` scenario, never set in a release build.
    pub hook_panics: bool,
    /// Whether the process's standard input and output are both a
    /// terminal, which a verb that writes a principal act needs
    /// (OWN-12): a person's are, an agent's tool call's are not.
    pub terminal: bool,
    /// The working directory, whose repository's configuration is a
    /// settings layer (SEC-11).
    pub working_dir: Option<PathBuf>,
    /// Where managed policy is read from (ADM-04).
    pub managed_policy: PathBuf,
    /// The running OS user, whom managed policy's files must not let
    /// write (ADM-04).
    pub os_user: OsUser,
    /// Starts reading the process's input, one line at a time.
    pub input: &'a dyn Fn() -> Receiver<io::Result<String>>,
    /// The wall clock, in seconds since the Unix epoch, which only
    /// audit entries read.
    pub clock: &'a dyn Fn() -> u64,
}

impl Context<'_> {
    /// The home, checked before anything reads or writes in it: it and
    /// everything in it owner-only (SEC-02), and its home id the one the
    /// environment supplies (SEC-03).
    fn home(&self) -> Result<Home, administration::AdminError> {
        let home = Home::locate(self.cairn_home.as_deref(), self.home.as_deref())?;
        home.check()?;
        home.check_home_id(self.home_id.as_deref())?;

        Ok(home)
    }

    /// Appends `occurrence` to the checked home's audit log, making the
    /// home first where it does not exist yet.
    fn audit(&self, occurrence: &Occurrence) -> Result<Entry, Box<dyn std::error::Error>> {
        let home = self.home()?;
        home.create()?;

        Ok(Log::new(home.audit()).append(occurrence, (self.clock)())?)
    }
}

/// The structured log (ENG-05, OPS-05) for the home the environment
/// names: written to its log file once the home exists and passes its
/// checks (SEC-02, SEC-03), and nowhere before; each field redacted by
/// the maintained redaction rule set.
///
/// # Errors
///
/// Fails when the rule set does not compile, which its unit test rules
/// out.
pub fn structured_log(
    cairn_home: Option<OsString>,
    home: Option<OsString>,
    home_id: Option<OsString>,
    clock: fn() -> u64,
) -> Result<structured_log::StructuredLog, String> {
    let open: structured_log::Open = Box::new(move || {
        let Ok(home) = Home::locate(cairn_home.as_deref(), home.as_deref()) else {
            return Ok(None);
        };
        if !home.root().exists() || home.check().is_err() {
            return Ok(None);
        }
        if home.check_home_id(home_id.as_deref()).is_err() {
            return Ok(None);
        }
        let file = structured_log::open_log_file(&home.root().join("logs"))?;
        Ok(Some(Box::new(file)))
    });
    let (redactor, _) = Redactor::new(&[]).map_err(|e| e.to_string())?;

    Ok(structured_log::StructuredLog::new(
        open,
        redactor,
        Box::new(clock),
    ))
}

/// Audits the log lines a run could not write (I6).
pub fn audit_dropped_log_lines(ctx: &Context, dropped: u64) {
    for _ in 0..dropped {
        let _ = ctx.audit(&Occurrence::Degraded {
            kind: Kind::Dropped,
            counter: Counter::LogLineDropped,
            operation: "log",
            detail: "log line dropped",
        });
    }
}

/// Runs one invocation and returns its exit code. It is `main` without
/// the process: every test drives the command line through here.
pub fn run(
    args: impl IntoIterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let mut args = args.into_iter();
    match args.next().as_deref() {
        None => {
            let _ = stderr.write_all(USAGE.as_bytes());
            EXIT_USAGE
        }
        Some("configuration") => act::configuration(args, ctx, stdout, stderr),
        Some("counter") => act::counter(args, ctx, stdout, stderr),
        Some("doctor") => doctor(args, ctx, stdout, stderr),
        Some("hook") => hook_command(args.next().as_deref(), ctx, stdout),
        Some("ingest") => ingest(args, ctx, stdout, stderr),
        Some("mcp") => mcp(args, ctx, stdout, stderr),
        Some("risk-acceptance") => act::risk_acceptance(args, ctx, stdout, stderr),
        Some("status") => status(args, ctx, stdout, stderr),
        Some("verify") => verify_command(args, ctx, stdout, stderr),
        Some("version") => version(args, ctx.build, stdout, stderr),
        Some("help" | "-h" | "--help") => written(stdout.write_all(USAGE.as_bytes()), stderr),
        Some(other) => {
            let _ = write!(stderr, "cairn: unknown command {other:?}\n\n{USAGE}");
            EXIT_USAGE
        }
    }
}

/// `cairn version [--json]`: the stamped version as a bare line, or as
/// `{"version": ..., "commit": ...}`, the commit left out when the
/// build stamped none.
fn version(
    args: impl Iterator<Item = String>,
    build: &Build,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let Some(as_json) = json_flag("version", args, stderr) else {
        return EXIT_USAGE;
    };
    let line = if as_json {
        version_json(build)
    } else {
        build.version.to_owned()
    };

    written(writeln!(stdout, "{line}"), stderr)
}

/// The JSON object `cairn version --json` prints.
fn version_json(build: &Build) -> String {
    let mut out = format!("{{\"version\":{}", json_string(build.version));
    if let Some(commit) = build.commit {
        out.push_str(&format!(",\"commit\":{}", json_string(commit)));
    }
    out.push('}');

    out
}

/// The flags a command takes, with `--json` among them; anything else
/// is a usage error, reported on `stderr`.
pub(crate) fn json_flag(
    command: &str,
    args: impl Iterator<Item = String>,
    stderr: &mut dyn Write,
) -> Option<bool> {
    let mut as_json = false;
    for arg in args {
        if arg != "--json" {
            let _ = writeln!(stderr, "cairn {command}: unknown flag {arg:?}");
            return None;
        }
        as_json = true;
    }

    Some(as_json)
}

/// Reports a failure of `command` on `stderr`.
pub(crate) fn failed(command: &str, e: &dyn std::fmt::Display, stderr: &mut dyn Write) -> u8 {
    let _ = writeln!(stderr, "cairn {command}: {e}");
    EXIT_FAILURE
}

/// `cairn ingest --path P [--json]`: records the transcript at `P` in a
/// new ingest seat.
fn ingest(
    mut args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let usage = |stderr: &mut dyn Write, why: &str| {
        let _ = writeln!(stderr, "cairn ingest: {why}");
        EXIT_USAGE
    };
    let (mut path, mut as_json) = (None, false);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => as_json = true,
            "--path" => match args.next() {
                Some(p) => path = Some(p),
                None => return usage(stderr, "--path needs a transcript"),
            },
            "--all" => return usage(stderr, "--all is not supported yet; pass --path"),
            other => match other.strip_prefix("--path=") {
                Some(p) => path = Some(p.to_owned()),
                None => return usage(stderr, &format!("unknown flag {other:?}")),
            },
        }
    }
    let Some(path) = path else {
        return usage(stderr, "want --path P");
    };
    let ingested = match ctx.home() {
        Ok(home) => {
            let in_force = audit_settings(ctx, &home, stderr);
            let settings = &in_force.resolved.settings;
            let transcript = allowed_transcript(ctx, settings, Path::new(&path), stderr);
            transcript.and_then(|transcript| {
                let redactor = redactor(ctx, settings, stderr)?;
                ingest::ingest_path(&home, transcript.path(), &redactor, &mut OsEntropy)
                    .map_err(|e| e.to_string())
            })
        }
        Err(e) => Err(e.to_string()),
    };
    let ingested = match ingested {
        Ok(i) => i,
        Err(e) => return failed("ingest", &e, stderr),
    };
    audit_redactions(ctx, &ingested.redactions, stderr);
    tracing::info!(
        transcript = %path,
        writer = %ingested.writer,
        events = ingested.events as u64,
        redactions = ingested.redactions.len() as u64,
        "ingested"
    );
    let line = if as_json {
        json!({
            "writer": ingested.writer.to_string(),
            "seat": ingested.seat.to_string(),
            "room": ingested.room.to_string(),
            "events": ingested.events,
        })
        .to_string()
    } else {
        format!(
            "ingested {} lines into writer {}",
            ingested.events, ingested.writer
        )
    };

    written(writeln!(stdout, "{line}"), stderr)
}

/// The transcript at `path` resolved within the transcript roots, the
/// only allowed roots `cairn ingest` reads from (SEC-18): nothing in
/// the home, its keys and store among it, is a transcript. A path
/// outside them is refused, audited and counted, and nothing is read
/// from it.
fn allowed_transcript(
    ctx: &Context,
    settings: &configuration::resolve::Settings,
    path: &Path,
    stderr: &mut dyn Write,
) -> Result<AllowedPath, String> {
    let os_home = ctx.home.as_deref().map(Path::new);
    let allowed = settings
        .texts("transcript.roots")
        .iter()
        .fold(AllowedRoots::default(), |allowed, root| {
            allowed.with(&allowed_root::expand_home(root, os_home), Access::Read)
        });
    allowed.resolve(path, Access::Read).map_err(|refused| {
        let occurrence = Occurrence::Degraded {
            kind: Kind::Rejected,
            counter: Counter::PathOutsideAllowedRoots,
            operation: "ingest",
            detail: "path outside allowed roots",
        };
        if let Err(e) = ctx.audit(&occurrence) {
            let _ = writeln!(
                stderr,
                "cairn ingest: audit a path outside allowed roots: {e}"
            );
        }
        refused.to_string()
    })
}

/// The redactor for the settings in force: the rule set and the
/// principal's redaction patterns (SEC-08), each pattern refused
/// audited and counted while the rest still apply.
fn redactor(
    ctx: &Context,
    settings: &configuration::resolve::Settings,
    stderr: &mut dyn Write,
) -> Result<Redactor, String> {
    let patterns = settings.texts("redaction.extra_patterns");
    let (redactor, refused) = Redactor::new(patterns).map_err(|e| e.to_string())?;
    for r in &refused {
        let occurrence = Occurrence::Keyed {
            kind: Kind::Rejected,
            counter: Counter::RedactionPatternRefused,
            operation: "ingest",
            detail: "refused redaction pattern",
            key: &r.rule,
        };
        if let Err(e) = ctx.audit(&occurrence) {
            let _ = writeln!(
                stderr,
                "cairn ingest: audit a refused redaction pattern: {e}"
            );
        }
    }

    Ok(redactor)
}

/// Audits each redaction an ingest made, one entry per redaction
/// (SEC-08, I6), naming its rule and nothing of what it removed.
fn audit_redactions(ctx: &Context, rules: &[String], stderr: &mut dyn Write) {
    for rule in rules {
        let occurrence = Occurrence::Keyed {
            kind: Kind::Redacted,
            counter: Counter::Redacted,
            operation: "ingest",
            detail: "redacted by rule",
            key: rule,
        };
        if let Err(e) = ctx.audit(&occurrence) {
            let _ = writeln!(stderr, "cairn ingest: audit a redaction: {e}");
        }
    }
}

/// Audits each settings layer that fails and each key ignored before a
/// recording command goes on recording whatever they say (ADM-04, I9);
/// says on `stderr` what cannot be read or audited. Returns the
/// settings in force.
fn audit_settings(ctx: &Context, home: &Home, stderr: &mut dyn Write) -> settings::InForce {
    let acts = settings::recorded_acts(home).unwrap_or_else(|e| {
        let _ = writeln!(stderr, "cairn ingest: read the accepted configuration: {e}");
        Vec::new()
    });
    let in_force = settings::in_force(ctx, home, &acts);
    for failure in &in_force.loaded.failures {
        let _ = writeln!(stderr, "cairn ingest: recording without {failure}");
    }
    for e in settings::audit(ctx, "ingest", &in_force) {
        let _ = writeln!(stderr, "cairn ingest: audit the settings: {e}");
    }

    in_force
}

/// `cairn mcp`: serves the MCP tools over the process's input and
/// `stdout` until the input closes.
fn mcp(
    args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    if let Some(arg) = args.into_iter().next() {
        let _ = writeln!(stderr, "cairn mcp: unknown flag {arg:?}");
        return EXIT_USAGE;
    }
    let home = match ctx.home() {
        Ok(home) => home,
        Err(e) => return failed("mcp", &e, stderr),
    };
    // Recall reads a record a start made; it makes no home or store.
    if !home.store().exists() {
        let why = format!(
            "no store at {}: run cairn ingest first",
            home.store().display()
        );
        return failed("mcp", &why, stderr);
    }
    let served = Store::open(&home.store())
        .map_err(|e| e.to_string())
        .and_then(|store| {
            let node = Node::find(&home, &store).map_err(|e| e.to_string())?;
            let key = node.device().public();
            let (writer, room) = (node.device_writer(), node.personal_room());
            Ok(mcp::Server::new(
                store,
                ctx.build.version,
                key,
                writer,
                room,
            ))
        });
    let mut server = match served {
        Ok(server) => server,
        Err(e) => return failed("mcp", &e, stderr),
    };
    let served = mcp::serve(
        &mut |l| server.handle(l),
        &(ctx.input)(),
        stdout,
        IDLE,
        &mut on_mcp_panic(&home, ctx.clock, stderr),
    );
    match served {
        Ok(()) => EXIT_OK,
        Err(e) => failed("mcp", &e, stderr),
    }
}

/// `cairn verify [--json]`: checks the record against this node's
/// device key, exiting 3 at the first failure, which it names. No log
/// is signed yet, so a record that passes is `unsigned` (§9.7.5).
fn verify_command(
    args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let Some(as_json) = json_flag("verify", args, stderr) else {
        return EXIT_USAGE;
    };
    let home = match ctx.home() {
        Ok(home) => home,
        Err(e) => return failed("verify", &e, stderr),
    };
    if !home.store().exists() {
        return failed(
            "verify",
            &format!("no store at {}", home.store().display()),
            stderr,
        );
    }
    let store = match Store::open(&home.store()) {
        Ok(store) => store,
        Err(e) => return failed("verify", &e, stderr),
    };
    let device = match DeviceKey::load(&home.device_key()) {
        Ok(device) => device,
        Err(e) => return failed("verify", &e, stderr),
    };
    let checked = match verify(&store, &device.public()) {
        Ok(r) => Log::new(home.audit()).verify().map(|a| (r, a)).map_err(Err),
        Err(e) => Err(Ok(e)),
    };
    let (code, line) = match checked {
        Ok((r, a)) if as_json => (
            EXIT_OK,
            json!({
                "integrity": "unsigned",
                "writers": r.writers,
                "events": r.events,
                "order": r.full_order(),
                "audit": {"files": a.files, "entries": a.entries},
                "broken": null,
            })
            .to_string(),
        ),
        Ok((r, a)) => (
            EXIT_OK,
            format!(
                "checked {} writers and {} events: every chain, commitment and seat certificate \
                 holds; integrity unsigned, as no log is signed yet; the audit log's chain holds \
                 over {} entries",
                r.writers, r.events, a.entries
            ),
        ),
        Err(Ok(VerifyError::Store(e))) => return failed("verify", &e, stderr),
        Err(Err(e @ AuditError::Broken { .. })) => {
            broken_json(as_json, "audit", None, &e.to_string(), stderr)
        }
        Err(Err(e)) => return failed("verify", &e, stderr),
        Err(Ok(broken)) => broken_json(
            as_json,
            broken.class(),
            broken.address(),
            &broken.to_string(),
            stderr,
        ),
    };
    if line.is_empty() {
        return code;
    }

    match written(writeln!(stdout, "{line}"), stderr) {
        EXIT_OK => code,
        other => other,
    }
}

/// Reports a broken record or audit log on `stderr`, exiting 3, with
/// the failure's class, address and message as JSON when asked; the
/// line is empty when no JSON was asked for.
fn broken_json(
    as_json: bool,
    class: &str,
    address: Option<&str>,
    message: &str,
    stderr: &mut dyn Write,
) -> (u8, String) {
    let _ = writeln!(stderr, "cairn verify: {message}");
    if !as_json {
        return (EXIT_INTEGRITY, String::new());
    }
    let line = json!({
        "integrity": "broken",
        "writers": null,
        "events": null,
        "order": null,
        "audit": null,
        "broken": {"class": class, "address": address, "message": message},
    });

    (EXIT_INTEGRITY, line.to_string())
}

/// `cairn hook <hook>`: the hook handlers' entry point (§9.1). It reads
/// the hook input from stdin and writes at most the handler's output to
/// stdout, never anything to stderr, and exits 0 on every path: a
/// failure, a panic among them, is audited and the agent goes on
/// (ENG-04, I9). A home it must refuse is left untouched (I8).
fn hook_command(name: Option<&str>, ctx: &Context, stdout: &mut dyn Write) -> u8 {
    let hook = name.and_then(Hook::parse);
    let ran = match hook {
        None => Err(Failure::UnknownHook),
        Some(h) => hook::guarded(|| {
            if ctx.hook_panics {
                seam_panic();
            }
            let hook_input = hook_input(ctx)?;
            hook::handle(h, &hook_input)
        }),
    };
    match ran {
        Ok(out) => {
            let _ = stdout.write_all(out.as_bytes());
        }
        Err(failure) => {
            let operation = hook.map_or("hook", Hook::operation);
            // Nowhere to report an audit log that takes no entry: the
            // hook writes nothing to stderr. `cairn status` and `cairn
            // doctor` name what keeps the home from taking it. A panic
            // while auditing stops here too (ENG-04).
            let _ = panic::catch_unwind(AssertUnwindSafe(|| {
                ctx.audit(&failure.occurrence(operation))
            }));
        }
    }

    EXIT_OK
}

/// The development build's seam for the `@ENG-04` scenario: the hook
/// handler panics.
#[expect(
    clippy::panic,
    reason = "the seam's whole purpose is a panic the hook entry point must catch"
)]
fn seam_panic() {
    panic!("the hook handler panics, as CAIRN_TEST_FAULT asked");
}

/// The hook input on stdin, its lines joined, once the input closes.
/// Reading stops once it holds more than 1 MiB (SEC-05).
///
/// # Errors
///
/// [`Failure::TooLarge`] past 1 MiB or for a line the reader cut at its
/// limit; [`Failure::Malformed`] when the input cannot be read or does
/// not close in time.
#[expect(
    clippy::disallowed_methods,
    reason = "the deadline on reading the hook input (ENG-03); no derived artifact reads it (I10)"
)]
fn hook_input(ctx: &Context) -> Result<String, Failure> {
    let input = (ctx.input)();
    let deadline = Instant::now() + HOOK_INPUT_WAIT;
    let (mut text, mut lines) = (String::new(), 0usize);
    loop {
        let left = deadline
            .checked_duration_since(Instant::now())
            .ok_or(Failure::Malformed)?;
        match input.recv_timeout(left) {
            Ok(Ok(line)) => {
                if lines > 0 {
                    text.push('\n');
                }
                text.push_str(&line);
                lines += 1;
                if text.len() > hook::HOOK_INPUT_LIMIT {
                    return Err(Failure::TooLarge);
                }
            }
            Ok(Err(e)) if e.kind() == io::ErrorKind::FileTooLarge => return Err(Failure::TooLarge),
            Ok(Err(_)) | Err(RecvTimeoutError::Timeout) => return Err(Failure::Malformed),
            Err(RecvTimeoutError::Disconnected) => return Ok(text),
        }
    }
}

/// The lines `reader` holds, each without its line ending. A line
/// longer than `limit` bytes is never held whole (SEC-05): an error of
/// kind [`io::ErrorKind::FileTooLarge`] stands in its place, its rest
/// skipped, and the lines after it go on, as they do after one that is
/// not UTF-8. A read that fails ends them.
pub fn bounded_lines<R: io::BufRead>(
    reader: R,
    limit: usize,
) -> impl Iterator<Item = io::Result<String>> {
    let mut reader = Some(reader);
    std::iter::from_fn(move || {
        let r = reader.as_mut()?;
        let mut buf = Vec::new();
        // The limit, and room for a line ending of two bytes.
        let read = io::BufRead::read_until(
            &mut io::Read::take(r.by_ref(), limit as u64 + 2),
            b'\n',
            &mut buf,
        );
        match read {
            Ok(0) => None,
            Ok(_) if buf.last() != Some(&b'\n') && buf.len() > limit => {
                let skipped = io::BufRead::skip_until(r, b'\n');
                Some(skipped.and(Err(io::ErrorKind::FileTooLarge.into())))
            }
            Ok(_) => Some(line_of(buf, limit)),
            Err(e) => {
                reader = None;
                Some(Err(e))
            }
        }
    })
}

/// One line read with its ending, checked against `limit`.
fn line_of(mut buf: Vec<u8>, limit: usize) -> io::Result<String> {
    if buf.last() == Some(&b'\n') {
        buf.pop();
        if buf.last() == Some(&b'\r') {
            buf.pop();
        }
    }
    if buf.len() > limit {
        return Err(io::ErrorKind::FileTooLarge.into());
    }

    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// What the MCP server does with each panic it catches: audit it,
/// saying on `stderr` when the audit log takes no entry.
fn on_mcp_panic<'a>(
    home: &'a Home,
    clock: &'a dyn Fn() -> u64,
    stderr: &'a mut dyn Write,
) -> impl FnMut() + 'a {
    move || {
        if let Err(e) = mcp_panicked(home, clock()) {
            let _ = writeln!(stderr, "cairn mcp: audit a caught panic: {e}");
        }
    }
}

/// Audits a panic the MCP server caught.
fn mcp_panicked(home: &Home, time: u64) -> Result<Entry, AuditError> {
    let occurrence = Occurrence::Degraded {
        kind: Kind::Failed,
        counter: Counter::McpPanicked,
        operation: "mcp",
        detail: "panic recovered",
    };

    Log::new(home.audit()).append(&occurrence, time)
}

/// The counts the home's audit log gives, their rise counted after the
/// entry the counters were last acknowledged through, and its number of
/// entries; else the exit code, its failure reported: 3 for a broken
/// chain.
fn audit_counts(
    command: &str,
    home: &Home,
    acknowledged: Option<(u64, record::digest::Digest)>,
    stderr: &mut dyn Write,
) -> Result<(Counts, u64), u8> {
    match Log::new(home.audit()).entries() {
        Ok(entries) => Ok((counts(&entries, acknowledged), entries.len() as u64)),
        Err(e @ AuditError::Broken { .. }) => {
            let _ = writeln!(stderr, "cairn {command}: {e}");
            Err(EXIT_INTEGRITY)
        }
        Err(e) => Err(failed(command, &e, stderr)),
    }
}

/// The counts of `home`'s audit log since the counters were last
/// acknowledged, a neutral principal act the record holds (OWN-11), and
/// its number of entries; none before the home exists.
fn home_counts(
    command: &str,
    home: &Home,
    acts: &[act_record::Recorded],
    stderr: &mut dyn Write,
) -> Result<Option<(Counts, u64)>, u8> {
    if !home.root().exists() {
        return Ok(None);
    }

    audit_counts(command, home, act_record::acknowledged(acts), stderr).map(Some)
}

/// `cairn doctor [--json]`: read-only (ADM-10); exits 1 while a counter
/// rose since the counters were last acknowledged, naming each
/// (OPS-03).
fn doctor(
    args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let Some(as_json) = json_flag("doctor", args, stderr) else {
        return EXIT_USAGE;
    };
    let read = ctx
        .home()
        .map_err(|e| e.to_string())
        .and_then(|home| Ok((settings::recorded_acts(&home)?, home)));
    let (acts, home) = match read {
        Ok(read) => read,
        Err(e) => return failed("doctor", &e, stderr),
    };
    let counts = match home_counts("doctor", &home, &acts, stderr) {
        Ok(c) => c.map(|(c, _)| c),
        Err(code) => return code,
    };
    let code = if counts.as_ref().is_some_and(Counts::risen) {
        EXIT_FAILURE
    } else {
        EXIT_OK
    };
    let risen = counts.map(|c| c.since_ack);
    let text = if as_json {
        json!({"risen": risen.unwrap_or_default()}).to_string()
    } else {
        match risen {
            None => format!(
                "no home at {} yet: nothing to diagnose",
                home.root().display()
            ),
            Some(r) if r.is_empty() => {
                "no counter rose since the counters were last acknowledged".to_owned()
            }
            Some(r) => r
                .iter()
                .map(|(name, n)| {
                    format!("counter {name} rose by {n} since the counters were last acknowledged")
                })
                .collect::<Vec<_>>()
                .join("\n"),
        }
    };

    match written(writeln!(stdout, "{text}"), stderr) {
        EXIT_OK => code,
        other => other,
    }
}

/// What `cairn status` shows.
struct Status {
    home: String,
    /// The store's path and size, once the home exists.
    store: Option<(String, Option<u64>)>,
    /// The audit log's path and number of entries, once the home exists.
    audit: Option<(String, u64)>,
    /// Each counter's total and rise since the last acknowledgement.
    counters: Vec<(String, u64, u64)>,
    deployment_mode: String,
    awaiting_acceptance: Vec<String>,
    /// Why managed policy is refused, when it is.
    managed_policy: Option<String>,
}

impl Status {
    fn json(&self) -> String {
        let counters: BTreeMap<&String, Value> = self
            .counters
            .iter()
            .map(|(name, total, since)| (name, json!({"total": total, "since_ack": since})))
            .collect();
        json!({
            "home": self.home,
            "store": self.store.as_ref().map(|(p, b)| json!({"path": p, "bytes": b})),
            "audit": self.audit.as_ref().map(|(p, n)| json!({"path": p, "entries": n})),
            "counters": counters,
            "deployment_mode": self.deployment_mode,
            "awaiting_acceptance": self.awaiting_acceptance,
            "managed_policy": {"refused": self.managed_policy},
        })
        .to_string()
    }

    fn text(&self) -> String {
        let mut lines = Vec::new();
        match (&self.store, &self.audit) {
            (Some((store, bytes)), Some((audit, entries))) => {
                let size = bytes.map_or("none yet".to_owned(), |b| format!("{b} bytes"));
                lines.push(format!("home: {}", self.home));
                lines.push(format!("store: {store} ({size})"));
                lines.push(format!("audit log: {audit} ({entries} entries)"));
            }
            _ => lines.push(format!("home: {} (none yet)", self.home)),
        }
        let counters: Vec<String> = self
            .counters
            .iter()
            .map(|(name, total, since)| format!("{name} {total} ({since} since acknowledged)"))
            .collect();
        if self.store.is_some() {
            let counters = if counters.is_empty() {
                "none".to_owned()
            } else {
                counters.join(", ")
            };
            lines.push(format!("counters: {counters}"));
        }
        lines.push(format!("deployment mode: {}", self.deployment_mode));
        if !self.awaiting_acceptance.is_empty() {
            lines.push(format!(
                "awaiting acceptance (cairn configuration accept): {}",
                self.awaiting_acceptance.join(", ")
            ));
        }
        if let Some(why) = &self.managed_policy {
            lines.push(format!("managed policy refused: {why}"));
        }

        lines.join("\n")
    }
}

/// `cairn status [--json]`: the home, the store, the audit log, the
/// non-zero counters and the settings in force (ADM-11, which phase 15
/// completes). It exits 2 while the configuration or the repository
/// configuration fails validation (ADM-04).
fn status(
    args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let Some(as_json) = json_flag("status", args, stderr) else {
        return EXIT_USAGE;
    };
    let read = ctx
        .home()
        .map_err(|e| e.to_string())
        .and_then(|home| Ok((settings::recorded_acts(&home)?, home)));
    let (acts, home) = match read {
        Ok(read) => read,
        Err(e) => return failed("status", &e, stderr),
    };
    let in_force = settings::in_force(ctx, &home, &acts);
    if in_force.loaded.principal_layer_failed() {
        return invalid_layers("status", &in_force.loaded, stderr);
    }
    let counted = match home_counts("status", &home, &acts, stderr) {
        Ok(c) => c,
        Err(code) => return code,
    };
    let status = status_of(&home, counted, &in_force);
    let text = if as_json {
        status.json()
    } else {
        status.text()
    };

    written(writeln!(stdout, "{text}"), stderr)
}

/// The status of `home`, given its audit log's counts.
fn status_of(home: &Home, counted: Option<(Counts, u64)>, in_force: &settings::InForce) -> Status {
    let (store, audit) = (home.store(), home.audit());
    let mut status = Status {
        home: home.root().display().to_string(),
        store: None,
        audit: None,
        counters: Vec::new(),
        deployment_mode: in_force.resolved.settings.deployment_mode().to_owned(),
        awaiting_acceptance: in_force
            .resolved
            .awaiting_acceptance
            .iter()
            .map(|k| k.name().to_owned())
            .collect(),
        managed_policy: in_force
            .loaded
            .failures
            .iter()
            .find(|f| f.layer == SettingsLayer::ManagedPolicy)
            .map(ToString::to_string),
    };
    if let Some((counts, entries)) = counted {
        let bytes = fs::metadata(&store).ok().map(|m| m.len());
        status.store = Some((store.display().to_string(), bytes));
        status.audit = Some((audit.display().to_string(), entries));
        status.counters = counts
            .total
            .iter()
            .map(|(name, total)| {
                let since = counts.since_ack.get(name).copied().unwrap_or(0);
                (name.clone(), *total, since)
            })
            .collect();
    }

    status
}

/// `s` as a JSON string literal.
fn json_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if u32::from(c) < 0x20 => out.push_str(&format!("\\u{:04x}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');

    out
}

/// Turns the result of writing the command's output into its exit
/// code: a write that fails, to a closed stdout say, is a failure.
pub(crate) fn written(result: std::io::Result<()>, stderr: &mut dyn Write) -> u8 {
    match result {
        Ok(()) => EXIT_OK,
        Err(e) => {
            let _ = writeln!(stderr, "cairn: write the output: {e}");
            EXIT_FAILURE
        }
    }
}

/// Reports each failed settings layer but managed policy, exiting 2
/// (ADM-04): the error names the layer, the key and the problem.
pub(crate) fn invalid_layers(command: &str, loaded: &Loaded, stderr: &mut dyn Write) -> u8 {
    for failure in loaded
        .failures
        .iter()
        .filter(|f| f.layer != SettingsLayer::ManagedPolicy)
    {
        let _ = writeln!(stderr, "cairn {command}: {failure}");
    }

    EXIT_USAGE
}

mod act;
mod settings;

pub use settings::managed_policy;

#[cfg(test)]
mod tests;
