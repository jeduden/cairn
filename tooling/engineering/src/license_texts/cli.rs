//! The `license-check` command line: ENG-30's check on a checkout, run
//! by CI on every pull request. It prints each problem on its own line
//! and exits non-zero when there is any.

use std::io::Write;
use std::path::Path;

/// Every crate a release links has a license text that fits.
pub const EXIT_OK: u8 = 0;
/// A crate lacks a fitting license text, or the check could not run.
pub const EXIT_FAILURE: u8 = 1;
/// The command line is wrong.
pub const EXIT_USAGE: u8 = 2;

const USAGE: &str = "usage: license-check [--root DIR]";

/// Runs the command on `args` and returns its exit code. It is `main`
/// without the process: `check` runs ENG-30's check on a checkout root.
pub fn run(
    args: impl IntoIterator<Item = String>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    check: &dyn Fn(&Path) -> Result<Vec<String>, String>,
) -> u8 {
    let root = match root(args) {
        Ok(root) => root,
        Err(msg) => {
            let _ = writeln!(stderr, "license-check: {msg}\n{USAGE}");
            return EXIT_USAGE;
        }
    };
    match check(Path::new(&root)) {
        Err(msg) => {
            let _ = writeln!(stderr, "license-check: {msg}");
            EXIT_FAILURE
        }
        Ok(problems) if problems.is_empty() => {
            let _ = writeln!(stdout, "every crate a release links has a license text");
            EXIT_OK
        }
        Ok(problems) => {
            for p in problems {
                let _ = writeln!(stderr, "license-check: {p}");
            }
            EXIT_FAILURE
        }
    }
}

/// The checkout root, from `--root DIR`, the current directory when
/// absent.
fn root(args: impl IntoIterator<Item = String>) -> Result<String, String> {
    let mut root = ".".to_owned();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if arg != "--root" {
            return Err(format!("unexpected argument {arg:?}"));
        }
        root = args
            .next()
            .filter(|v| !v.is_empty())
            .ok_or("--root needs a value")?;
    }

    Ok(root)
}

#[cfg(test)]
mod tests;
