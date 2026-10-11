//! The product crates under `crates/` keep two promises a compiler can
//! hold: the core's reach (I4, SEC-01) and derived artifacts that read
//! no clock or random order (I10, ENG-03). Every crate there serves the
//! core today; a B1 to B3 component's crate weighs its own reach when
//! its ADR lands (ENG-29), and this gate changes with it. Each crate's clippy.toml
//! carries the bans, so clippy refuses a socket, a program start, a
//! clock read or a hash map's random order; a call site that needs one
//! says so with an `expect` and a reason, under review. And no crate
//! keeps mutable global state (ENG-03), which no lint refuses, so this
//! reads the source for it.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::Checkout;

/// The types the core may not name (I4, SEC-01): sockets and program
/// starts.
pub const REACH_BANS: [&str; 7] = [
    "std::net::TcpListener",
    "std::net::TcpStream",
    "std::net::UdpSocket",
    "std::os::unix::net::UnixDatagram",
    "std::os::unix::net::UnixListener",
    "std::os::unix::net::UnixStream",
    "std::process::Command",
];

/// The calls and types derived artifacts may not reach (I10, ENG-03):
/// the clocks, and the hash collections whose order is random per
/// process.
pub const DETERMINISM_BANS: [&str; 5] = [
    "std::time::SystemTime::now",
    "std::time::Instant::now",
    "std::collections::HashMap",
    "std::collections::HashSet",
    "std::hash::RandomState",
];

/// Where the product's crates live.
const CRATES: &str = "crates";

/// The interior-mutable types a global may not hold (ENG-03). A
/// `LazyLock` or `OnceLock` that is set once and only read is no
/// mutable state.
const MUTABLE: [&str; 7] = [
    "Mutex",
    "RwLock",
    "Cell<",
    "RefCell",
    "UnsafeCell",
    "Atomic",
    "mut ",
];

/// Every `path = "..."` a clippy.toml names.
#[must_use]
pub fn banned(clippy_toml: &str) -> BTreeSet<String> {
    clippy_toml
        .lines()
        .flat_map(|l| l.split("path").skip(1))
        .filter_map(|rest| {
            let rest = rest.trim_start().strip_prefix('=')?.trim_start();
            let rest = rest.strip_prefix('"')?;
            rest.split_once('"').map(|(p, _)| p.to_owned())
        })
        .collect()
}

/// The bans `clippy_toml` lacks, of [`REACH_BANS`] and
/// [`DETERMINISM_BANS`].
#[must_use]
pub fn missing_bans(clippy_toml: &str) -> Vec<&'static str> {
    let have = banned(clippy_toml);
    REACH_BANS
        .iter()
        .chain(&DETERMINISM_BANS)
        .filter(|b| !have.contains(**b))
        .copied()
        .collect()
}

/// Each global in `body` that holds mutable state: a `static` whose
/// type is interior-mutable or `mut`, or a `thread_local!`. `file`
/// names the place.
#[must_use]
pub fn mutable_globals(file: &str, body: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (i, line) in body.lines().enumerate() {
        let code = line.trim_start();
        let decl = ["pub(crate) static ", "pub static ", "static "]
            .iter()
            .find_map(|p| code.strip_prefix(p));
        let held = decl.map(|rest| {
            let end = rest.find(['=', ';']).unwrap_or(rest.len());
            &rest[..end]
        });
        let mutable = held.is_some_and(|ty| MUTABLE.iter().any(|m| ty.contains(m)));
        if mutable || code.starts_with("thread_local!") {
            out.push(format!("{file}:{}: mutable global state: {}", i + 1, code));
        }
    }

    out
}

/// Every product crate's directory name under `crates/`, sorted.
fn crates(c: &Checkout) -> Result<Vec<String>, String> {
    let dir = c.root.join(CRATES);
    let entries = fs::read_dir(&dir).map_err(|e| format!("list {CRATES}: {e}"))?;
    let mut out = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("list {CRATES}: {e}"))?;
        if entry.path().join("Cargo.toml").is_file() {
            out.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    out.sort();

    Ok(out)
}

/// Requires every product crate's clippy.toml, or the workspace's when
/// it has none, to carry every ban.
#[must_use]
pub fn crates_keep_their_bans(c: &Checkout) -> Vec<String> {
    let names = match crates(c) {
        Ok(names) => names,
        Err(e) => return vec![e],
    };
    let mut problems = Vec::new();
    for name in names {
        let own = format!("{CRATES}/{name}/clippy.toml");
        let file = if c.root.join(&own).is_file() {
            own
        } else {
            "clippy.toml".to_owned()
        };
        match c.read(&file) {
            Err(e) => problems.push(e),
            Ok(body) => problems.extend(
                missing_bans(&body)
                    .into_iter()
                    .map(|b| format!("crate {name}: {file} does not ban {b}")),
            ),
        }
    }

    problems
}

/// Requires no product crate's shipped code, its unit-test files and
/// test targets left out, to keep mutable global state.
#[must_use]
pub fn no_mutable_globals(c: &Checkout) -> Vec<String> {
    let names = match crates(c) {
        Ok(names) => names,
        Err(e) => return vec![e],
    };
    let mut problems = Vec::new();
    for name in names {
        let src = c.root.join(CRATES).join(&name).join("src");
        let mut files = Vec::new();
        if let Err(e) = sources(&src, &mut files) {
            problems.push(e);
        }
        for path in files {
            let rel = path
                .strip_prefix(&c.root)
                .unwrap_or(&path)
                .display()
                .to_string();
            match fs::read_to_string(&path) {
                Err(e) => problems.push(format!("read {rel}: {e}")),
                Ok(body) => problems.extend(mutable_globals(&rel, &body)),
            }
        }
    }

    problems
}

/// Collects every `.rs` file under `dir` but `tests.rs`, sorted.
fn sources(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<(), String> {
    let list = |e: std::io::Error| format!("list {}: {e}", dir.display());
    let mut paths = fs::read_dir(dir)
        .map_err(list)?
        .map(|e| e.map(|e| e.path()).map_err(list))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    for path in paths {
        if path.is_dir() {
            sources(&path, out)?;
        } else if path.extension().is_some_and(|x| x == "rs")
            && path.file_name().is_some_and(|n| n != "tests.rs")
        {
            out.push(path);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
