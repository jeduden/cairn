//! ENG-12: the end-to-end suite runs each component's entry point
//! confined to its boundary, and fails on any socket outside it,
//! naming the component and the test that started it.
//!
//! Only the core exists, so only its row is here: all network denied.
//! A confined start runs the program in a fresh user and network
//! namespace (`unshare --net`), which holds no interface but a loopback
//! that is down, under `strace -f`, which records every network call
//! the program or any process it starts makes, and fails each
//! `socket`, `socketpair` and `io_uring_setup` with `EACCES`. The
//! namespace denies IP even on a path the trace could miss; the trace
//! denies the Unix sockets a namespace lets through, and catches every
//! attempt, which a namespace alone lets fail silently. Any recorded
//! call fails the start. The room-view component, the launcher and the
//! peer, publish and bridge components join with their own rows as M7
//! to M9 build them.
//!
//! A machine that cannot confine fails every start, loudly. A developer
//! whose machine lacks the tools (not Linux, no `strace`, or user
//! namespaces turned off) may set [`UNCONFINED`] to run the end-to-end
//! tests unconfined there; CI refuses it, so the gate holds where it
//! counts.

use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::panic::Location;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use crate::{TempDir, isolate};

/// The opt-out: set, outside CI, the end-to-end tests run unconfined.
pub const UNCONFINED: &str = "CAIRN_E2E_UNCONFINED";

/// Given as its first argument, an executable that hosts the probe
/// runs [`probe`] instead of itself: the scenario runner's way to stand
/// in for a component that opens a socket.
pub const PROBE: &str = "--cairn-boundary-probe";

/// The variable CI sets.
const CI: &str = "CI";

/// The tool that makes the namespace.
const UNSHARE: &str = "unshare";

/// The namespace: the current OS user, and a network of its own.
const NAMESPACE: [&str; 3] = ["--map-current-user", "--net", "--"];

/// The trace, up to its output file: every process followed, attach
/// and exit notes and signals left out, every network call recorded,
/// and each way to open a socket failed.
const TRACE: [&str; 10] = [
    "strace",
    "-f",
    "-qq",
    "-e",
    "signal=none",
    "-e",
    "trace=%net,io_uring_setup",
    "-e",
    "inject=socket,socketpair,io_uring_setup:error=EACCES",
    "-o",
];

/// A component the end-to-end suite starts confined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Component {
    /// B0, the core, as ENG-12 and §6.3 list it.
    Core,
}

impl Component {
    /// The component a scenario names.
    ///
    /// # Errors
    ///
    /// Fails on a component the suite cannot confine yet.
    pub fn named(name: &str) -> Result<Self, Error> {
        match name {
            "core" => Ok(Self::Core),
            other => Err(Error::NoConfinement {
                component: other.to_owned(),
            }),
        }
    }

    /// The component's name, as the boundary register writes it.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Core => "core",
        }
    }

    /// What its confinement denies, as ENG-12's scenario says it.
    #[must_use]
    pub fn confinement(self) -> &'static str {
        match self {
            Self::Core => "denies all network",
        }
    }
}

/// How a start runs on this machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Confined,
    /// Opted out with [`UNCONFINED`], outside CI.
    Unconfined,
}

/// The confinement one component's starts run in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Confinement {
    component: Component,
    mode: Mode,
}

impl Confinement {
    /// The confinement `component` runs in on this machine.
    ///
    /// # Errors
    ///
    /// Fails when this machine cannot confine and no opt-out says to
    /// run unconfined, or when CI is handed the opt-out.
    pub fn of(component: Component) -> Result<Self, Error> {
        let mode = mode(|k| std::env::var_os(k), preflight)?;

        Ok(Self { component, mode })
    }

    /// The component it confines.
    #[must_use]
    pub fn component(&self) -> Component {
        self.component
    }

    /// Starts `program` with `args` and a `HOME` and `CAIRN_HOME` under
    /// `home` (ENG-14), confined, and waits for it. The caller, the
    /// test thread and the place that called, is named in a failure.
    ///
    /// # Errors
    ///
    /// Fails when the program cannot start, when its trace cannot be
    /// read, or when it, or anything it started, made a network call.
    #[track_caller]
    pub fn run(&self, program: &Path, args: &[&str], home: &Path) -> Result<Output, Error> {
        let caller = caller(std::thread::current().name(), Location::caller());
        self.start(program, args, home, &[], None, caller)
    }

    /// [`run`](Self::run), with `input` written to the program's stdin,
    /// which then closes. The input is written whole before the output
    /// is read, so it suits the short exchanges a test makes.
    ///
    /// # Errors
    ///
    /// As [`run`](Self::run), and when the input cannot be written.
    #[track_caller]
    pub fn run_with_input(
        &self,
        program: &Path,
        args: &[&str],
        home: &Path,
        input: &[u8],
    ) -> Result<Output, Error> {
        let caller = caller(std::thread::current().name(), Location::caller());
        self.start(program, args, home, &[], Some(input), caller)
    }

    /// [`run`](Self::run), with each of `env` set in the program's
    /// environment, and `input`, when given, written to its stdin.
    ///
    /// # Errors
    ///
    /// As [`run_with_input`](Self::run_with_input).
    #[track_caller]
    pub fn run_in(
        &self,
        program: &Path,
        args: &[&str],
        home: &Path,
        env: &[(&str, &str)],
        input: Option<&[u8]>,
    ) -> Result<Output, Error> {
        let caller = caller(std::thread::current().name(), Location::caller());
        self.start(program, args, home, env, input, caller)
    }

    fn start(
        &self,
        program: &Path,
        args: &[&str],
        home: &Path,
        env: &[(&str, &str)],
        input: Option<&[u8]>,
        caller: String,
    ) -> Result<Output, Error> {
        let dir = TempDir::new().map_err(|source| Error::Trace { source })?;
        let trace = dir.path().join("trace");
        let mut command = self.command(program, args, &trace);
        isolate(&mut command, home);
        command.envs(env.iter().copied());
        let output = match input {
            None => command.stdin(Stdio::null()).output(),
            Some(bytes) => feed(&mut command, bytes),
        }
        .map_err(|source| Error::Start {
            program: program.to_path_buf(),
            source,
        })?;
        if self.mode == Mode::Unconfined {
            return Ok(output);
        }

        self.finish(output, fs::read_to_string(&trace), caller)
    }

    /// The command that starts `program`, wrapped in the namespace and
    /// the trace writing to `trace` when confined.
    fn command(&self, program: &Path, args: &[&str], trace: &Path) -> Command {
        let mut command = match self.mode {
            Mode::Unconfined => Command::new(program),
            Mode::Confined => {
                let mut c = Command::new(UNSHARE);
                c.args(NAMESPACE)
                    .args(TRACE)
                    .arg(trace)
                    .arg("--")
                    .arg(program);
                c
            }
        };
        command.args(args);

        command
    }

    /// The start's outcome from its output and its trace: any network
    /// call in the trace fails it.
    fn finish(
        &self,
        output: Output,
        trace: io::Result<String>,
        caller: String,
    ) -> Result<Output, Error> {
        let calls = network_calls(&trace.map_err(|source| Error::Trace { source })?);
        if calls.is_empty() {
            Ok(output)
        } else {
            Err(Error::Socket {
                component: self.component,
                caller,
                calls,
            })
        }
    }
}

/// Starts `command` with `input` on its stdin, closes it, and waits
/// for the program's output.
fn feed(command: &mut Command, input: &[u8]) -> io::Result<Output> {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    // The stdin was piped above, so it is there; dropping it closes it.
    child
        .stdin
        .take()
        .map_or(Ok(()), |mut stdin| stdin.write_all(input))?;

    child.wait_with_output()
}

/// How a start runs, from the environment `var` reads: unconfined when
/// opted out outside CI, refused when opted out in CI, else confined
/// once `preflight` shows this machine can confine.
fn mode(
    var: impl Fn(&str) -> Option<OsString>,
    preflight: impl FnOnce() -> Result<(), String>,
) -> Result<Mode, Error> {
    let set = |k: &str| var(k).is_some_and(|v| !v.is_empty() && v != "false");
    match (set(UNCONFINED), set(CI)) {
        (true, true) => Err(Error::Refused),
        (true, false) => Ok(Mode::Unconfined),
        (false, _) => preflight()
            .map(|()| Mode::Confined)
            .map_err(|reason| Error::Unavailable { reason }),
    }
}

/// Whether this machine can confine: `true` runs in the namespace
/// under the trace.
fn preflight() -> Result<(), String> {
    preflight_with(UNSHARE)
}

/// [`preflight`] with the namespace tool `unshare`.
fn preflight_with(unshare: &str) -> Result<(), String> {
    let mut command = Command::new(unshare);
    command
        .args(NAMESPACE)
        .args(TRACE)
        .args(["/dev/null", "--", "true"]);
    let out = command
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("start {unshare}: {e}"))?;
    if out.status.success() {
        return Ok(());
    }
    let line: Vec<String> = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|a| a.to_string_lossy().into_owned())
        .collect();

    Err(format!(
        "{} exited {}: {}",
        line.join(" "),
        out.status,
        String::from_utf8_lossy(&out.stderr).trim()
    ))
}

/// The network calls a trace records: every line that is not blank,
/// the process id strace puts first left out.
fn network_calls(trace: &str) -> Vec<String> {
    trace
        .lines()
        .map(|l| {
            l.trim()
                .trim_start_matches(|c: char| c.is_ascii_digit())
                .trim()
        })
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Names the caller: the thread, which libtest names after the test,
/// and the place in the source.
fn caller(thread: Option<&str>, at: &Location<'_>) -> String {
    format!(
        "{} ({}:{})",
        thread.unwrap_or("an unnamed thread"),
        at.file(),
        at.line()
    )
}

/// The probe: opens a socket on loopback, as a component reaching
/// outside its boundary would. A confined start of it must fail; see
/// [`PROBE`].
///
/// # Errors
///
/// Fails when the socket cannot be opened, as a confinement makes it.
pub fn probe() -> io::Result<()> {
    std::net::UdpSocket::bind(("127.0.0.1", 0)).map(drop)
}

/// What a confined start can fail with.
#[derive(Debug)]
pub enum Error {
    /// The suite has no confinement for this component yet.
    NoConfinement { component: String },
    /// CI was handed the opt-out.
    Refused,
    /// This machine cannot confine, and no opt-out says to run
    /// unconfined.
    Unavailable { reason: String },
    /// The program, or the tools around it, did not start.
    Start { program: PathBuf, source: io::Error },
    /// The trace could not be made or read.
    Trace { source: io::Error },
    /// The component, or something it started, made a network call.
    Socket {
        component: Component,
        caller: String,
        calls: Vec<String>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoConfinement { component } => write!(
                f,
                "ENG-12: the end-to-end suite cannot confine the {component} yet: only the core exists"
            ),
            Self::Refused => write!(
                f,
                "ENG-12: {UNCONFINED} is set, but CI runs every component confined: unset it"
            ),
            Self::Unavailable { reason } => write!(
                f,
                "ENG-12: the end-to-end suite cannot confine a component on this machine: {reason}\n\
                 it needs Linux, unshare (util-linux) with unprivileged user namespaces, and strace; \
                 install them, or set {UNCONFINED}=1 to run the end-to-end tests unconfined here \
                 (CI refuses it)"
            ),
            Self::Start { program, source } => {
                write!(f, "ENG-12: start {}: {source}", program.display())
            }
            Self::Trace { source } => {
                write!(
                    f,
                    "ENG-12: make or read the trace of a confined start: {source}"
                )
            }
            Self::Socket {
                component,
                caller,
                calls,
            } => write!(
                f,
                "ENG-12: the {} opened a socket outside its boundary, which {}; started by {caller}:\n  {}",
                component.name(),
                component.confinement(),
                calls.join("\n  ")
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Start { source, .. } | Self::Trace { source } => Some(source),
            Self::NoConfinement { .. }
            | Self::Refused
            | Self::Unavailable { .. }
            | Self::Socket { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests;
