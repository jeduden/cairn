//! The `protection-check` command line: the committed ruleset and the
//! live one in; the ways the live one is weaker out.

use std::io::{self, Write};

use super::{parse, weaker_than};

/// The live ruleset is as strong as the committed one.
pub const EXIT_OK: u8 = 0;
/// The live ruleset is weaker, or a file could not be read.
pub const EXIT_FAILURE: u8 = 1;
/// The command line is wrong.
pub const EXIT_USAGE: u8 = 2;

const USAGE: &str = "usage: protection-check --committed FILE --live FILE";

/// Runs the command on `args` and returns its exit code. It is `main`
/// without the process: `read` maps a path to the file's text.
pub fn run(
    args: impl IntoIterator<Item = String>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    read: &dyn Fn(&str) -> io::Result<String>,
) -> u8 {
    let (committed, live) = match flags(args) {
        Ok(files) => files,
        Err(msg) => {
            let _ = writeln!(stderr, "protection-check: {msg}\n{USAGE}");
            return EXIT_USAGE;
        }
    };
    match compare(&committed, &live, read) {
        Err(msg) => {
            let _ = writeln!(stderr, "protection-check: {msg}");
            EXIT_FAILURE
        }
        Ok(problems) if problems.is_empty() => {
            let _ = writeln!(stdout, "the live ruleset is as strong as the committed one");
            EXIT_OK
        }
        Ok(problems) => {
            for p in problems {
                let _ = writeln!(stderr, "protection-check: {p}");
            }
            EXIT_FAILURE
        }
    }
}

/// Reads both rulesets and lists where the live one is weaker.
fn compare(
    committed: &str,
    live: &str,
    read: &dyn Fn(&str) -> io::Result<String>,
) -> Result<Vec<String>, String> {
    let load = |path: &str| {
        read(path)
            .map_err(|e| format!("read {path}: {e}"))
            .and_then(|text| parse(&text).map_err(|e| format!("{path}: {e}")))
    };

    Ok(weaker_than(&load(committed)?, &load(live)?))
}

/// The two files, from `--committed FILE --live FILE` in either order.
fn flags(args: impl IntoIterator<Item = String>) -> Result<(String, String), String> {
    let (mut committed, mut live) = (None, None);
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let slot = match arg.as_str() {
            "--committed" => &mut committed,
            "--live" => &mut live,
            _ => return Err(format!("unexpected argument {arg:?}")),
        };
        *slot = Some(
            args.next()
                .filter(|v| !v.is_empty())
                .ok_or_else(|| format!("{arg} needs a value"))?,
        );
    }

    Ok((
        committed.ok_or("--committed is required")?,
        live.ok_or("--live is required")?,
    ))
}

#[cfg(test)]
mod tests;
