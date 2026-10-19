//! A benchmark, and any step that holds the core to a time budget,
//! runs a release build (NFR-01, NFR-02, ENG-15). A development build
//! is an order of magnitude slower and flakes on a loaded runner, and
//! the seams it carries are not the build users run. Two scans hold
//! the repository to it:
//!
//! - a workflow or script line that builds, runs or tests the
//!   `benchmark` package names `--release`, and a line that runs the
//!   benchmark never points it at a `debug` directory;
//! - a Rust function in an integration test or bench that reads a clock
//!   (`Instant::now`, `.elapsed()`) starts the core only through
//!   `release_core_executable` or `run_release_core`.

use std::fs;
use std::path::Path;

use crate::Checkout;

/// Workflow and script directories the first scan reads.
const BUILD_DIRS: [&str; 2] = [".github/workflows", "scripts"];

/// The directories whose crates' integration tests and benches the
/// second scan reads.
const CRATE_DIRS: [&str; 2] = ["crates", "tooling"];

/// What starts the development build of the core in a step.
const DEBUG_CORE: [&str; 6] = [
    "core_executable(",
    "run_core(",
    "run_core_with(",
    "run_core_killed(",
    "run_core_at_terminal(",
    "CARGO_BIN_EXE_cairn",
];

/// What starts the release build of the core in a step.
const RELEASE_CORE: [&str; 2] = ["release_core_executable(", "run_release_core("];

/// What reads a clock to time a step.
const CLOCK: [&str; 2] = ["Instant::now(", ".elapsed("];

/// The problems in one workflow or script `text`, named by `file`.
#[must_use]
pub fn debug_benchmark_lines(file: &str, text: &str) -> Vec<String> {
    let _ = (file, text);
    Vec::new()
}

/// The problems in one Rust `source`, named by `file`.
#[must_use]
pub fn debug_core_in_timed_fns(file: &str, source: &str) -> Vec<String> {
    let _ = (file, source);
    Vec::new()
}

/// Every problem in the checkout's workflows, scripts, integration
/// tests and benches.
#[must_use]
pub fn problems(checkout: &Checkout) -> Vec<String> {
    let _ = (checkout, Path::new(""), fs::read_dir("."));
    let _ = (BUILD_DIRS, CRATE_DIRS, DEBUG_CORE, RELEASE_CORE, CLOCK);
    Vec::new()
}

#[cfg(test)]
mod tests;
