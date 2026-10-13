//! Timing one sample of a benchmark. Each blocking wait has a deadline
//! (ENG-03): a program that does not exit within it is an error, not a
//! slow sample, so a hung benchmark fails the run instead of stalling
//! the CI job until its timeout.

use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::suite::Benchmark;

/// How long one sample may take before it is an error.
pub const DEADLINE: Duration = Duration::from_secs(10);

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
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let (done, result) = mpsc::channel();
    thread::spawn(move || {
        let started = Instant::now();
        let status = command.status();
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
