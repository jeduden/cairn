//! The `review-bundle` command line: two trees and a diff in; the
//! bundle and the deterministic findings written to a directory, and
//! the route printed for the workflow's step outputs.

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::{Snapshot, analyse, render};

/// The bundle and the findings were written.
pub const EXIT_OK: u8 = 0;
/// A tree, the diff or an output could not be read or written.
pub const EXIT_FAILURE: u8 = 1;
/// The command line is wrong.
pub const EXIT_USAGE: u8 = 2;

const USAGE: &str = "usage: review-bundle --base DIR --head DIR --diff FILE --out DIR";

/// The flags, in the order the usage names them, each required.
const FLAGS: [&str; 4] = ["base", "head", "diff", "out"];

/// The bundle's file name under `--out`.
pub const BUNDLE: &str = "bundle.md";
/// The deterministic findings' file name under `--out`.
pub const CHECKED: &str = "checked.json";

/// Runs the command on `args` and returns its exit code. It is `main`
/// without the process.
pub fn run(
    args: impl IntoIterator<Item = String>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let [base, head, diff, out] = match parse_flags(args) {
        Ok(values) => values,
        Err(msg) => {
            let _ = writeln!(stderr, "review-bundle: {msg}\n{USAGE}");
            return EXIT_USAGE;
        }
    };
    match build(
        Path::new(&base),
        Path::new(&head),
        Path::new(&diff),
        Path::new(&out),
    ) {
        Ok(route) => {
            let _ = write!(stdout, "{route}");
            EXIT_OK
        }
        Err(msg) => {
            let _ = writeln!(stderr, "{msg}");
            EXIT_FAILURE
        }
    }
}

/// Reads both trees and the diff, writes the bundle and the findings
/// under `out`, and returns the step outputs to print.
fn build(base: &Path, head: &Path, diff: &Path, out: &Path) -> Result<String, String> {
    let base_tree = Snapshot::read(base).map_err(|e| format!("{e} (main's tree)"))?;
    let head_tree = Snapshot::read(head).map_err(|e| format!("{e} (the pull request's tree)"))?;
    let diff = fs::read_to_string(diff)
        .map_err(|e| format!("review-bundle: read {}: {e}", diff.display()))?;
    let a = analyse(&base_tree, &head_tree, &diff);
    let findings = serde_json::to_string_pretty(&a.findings)
        .map_err(|e| format!("review-bundle: encode findings: {e}"))?;
    let write = |name: &str, body: &str| {
        let path = out.join(name);
        fs::write(&path, body).map_err(|e| format!("review-bundle: write {}: {e}", path.display()))
    };
    write(BUNDLE, &render::bundle(&a, &base_tree, &head_tree))?;
    write(CHECKED, &format!("{findings}\n"))?;

    Ok(format!(
        "class={}\ntouched={}\nfindings={}\n",
        a.class.name(),
        a.touched.len(),
        a.findings.len()
    ))
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
            .filter(|v| !v.is_empty())
            .ok_or_else(|| format!("--{name} needs a value"))?;
        values[slot] = Some(value);
    }

    let mut out: [String; 4] = Default::default();
    for (i, value) in values.into_iter().enumerate() {
        out[i] = value.ok_or_else(|| format!("--{} is required", FLAGS[i]))?;
    }

    Ok(out)
}

#[cfg(test)]
mod tests;
