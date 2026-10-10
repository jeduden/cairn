//! The `review-gate` command line: four flags in; a review, nothing or
//! an error out.

use std::io::{self, Write};

use crate::{Input, decide, parse_check_runs, parse_outcome};

/// The review was printed, or there is none to post.
pub const EXIT_OK: u8 = 0;
/// The gate could not decide; nothing may be posted.
pub const EXIT_FAILURE: u8 = 1;
/// The command line is wrong.
pub const EXIT_USAGE: u8 = 2;

const USAGE: &str = "usage: review-gate --outcome FILE --check-runs FILE --reviewed SHA --head SHA";

/// The flags, in the order the usage names them.
const FLAGS: [&str; 4] = ["outcome", "check-runs", "reviewed", "head"];

/// Runs the command on `args` and returns its exit code. It is `main`
/// without the process: `read` maps a path to the file's text, and
/// every test drives the command through here.
pub fn run(
    args: impl IntoIterator<Item = String>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    read: &dyn Fn(&str) -> io::Result<String>,
) -> u8 {
    let [outcome, check_runs, reviewed, head] = match parse_flags(args) {
        Ok(values) => values,
        Err(msg) => {
            let _ = writeln!(stderr, "review-gate: {msg}\n{USAGE}");
            return EXIT_USAGE;
        }
    };
    let input = read_input(&outcome, &check_runs, read).map(|i| Input {
        reviewed,
        head,
        ..i
    });
    let decided = input.and_then(|input| {
        decide(&input)
            .map_err(|e| e.to_string())
            .map(|r| (r, input.reviewed))
    });

    match decided {
        Err(msg) => {
            let _ = writeln!(stderr, "review-gate: {msg}");
            EXIT_FAILURE
        }
        Ok((None, reviewed)) => {
            let _ = writeln!(
                stderr,
                "review-gate: the pull request moved past {reviewed}; its newer run reviews it"
            );
            EXIT_OK
        }
        Ok((Some(review), _)) => {
            match serde_json::to_writer(&mut *stdout, &review)
                .and_then(|()| writeln!(stdout).map_err(serde_json::Error::io))
            {
                Ok(()) => EXIT_OK,
                Err(err) => {
                    let _ = writeln!(stderr, "review-gate: {err}");
                    EXIT_FAILURE
                }
            }
        }
    }
}

/// Reads `--name value` and `--name=value` pairs into the values of
/// [`FLAGS`], each required.
fn parse_flags(args: impl IntoIterator<Item = String>) -> Result<[String; 4], String> {
    let mut values: [Option<String>; 4] = Default::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let Some(flag) = arg.strip_prefix("--") else {
            return Err(format!("unexpected argument {arg:?}"));
        };
        let (name, inline) = match flag.split_once('=') {
            Some((name, value)) => (name, Some(value.to_owned())),
            None => (flag, None),
        };
        let slot = FLAGS
            .iter()
            .position(|f| *f == name)
            .ok_or_else(|| format!("unknown flag {arg:?}"))?;
        let value = inline
            .or_else(|| args.next())
            .ok_or_else(|| format!("--{name} needs a value"))?;
        values[slot] = Some(value);
    }

    let mut out: [String; 4] = Default::default();
    for (i, value) in values.into_iter().enumerate() {
        out[i] = value.ok_or_else(|| format!("--{} is required", FLAGS[i]))?;
    }

    Ok(out)
}

/// Loads the review outcome and the check runs.
fn read_input(
    outcome: &str,
    check_runs: &str,
    read: &dyn Fn(&str) -> io::Result<String>,
) -> Result<Input, String> {
    let load = |path: &str| read(path).map_err(|e| format!("read {path}: {e}"));
    let outcome = parse_outcome(&load(outcome)?).map_err(|e| e.to_string())?;
    let check_runs = parse_check_runs(&load(check_runs)?).map_err(|e| e.to_string())?;

    Ok(Input {
        outcome,
        check_runs,
        ..Input::default()
    })
}

#[cfg(test)]
mod tests;
