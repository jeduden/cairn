//! ENG-12: the end-to-end suite runs each component's entry point
//! confined to its boundary. The testkit's `boundary` module confines a
//! start; this holds every test of a product crate to it. A test that
//! names a component's executable, `CARGO_BIN_EXE_<name>`, must start
//! it through the confinement and name no `Command` of its own: a start
//! outside it would run with the network open and its sockets unseen.

use std::fs;
use std::path::{Path, PathBuf};

use crate::Checkout;
use crate::product_crates::crates;

/// What a test names to start a component's executable.
const EXECUTABLE: &str = "CARGO_BIN_EXE_";

/// What a confined start names.
const CONFINED: &str = "Confinement";

/// What an unconfined start names.
const COMMAND: &str = "Command";

/// Whether a test file's `body` starts a component's executable outside
/// the confinement: it names an executable and a `Command`, or names an
/// executable but not the confinement. Comment lines do not count.
#[must_use]
pub fn starts_unconfined(body: &str) -> bool {
    let code: Vec<&str> = body
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect();
    let names = |word: &str| code.iter().any(|l| l.contains(word));

    names(EXECUTABLE) && (names(COMMAND) || !names(CONFINED))
}

/// Requires every `.rs` file under each product crate's `tests/` that
/// starts a component's executable to start it confined.
#[must_use]
pub fn unconfined_starts(c: &Checkout) -> Vec<String> {
    let names = match crates(c) {
        Ok(names) => names,
        Err(e) => return vec![e],
    };
    let mut files = Vec::new();
    for name in names {
        rust_files(&c.root.join("crates").join(name).join("tests"), &mut files);
    }
    let mut problems = Vec::new();
    for path in files {
        let rel = path
            .strip_prefix(&c.root)
            .unwrap_or(&path)
            .display()
            .to_string();
        match fs::read_to_string(&path) {
            Err(e) => problems.push(format!("read {rel}: {e}")),
            Ok(body) if starts_unconfined(&body) => problems.push(format!(
                "{rel} starts a component's executable outside the end-to-end suite's confinement (ENG-12)"
            )),
            Ok(_) => {}
        }
    }

    problems
}

/// Collects every `.rs` file under `dir`, sorted; a directory that
/// cannot be listed, such as a crate without tests, holds none.
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|x| x == "rs") {
            out.push(path);
        }
    }
}

#[cfg(test)]
mod tests;
