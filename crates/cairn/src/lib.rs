//! The Cairn core executable, specified in docs/srs: the hook
//! handlers, the MCP server and the CLI so far.
//!
//! It runs `version`, `ingest --path`, `mcp` and `verify`. Every other
//! command in the SRS's §9.5 is a pending scenario under features/, and
//! lands with the plan that makes its scenario pass.

use std::ffi::OsString;
use std::io::{self, Write};
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use administration::home::Home;
use administration::node::Node;
use administration::verify::{VerifyError, verify};
use key::random::OsEntropy;
use key::signing::DeviceKey;
use serde_json::json;
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
  ingest --path P [--json]   record the transcript at P
  mcp                        serve the MCP tools over stdio
  verify [--json]            check the record against this node's key
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
    /// Starts reading the process's input, one line at a time.
    pub input: &'a dyn Fn() -> Receiver<io::Result<String>>,
}

impl Context<'_> {
    fn home(&self) -> Result<Home, administration::AdminError> {
        Home::locate(self.cairn_home.as_deref(), self.home.as_deref())
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
        Some("ingest") => ingest(args, ctx, stdout, stderr),
        Some("mcp") => mcp(args, ctx, stdout, stderr),
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
    let mut as_json = false;
    for arg in args {
        if arg != "--json" {
            let _ = writeln!(stderr, "cairn version: unknown flag {arg:?}");
            return EXIT_USAGE;
        }
        as_json = true;
    }
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
    match mcp::serve(
        &mut |l| server.handle(l),
        &(ctx.input)(),
        stdout,
        IDLE,
        &mut || {},
    ) {
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
    let checked = verify(&store, &device.public());
    let (code, line) = match checked {
        Ok(r) if as_json => (
            EXIT_OK,
            json!({
                "integrity": "unsigned",
                "writers": r.writers,
                "events": r.events,
                "order": r.full_order(),
                "broken": null,
            })
            .to_string(),
        ),
        Ok(r) => (
            EXIT_OK,
            format!(
                "checked {} writers and {} events: every chain, commitment and seat certificate \
                 holds; integrity unsigned, as no log is signed yet",
                r.writers, r.events
            ),
        ),
        Err(VerifyError::Store(e)) => return failed("verify", &e, stderr),
        Err(broken) => {
            let _ = writeln!(stderr, "cairn verify: {broken}");
            if !as_json {
                return EXIT_INTEGRITY;
            }
            let failure = json!({
                "class": broken.class(),
                "address": broken.address(),
                "message": broken.to_string(),
            });
            (
                EXIT_INTEGRITY,
                json!({
                    "integrity": "broken",
                    "writers": null,
                    "events": null,
                    "order": null,
                    "broken": failure,
                })
                .to_string(),
            )
        }
    };

    match written(writeln!(stdout, "{line}"), stderr) {
        EXIT_OK => code,
        other => other,
    }
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
