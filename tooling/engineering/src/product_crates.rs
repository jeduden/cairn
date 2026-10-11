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

use std::collections::{BTreeMap, BTreeSet};
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

/// The types derived artifacts may not name (I10, ENG-03): the hash
/// collections whose order is random per process.
pub const DETERMINISM_TYPES: [&str; 3] = [
    "std::collections::HashMap",
    "std::collections::HashSet",
    "std::hash::RandomState",
];

/// The calls derived artifacts may not make (I10, ENG-03): the clocks.
pub const DETERMINISM_METHODS: [&str; 2] =
    ["std::time::SystemTime::now", "std::time::Instant::now"];

/// The clippy.toml table that bans a type.
pub const TYPES: &str = "disallowed-types";

/// The clippy.toml table that bans a call.
pub const METHODS: &str = "disallowed-methods";

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

/// `line` up to its comment: a `#` outside a double-quoted string.
fn strip_comment(line: &str) -> &str {
    let (mut quoted, mut escaped) = (false, false);
    for (i, ch) in line.char_indices() {
        match ch {
            _ if escaped => escaped = false,
            '\\' if quoted => escaped = true,
            '"' => quoted = !quoted,
            '#' if !quoted => return &line[..i],
            _ => {}
        }
    }

    line
}

/// Every `path = "..."` in one array's `body`.
fn paths(body: &str) -> impl Iterator<Item = String> + '_ {
    body.split("path").skip(1).filter_map(|rest| {
        let rest = rest.trim_start().strip_prefix('=')?.trim_start();
        let rest = rest.strip_prefix('"')?;
        rest.split_once('"').map(|(p, _)| p.to_owned())
    })
}

/// Every `path = "..."` a clippy.toml names, by the array it sits in:
/// `key = [ ... ]`, on one line or many. Comments do not count, and an
/// array never closed names nothing.
#[must_use]
pub fn banned(clippy_toml: &str) -> BTreeMap<String, BTreeSet<String>> {
    let text = clippy_toml
        .lines()
        .map(strip_comment)
        .collect::<Vec<_>>()
        .join("\n");
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let (mut key, mut body) = (String::new(), String::new());
    let (mut depth, mut quoted, mut escaped) = (0_usize, false, false);
    for ch in text.chars() {
        let into = if depth == 0 { &mut key } else { &mut body };
        if quoted {
            into.push(ch);
            match ch {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => quoted = false,
                _ => {}
            }
            continue;
        }
        match ch {
            '"' => {
                quoted = true;
                into.push(ch);
            }
            '[' => {
                depth += 1;
                body.push(ch);
            }
            ']' if depth > 0 => {
                depth -= 1;
                body.push(ch);
                if depth == 0 {
                    let name = key.trim().trim_end_matches('=').trim().to_owned();
                    out.entry(name).or_default().extend(paths(&body));
                    key.clear();
                    body.clear();
                }
            }
            '\n' if depth == 0 => key.clear(),
            _ => into.push(ch),
        }
    }

    out
}

/// The bans `clippy_toml` lacks, each with the table it belongs in:
/// [`REACH_BANS`] and [`DETERMINISM_TYPES`] in [`TYPES`],
/// [`DETERMINISM_METHODS`] in [`METHODS`].
#[must_use]
pub fn missing_bans(clippy_toml: &str) -> Vec<(&'static str, &'static str)> {
    let have = banned(clippy_toml);
    let types = REACH_BANS
        .iter()
        .chain(&DETERMINISM_TYPES)
        .map(|b| (TYPES, *b));
    let methods = DETERMINISM_METHODS.iter().map(|b| (METHODS, *b));
    types
        .chain(methods)
        .filter(|(table, b)| !have.get(*table).is_some_and(|set| set.contains(*b)))
        .collect()
}

/// What follows `static ` on a line that declares one, under any
/// visibility: `pub`, `pub(crate)`, `pub(super)`, `pub(in path)`.
fn after_static(code: &str) -> Option<&str> {
    let rest = match code.strip_prefix("pub(") {
        Some(scoped) => scoped.split_once(')')?.1.trim_start(),
        None => code.strip_prefix("pub ").unwrap_or(code).trim_start(),
    };
    rest.strip_prefix("static ")
}

/// The type a `static` holds: from `first` on, across `next` lines,
/// up to the first `=` or `;`.
fn held<'a>(first: &'a str, next: impl Iterator<Item = &'a str>) -> String {
    let mut ty = String::new();
    for part in std::iter::once(first).chain(next) {
        if let Some(end) = part.find(['=', ';']) {
            ty.push_str(&part[..end]);
            break;
        }
        ty.push_str(part);
        ty.push('\n');
    }

    ty
}

/// Each global in `body` that holds mutable state: a `static` whose
/// type is interior-mutable or `mut`, read across lines to its `=` or
/// `;`, or a `thread_local!`. `file` names the place, and each finding
/// the line its declaration starts on.
#[must_use]
pub fn mutable_globals(file: &str, body: &str) -> Vec<String> {
    let lines: Vec<&str> = body.lines().collect();
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let code = line.trim_start();
        let held = after_static(code).map(|rest| held(rest, lines[i + 1..].iter().copied()));
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

/// The configuration file clippy reads for the crate `name`, relative
/// to the root: as clippy looks it up, the first of `.clippy.toml` and
/// `clippy.toml` in the crate's directory, then in each directory above
/// it up to the root. The root's `clippy.toml` when there is none, so a
/// missing file is named.
fn clippy_config(c: &Checkout, name: &str) -> String {
    let crate_dir = format!("{CRATES}/{name}/");
    let dirs = [crate_dir.as_str(), "crates/", ""];
    dirs.iter()
        .flat_map(|dir| [".clippy.toml", "clippy.toml"].map(|f| format!("{dir}{f}")))
        .find(|rel| c.root.join(rel).is_file())
        .unwrap_or_else(|| "clippy.toml".to_owned())
}

/// Requires the clippy configuration each product crate builds with,
/// its own or one in a directory above it, to carry every ban.
#[must_use]
pub fn crates_keep_their_bans(c: &Checkout) -> Vec<String> {
    let names = match crates(c) {
        Ok(names) => names,
        Err(e) => return vec![e],
    };
    let mut problems = Vec::new();
    for name in names {
        let file = clippy_config(c, &name);
        match c.read(&file) {
            Err(e) => problems.push(e),
            Ok(body) => problems.extend(
                missing_bans(&body)
                    .into_iter()
                    .map(|(table, b)| format!("crate {name}: {file} does not ban {b} in {table}")),
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
