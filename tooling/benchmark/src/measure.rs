//! Timing one sample of a benchmark. Each blocking wait has a deadline
//! (ENG-03): a program that does not exit within it is an error, not a
//! slow sample, so a hung benchmark fails the run instead of stalling
//! the CI job until its timeout.

use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::fixture;
use crate::suite::{Benchmark, Fixture};

/// How long one sample may take before it is an error.
pub const DEADLINE: Duration = Duration::from_secs(10);

/// How long preparing a sample's fixture may take.
pub const PREPARE_DEADLINE: Duration = Duration::from_secs(120);

/// Times `program` started with `args` until it exits, its standard
/// streams discarded. The time is taken on the thread that waits, so it
/// counts the program's start and run and nothing of the wait for the
/// result.
///
/// # Errors
///
/// Fails when the program cannot start, exits unsuccessfully, or has
/// not exited by `deadline`; it is then left to finish on its own.
pub fn time_program(program: &Path, args: &[&str], deadline: Duration) -> Result<Duration, String> {
    time_program_in(program, args, &[], deadline)
}

/// Times one sample of `benchmark` against the core executable
/// `program`: over a fresh fixture, made before and removed after, that
/// counts nothing of the time.
///
/// # Errors
///
/// Fails when the fixture cannot be made, or as [`time_program`].
pub fn time_benchmark(
    program: &Path,
    benchmark: &Benchmark,
    deadline: Duration,
) -> Result<Duration, String> {
    match benchmark.fixture {
        Fixture::None => time_program(program, benchmark.args, deadline),
        Fixture::Transcript { lines } => {
            let home =
                fixture::Home::with_transcript(&std::env::temp_dir(), &fixture::transcript(lines))
                    .map_err(|e| format!("make the fixture of {}: {e}", benchmark.name))?;
            time_program_in(program, benchmark.args, &home.env(), deadline)
        }
        Fixture::Search { lines, prepare } => {
            let home =
                fixture::Home::with_transcript(&std::env::temp_dir(), &fixture::transcript(lines))
                    .map_err(|e| format!("make the fixture of {}: {e}", benchmark.name))?;
            time_program_in(program, prepare, &home.env(), PREPARE_DEADLINE)
                .map_err(|e| format!("prepare the fixture of {}: {e}", benchmark.name))?;
            let input = Some(fixture::SEARCH_REQUESTS);
            time_program_fed(program, benchmark.args, &home.env(), input, deadline)
        }
    }
}

/// Times `program` as [`time_program`] does, with `env` set beside the
/// environment it inherits.
///
/// # Errors
///
/// As [`time_program`].
pub fn time_program_in(
    program: &Path,
    args: &[&str],
    env: &[(&str, std::path::PathBuf)],
    deadline: Duration,
) -> Result<Duration, String> {
    time_program_fed(program, args, env, None, deadline)
}

/// Times `program` as [`time_program_in`] does, `input` written to its
/// standard input, which then closes; none gives it no input at all.
///
/// # Errors
///
/// As [`time_program`].
pub fn time_program_fed(
    program: &Path,
    args: &[&str],
    env: &[(&str, std::path::PathBuf)],
    input: Option<&'static str>,
    deadline: Duration,
) -> Result<Duration, String> {
    let mut command = Command::new(program);
    command
        .args(args)
        .envs(env.iter().map(|(k, v)| (*k, v.as_os_str())))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let (done, result) = mpsc::channel();
    thread::spawn(move || {
        let started = Instant::now();
        let status = run(command, input);
        let _ = done.send((started.elapsed(), status));
    });

    let shown = program.display();
    match result.recv_timeout(deadline) {
        Ok((took, Ok(status))) if status.success() => Ok(took),
        Ok((_, Ok(status))) => Err(format!("{shown} {} exited {status}", args.join(" "))),
        Ok((_, Err(err))) => Err(format!("start {shown}: {err}")),
        Err(_) => Err(format!(
            "{shown} {} did not exit within {deadline:?}",
            args.join(" ")
        )),
    }
}

/// Runs `command` to its exit, `input` on its standard input when given.
fn run(mut command: Command, input: Option<&str>) -> io::Result<ExitStatus> {
    let Some(input) = input else {
        return command.status();
    };
    let mut child = command.stdin(Stdio::piped()).spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes())?;
    }

    child.wait()
}

/// Takes `count` samples of `benchmark` with `time`, after one warm-up
/// sample it discards, so the first sample does not pay for a cold
/// page cache the others do not.
///
/// # Errors
///
/// Fails on the first sample `time` cannot take, naming the benchmark.
pub fn sample(
    benchmark: &Benchmark,
    count: usize,
    time: &mut dyn FnMut(&Benchmark) -> Result<Duration, String>,
) -> Result<Vec<u64>, String> {
    let name = benchmark.name;
    time(benchmark).map_err(|e| format!("{name}: {e}"))?;
    (0..count)
        .map(|_| {
            time(benchmark)
                .map(|took| u64::try_from(took.as_nanos()).unwrap_or(u64::MAX))
                .map_err(|e| format!("{name}: {e}"))
        })
        .collect()
}

#[cfg(test)]
mod tests;
