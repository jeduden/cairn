//! The Cairn core executable, specified in docs/srs: the hook
//! handlers, the MCP server and the CLI so far.
//!
//! It runs `version`, `ingest --path`, `mcp`, `verify`, `hook`,
//! `status`, `doctor` and `counter ack`. Every other
//! command in the SRS's §9.5 is a pending scenario under features/, and
//! lands with the plan that makes its scenario pass.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::panic::{self, AssertUnwindSafe};
use std::path::Path;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use administration::home::Home;
use administration::node::Node;
use administration::verify::{VerifyError, verify};
use audit::AuditError;
use audit::counters::{Counts, counts};
use audit::entry::{Counter, Entry, Kind, Occurrence};
use audit::log::Log;
use hook::{Failure, Hook};
use key::random::OsEntropy;
use key::signing::DeviceKey;
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

/// Why a release build refuses `cairn ingest`: until redaction before
/// storage (SEC-08) lands, an ingested secret would stay in the
/// append-only record with no purge (ADM-07) to remove it. A
/// development build still ingests, as the tests and the proving slice
/// need.
const RELEASE_INGEST: &str = "a release build records no transcript until redaction before \
                              storage (SEC-08) lands; use a development build";

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
  counter ack [--json]       acknowledge the counters
  doctor [--json]            fail while a counter rose since the last acknowledgement
  hook <hook>                run the hook handler on the hook input on stdin
  ingest --path P [--json]   record the transcript at P
  mcp                        serve the MCP tools over stdio
  status [--json]            show the home, the store, the audit log and the counters
  verify [--json]            check the record and the audit log
  version [--json]           print the build version
  help                       print this message
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
        Some("counter") => counter(args, ctx, stdout, stderr),
        Some("doctor") => doctor(args, ctx, stdout, stderr),
        Some("hook") => hook_command(args.next().as_deref(), ctx, stdout),
        Some("ingest") => ingest(args, ctx, stdout, stderr),
        Some("mcp") => mcp(args, ctx, stdout, stderr),
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
fn json_flag(
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
fn failed(command: &str, e: &dyn std::fmt::Display, stderr: &mut dyn Write) -> u8 {
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
    if ctx.build.version != BUILD_DEV {
        return failed("ingest", &RELEASE_INGEST, stderr);
    }
    let ingested = match ctx.home() {
        Ok(home) => {
            ingest::ingest_path(&home, Path::new(&path), &mut OsEntropy).map_err(|e| e.to_string())
        }
        Err(e) => Err(e.to_string()),
    };
    let ingested = match ingested {
        Ok(i) => i,
        Err(e) => return failed("ingest", &e, stderr),
    };
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
            let hook_input = hook_input(ctx).ok_or(Failure::Malformed)?;
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

/// The hook input on stdin, its lines joined, once the input closes;
/// none when it cannot be read or does not close in time.
#[expect(
    clippy::disallowed_methods,
    reason = "the deadline on reading the hook input (ENG-03); no derived artifact reads it (I10)"
)]
fn hook_input(ctx: &Context) -> Option<String> {
    let input = (ctx.input)();
    let deadline = Instant::now() + HOOK_INPUT_WAIT;
    let mut lines = Vec::new();
    loop {
        let left = deadline.checked_duration_since(Instant::now())?;
        match input.recv_timeout(left) {
            Ok(Ok(line)) => lines.push(line),
            Ok(Err(_)) | Err(RecvTimeoutError::Timeout) => return None,
            Err(RecvTimeoutError::Disconnected) => return Some(lines.join("\n")),
        }
    }
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

/// `cairn counter ack [--json]`: acknowledges the counters (OPS-03). It
/// appends a `counter_ack` entry to the audit log; phase 3 records the
/// neutral principal act it is (OWN-11).
fn counter(
    mut args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    if args.next().as_deref() != Some("ack") {
        let _ = writeln!(stderr, "cairn counter: want ack");
        return EXIT_USAGE;
    }
    let Some(as_json) = json_flag("counter ack", args, stderr) else {
        return EXIT_USAGE;
    };
    let entry = match ctx.audit(&Occurrence::CounterAck) {
        Ok(entry) => entry,
        Err(e) => return failed("counter ack", &e, stderr),
    };
    let line = if as_json {
        json!({"acknowledged": entry.number}).to_string()
    } else {
        "counters acknowledged".to_owned()
    };

    written(writeln!(stdout, "{line}"), stderr)
}

/// The counts the home's audit log gives, and its number of entries;
/// else the exit code, its failure reported: 3 for a broken chain.
fn audit_counts(command: &str, home: &Home, stderr: &mut dyn Write) -> Result<(Counts, u64), u8> {
    match Log::new(home.audit()).entries() {
        Ok(entries) => Ok((counts(&entries), entries.len() as u64)),
        Err(e @ AuditError::Broken { .. }) => {
            let _ = writeln!(stderr, "cairn {command}: {e}");
            Err(EXIT_INTEGRITY)
        }
        Err(e) => Err(failed(command, &e, stderr)),
    }
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
    let home = match ctx.home() {
        Ok(home) => home,
        Err(e) => return failed("doctor", &e, stderr),
    };
    let counts = if home.root().exists() {
        match audit_counts("doctor", &home, stderr) {
            Ok((c, _)) => Some(c),
            Err(code) => return code,
        }
    } else {
        None
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

/// `cairn status [--json]`: the home, the store, the audit log and the
/// non-zero counters (ADM-11, which phase 15 completes).
fn status(
    args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let Some(as_json) = json_flag("status", args, stderr) else {
        return EXIT_USAGE;
    };
    let home = match ctx.home() {
        Ok(home) => home,
        Err(e) => return failed("status", &e, stderr),
    };
    let root = home.root().display().to_string();
    if !home.root().exists() {
        let text = if as_json {
            json!({"home": root, "store": null, "audit": null, "counters": {}}).to_string()
        } else {
            format!("home: {root} (none yet)")
        };
        return written(writeln!(stdout, "{text}"), stderr);
    }
    let (counts, entries) = match audit_counts("status", &home, stderr) {
        Ok(c) => c,
        Err(code) => return code,
    };
    let (store, audit) = (home.store(), home.audit());
    let bytes = fs::metadata(&store).ok().map(|m| m.len());
    let rows: Vec<(&String, u64, u64)> = counts
        .total
        .iter()
        .map(|(name, total)| {
            let since = counts.since_ack.get(name).copied().unwrap_or(0);
            (name, *total, since)
        })
        .collect();
    let text = if as_json {
        let counters: BTreeMap<&String, Value> = rows
            .iter()
            .map(|(name, total, since)| (*name, json!({"total": total, "since_ack": since})))
            .collect();
        json!({
            "home": root,
            "store": {"path": store.display().to_string(), "bytes": bytes},
            "audit": {"path": audit.display().to_string(), "entries": entries},
            "counters": counters,
        })
        .to_string()
    } else {
        let size = bytes.map_or("none yet".to_owned(), |b| format!("{b} bytes"));
        let counters = rows
            .iter()
            .map(|(name, total, since)| format!("{name} {total} ({since} since acknowledged)"))
            .collect::<Vec<_>>();
        let counters = if counters.is_empty() {
            "none".to_owned()
        } else {
            counters.join(", ")
        };
        format!(
            "home: {root}\nstore: {} ({size})\naudit log: {} ({entries} entries)\ncounters: {counters}",
            store.display(),
            audit.display()
        )
    };

    written(writeln!(stdout, "{text}"), stderr)
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
fn written(result: std::io::Result<()>, stderr: &mut dyn Write) -> u8 {
    match result {
        Ok(()) => EXIT_OK,
        Err(e) => {
            let _ = writeln!(stderr, "cairn: write the output: {e}");
            EXIT_FAILURE
        }
    }
}

#[cfg(test)]
mod tests;
