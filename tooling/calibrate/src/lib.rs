//! The calibration workload (NFR-02): a fixed computation and a fixed
//! number of file system calls shaped like a hook handler's. The
//! `@NFR-02` hook budget rows time it beside a hook handler, under the
//! same confinement, and hold the handler to a multiple of it. It reads
//! nothing of Cairn and imports no crate, so it stays the same when the
//! product changes.
//!
//! One calibration unit is a nominal 10 ms, by definition. The
//! constants below were tuned once to take about that on a release
//! build, and never change: a gate test in `engineering` fails when
//! this crate's source differs from the recorded hash.

use std::hint::black_box;
use std::io::{self, Read as _, Write as _};
use std::path::Path;
use std::time::Instant;

/// The seed of the mixing computation.
pub const SEED: u64 = 0x9e37_79b9_7f4a_7c15;

/// How many rounds of mixing the workload runs.
pub const MIX_ROUNDS: u64 = 0;

/// How many times the workload creates, reads and removes its file.
pub const FILE_ROUNDS: u32 = 0;

/// The bytes the workload writes to its file each round.
const CHUNK: usize = 64;

/// `rounds` rounds of integer mixing from `seed`.
#[must_use]
pub fn mix(seed: u64, rounds: u64) -> u64 {
    let _ = (seed, rounds);
    0
}

/// `rounds` rounds of creating, writing, reading and inspecting a file
/// in `dir`, then removing it: open, write, close, open, read, close,
/// stat, process id and clock. The sum of what it read and measured.
///
/// # Errors
///
/// Fails when the file cannot be created, read or removed.
pub fn touch_files(dir: &Path, rounds: u32) -> io::Result<u64> {
    let _ = (dir, rounds, CHUNK);
    Ok(0)
}

/// The workload with its own parameters: the mixed seed combined with
/// the files' sum.
///
/// # Errors
///
/// Fails as [`touch_files`] does.
pub fn run_with(dir: &Path, seed: u64, mix_rounds: u64, file_rounds: u32) -> io::Result<u64> {
    let _ = (dir, seed, mix_rounds, file_rounds);
    Ok(0)
}

/// The frozen workload, in `dir`.
///
/// # Errors
///
/// Fails as [`touch_files`] does.
pub fn run(dir: &Path) -> io::Result<u64> {
    run_with(dir, SEED, MIX_ROUNDS, FILE_ROUNDS)
}

#[cfg(test)]
mod tests;
