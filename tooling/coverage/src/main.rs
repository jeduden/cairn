//! The `coverage` command: line coverage per test layer and per crate,
//! held to the floors in `[workspace.metadata.coverage]`. CI's coverage
//! job runs it; `cargo run -p coverage` runs it locally, with
//! cargo-llvm-cov installed.

use std::env;
use std::io;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let summary = env::var_os("GITHUB_STEP_SUMMARY").map(PathBuf::from);
    let host = coverage::Cargo::from_env();

    ExitCode::from(coverage::cli::run(
        env::args().skip(1),
        &mut io::stdout(),
        &mut io::stderr(),
        &host,
        summary.as_deref(),
    ))
}
