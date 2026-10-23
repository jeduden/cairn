//! The `review-bundle` command line: two trees, a diff and the
//! stakeholder's comments in; the review bundle and the deterministic
//! findings written to a directory, and the review class printed for the
//! workflow's step outputs.
//!
//! `--consent` names a JSON array of the comments the CODEOWNERS owners
//! wrote on the pull request, each `{"author", "at", "body"}` and, for a
//! review, its `state`, which the workflow filters before the command
//! reads it. One with a non-blank body, other than a review that
//! requests changes, is the stakeholder's consent ENG-29 asks for.

use std::fs;
use std::io::Write;
use std::path::Path;

use serde_json::Value;

use crate::approvals::{self, Verdict};
use crate::{Snapshot, analyse, render};

/// The review bundle and the findings were written.
pub const EXIT_OK: u8 = 0;
/// A tree, the diff, the consent or an output could not be read or
/// written.
pub const EXIT_FAILURE: u8 = 1;
/// The command line is wrong.
pub const EXIT_USAGE: u8 = 2;

const USAGE: &str =
    "usage: review-bundle --base DIR --head DIR --diff FILE --out DIR --consent FILE";

/// The flags, in the order the usage names them, each required.
const FLAGS: [&str; 5] = ["base", "head", "diff", "out", "consent"];

/// The state of a review that requests changes, as GitHub names it.
const CHANGES_REQUESTED: &str = "CHANGES_REQUESTED";

/// The review bundle's file name under `--out`.
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
    let mut args = args.into_iter().peekable();
    if args.next_if(|a| a == "approvals").is_some() {
        return run_approvals(args, stdout, stderr);
    }
    let [base, head, diff, out, consent] = match parse_flags(args) {
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
        Path::new(&consent),
    ) {
        Ok(outputs) => {
            let _ = write!(stdout, "{outputs}");
            EXIT_OK
        }
        Err(msg) => {
            let _ = writeln!(stderr, "{msg}");
            EXIT_FAILURE
        }
    }
}

const APPROVALS_USAGE: &str = "usage: review-bundle approvals --codeowners FILE --files FILE --reviews FILE --author LOGIN --head SHA";

/// The flags of `approvals`, in the order the usage names them.
const APPROVALS_FLAGS: [&str; 5] = ["codeowners", "files", "reviews", "author", "head"];

/// The `approvals` subcommand (ENG-21): prints `ok` or `not-security`
/// when the pull request has the approvals its files need, and exits
/// [`EXIT_FAILURE`] with the shortfall on stderr when it has not.
fn run_approvals(
    args: impl Iterator<Item = String>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let [codeowners, files, reviews, author, head] = match parse_named(args, &APPROVALS_FLAGS) {
        Ok(values) => values,
        Err(msg) => {
            let _ = writeln!(stderr, "review-bundle: {msg}\n{APPROVALS_USAGE}");
            return EXIT_USAGE;
        }
    };
    let read = |p: &str| fs::read_to_string(p).map_err(failed("read", Path::new(p)));
    let verdict = read(&codeowners).and_then(|owners| {
        let files =
            approvals::parse_files(&read(&files)?).map_err(failed("parse", Path::new(&files)))?;
        let reviews = approvals::parse_reviews(&read(&reviews)?)
            .map_err(failed("parse", Path::new(&reviews)))?;
        approvals::check(&owners, &files, &author, &head, &reviews)
    });
    match verdict {
        Ok(Verdict::NotSecurity) => {
            let _ = writeln!(stdout, "not-security");
            EXIT_OK
        }
        Ok(Verdict::Approved) => {
            let _ = writeln!(stdout, "ok");
            EXIT_OK
        }
        Err(msg) => {
            let _ = writeln!(stderr, "review-bundle: {msg}");
            EXIT_FAILURE
        }
    }
}

/// Reads both trees, the diff and the consent, writes the review bundle
/// and the findings under `out`, and returns the step outputs to print.
fn build(
    base: &Path,
    head: &Path,
    diff: &Path,
    out: &Path,
    consent: &Path,
) -> Result<String, String> {
    let base_tree = Snapshot::read(base).map_err(|e| format!("{e} (main's tree)"))?;
    let head_tree = Snapshot::read(head).map_err(|e| format!("{e} (the pull request's tree)"))?;
    let read = |path: &Path| fs::read_to_string(path).map_err(failed("read", path));
    let diff = read(diff)?;
    let consent = consents(&read(consent)?).map_err(failed("read", consent))?;
    let a = analyse(&base_tree, &head_tree, &diff, consent);
    let write = |name: &str, body: &str| {
        let path = out.join(name);
        fs::write(&path, body).map_err(failed("write", &path))
    };
    write(BUNDLE, &render::bundle(&a, &base_tree, &head_tree))?;
    let findings = serde_json::to_string_pretty(&a.findings).map_err(failed("encode", out))?;
    write(CHECKED, &format!("{findings}\n"))?;

    Ok(format!(
        "class={}\ntouched={}\nfindings={}\n",
        a.class.name(),
        a.touched.len(),
        a.findings.len()
    ))
}

/// The message for a failure to `verb` the file at `path`.
fn failed<E: std::fmt::Display>(verb: &'static str, path: &Path) -> impl FnOnce(E) -> String {
    let path = path.display().to_string();
    move |e| format!("review-bundle: {verb} {path}: {e}")
}

/// Whether `json`, the owners' comments, holds the stakeholder's
/// consent: at least one comment with a non-blank body that is no
/// review requesting changes. Each comment must carry a string
/// `author`, `at` and `body`.
fn consents(json: &str) -> Result<bool, String> {
    let value: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let comments = value.as_array().ok_or("not an array of comments")?;
    let mut consent = false;
    for comment in comments {
        let field = |name: &str| {
            comment
                .get(name)
                .and_then(Value::as_str)
                .ok_or_else(|| format!("a comment without a string {name:?}"))
        };
        field("author")?;
        field("at")?;
        let objects = comment.get("state").and_then(Value::as_str) == Some(CHANGES_REQUESTED);
        consent |= !objects && !field("body")?.trim().is_empty();
    }

    Ok(consent)
}

/// Reads `--name value` and `--name=value` pairs into the values of
/// [`FLAGS`], each required.
fn parse_flags(args: impl IntoIterator<Item = String>) -> Result<[String; 5], String> {
    parse_named(args, &FLAGS)
}

/// Reads `--name value` and `--name=value` pairs into the values of
/// `names`, each required.
fn parse_named(
    args: impl IntoIterator<Item = String>,
    names: &[&str; 5],
) -> Result<[String; 5], String> {
    let mut values: [Option<String>; 5] = Default::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let Some(flag) = arg.strip_prefix("--") else {
            return Err(format!("unexpected argument {arg:?}"));
        };
        let (name, inline) = match flag.split_once('=') {
            Some((name, value)) => (name, Some(value.to_owned())),
            None => (flag, None),
        };
        let slot = names
            .iter()
            .position(|f| *f == name)
            .ok_or_else(|| format!("unknown flag {arg:?}"))?;
        let value = inline
            .or_else(|| args.next())
            .filter(|v| !v.is_empty())
            .ok_or_else(|| format!("--{name} needs a value"))?;
        values[slot] = Some(value);
    }

    let mut out: [String; 5] = Default::default();
    for (i, value) in values.into_iter().enumerate() {
        out[i] = value.ok_or_else(|| format!("--{} is required", names[i]))?;
    }

    Ok(out)
}

#[cfg(test)]
mod tests;
