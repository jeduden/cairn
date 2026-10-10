//! The `coverage` command line: measure, print the table, and fail on a
//! crate below its floor.

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::Host;
use crate::lcov;
use crate::measure::measure;
use crate::plan::Plan;
use crate::report::{shortfalls, table, tally};

/// Every crate meets its floors.
pub const EXIT_OK: u8 = 0;
/// A crate is below a floor, or the measurement failed.
pub const EXIT_FAILURE: u8 = 1;
/// The command line is wrong.
pub const EXIT_USAGE: u8 = 2;

const USAGE: &str = "usage: coverage [--out DIR]

Runs cargo-llvm-cov once per test layer (unit, integration, end-to-end),
prints line coverage per crate and layer, writes each layer's lcov to
DIR (target/coverage by default), and fails when a crate is below a
floor in [workspace.metadata.coverage].";

/// Runs the command on `args` and returns its exit code. It is `main`
/// without the process: `host` runs cargo and writes files, and
/// `summary`, when set, is a Markdown file the table is appended to,
/// as GitHub's `$GITHUB_STEP_SUMMARY` is.
pub fn run(
    args: impl IntoIterator<Item = String>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    host: &dyn Host,
    summary: Option<&Path>,
) -> u8 {
    let out = match parse_args(args) {
        Ok(Some(out)) => out,
        Ok(None) => {
            let _ = writeln!(stdout, "{USAGE}");
            return EXIT_OK;
        }
        Err(msg) => {
            let _ = writeln!(stderr, "coverage: {msg}\n{USAGE}");
            return EXIT_USAGE;
        }
    };

    match report(host, &out, summary) {
        Ok((table, shortfalls)) => {
            let _ = write!(stdout, "{table}");
            for s in &shortfalls {
                let _ = writeln!(stderr, "{s}");
            }
            if shortfalls.is_empty() {
                EXIT_OK
            } else {
                EXIT_FAILURE
            }
        }
        Err(msg) => {
            let _ = writeln!(stderr, "coverage: {msg}");
            EXIT_FAILURE
        }
    }
}

/// Reads `--out DIR`; `None` asks for the usage.
fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Option<PathBuf>, String> {
    let mut out = PathBuf::from("target/coverage");
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(None),
            "--out" => {
                out = args
                    .next()
                    .map(PathBuf::from)
                    .ok_or("--out needs a directory")?
            }
            _ => match arg.strip_prefix("--out=") {
                Some(dir) => out = PathBuf::from(dir),
                None => return Err(format!("unexpected argument {arg:?}")),
            },
        }
    }

    Ok(Some(out))
}

/// Measures, and returns the table and the shortfalls.
fn report(
    host: &dyn Host,
    out: &Path,
    summary: Option<&Path>,
) -> Result<(String, Vec<String>), String> {
    let metadata = host.cargo_output(
        &["metadata", "--format-version", "1", "--no-deps", "--locked"].map(str::to_owned),
    )?;
    let plan = Plan::from_metadata(&metadata)?;
    let layers = measure(host, &plan, out)?;
    let all = lcov::merge(layers.iter().map(|(_, lines)| lines));
    let rows = tally(&plan, &layers, &all);
    let table = table(&rows);
    if let Some(summary) = summary {
        host.append(
            summary,
            &format!("## Line coverage per test layer\n\n{table}\n"),
        )?;
    }

    Ok((table, shortfalls(&plan, &rows)))
}

#[cfg(test)]
mod tests;
