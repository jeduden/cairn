//! Reads the requirement tables of the Software Requirements
//! Specification under `docs/srs`, so the test suite can hold the
//! specification and the executable scenarios in step. It is
//! repository tooling for the gates in the `scenario` crate and never
//! links into a shipped executable.

mod appendix;
mod personas;
mod table;

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub use appendix::{Counts, invariant_coverage, priority_counts, stated_counts, traced_coverage};
pub use personas::{persona_agents, persona_coverage};
pub use table::{Row, Table, fence_after, fence_marker, tables};

/// Every requirement family prefix, so the scenario gate reads ids the
/// same way.
pub const FAMILIES: [&str; 18] = [
    "REC", "PRV", "PIN", "RCL", "LMK", "INJ", "CMP", "ADM", "MEM", "OPS", "LANE", "VIEW", "OWN",
    "PEER", "SEC", "NFR", "ENG", "ASM",
];

/// One row of a requirement or assumption table. `priority` and
/// `traces` are empty when the table has no such column: the
/// non-functional and assumption tables carry neither, the engineering
/// tables no Traces.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Requirement {
    pub id: String,
    pub priority: String,
    pub verify: String,
    pub traces: Vec<String>,
    pub text: String,
    pub path: PathBuf,
    pub line: usize,
}

/// What reading the specification can fail with.
#[derive(Debug)]
pub enum Error {
    /// The directory could not be listed.
    List { dir: PathBuf, source: io::Error },
    /// A file could not be read.
    Read { path: PathBuf, source: io::Error },
    /// A table is malformed, or an id defined twice.
    Malformed(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::List { dir, source } => write!(f, "srs: list {}: {source}", dir.display()),
            Self::Read { path, source } => write!(f, "srs: read {}: {source}", path.display()),
            Self::Malformed(msg) => write!(f, "srs: {msg}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::List { source, .. } | Self::Read { source, .. } => Some(source),
            Self::Malformed(_) => None,
        }
    }
}

/// Whether `s` is a requirement id: a family prefix and a two-digit
/// number, such as `REC-01` or `ASM-04`.
#[must_use]
pub fn is_requirement_id(s: &str) -> bool {
    s.split_once('-').is_some_and(|(family, n)| {
        FAMILIES.contains(&family) && n.len() == 2 && n.bytes().all(|b| b.is_ascii_digit())
    })
}

/// Whether `s` is one invariant, `I1` to `I10`.
#[must_use]
pub fn is_invariant(s: &str) -> bool {
    s == "I10" || numbered_once(s, 'I')
}

/// Whether `s` is `prefix` followed by one digit from 1 to 9.
fn numbered_once(s: &str, prefix: char) -> bool {
    let mut chars = s.chars();
    chars.next() == Some(prefix)
        && chars.next().is_some_and(|c| ('1'..='9').contains(&c))
        && chars.next().is_none()
}

/// Reads every requirement from the Markdown files directly in `dir`,
/// in file-name order, and reports an id two rows share.
///
/// # Errors
///
/// Fails when `dir` or a file in it cannot be read, a table is
/// malformed, or two rows share an id.
pub fn load(dir: &Path) -> Result<Vec<Requirement>, Error> {
    let mut all = Vec::new();
    let mut seen: BTreeMap<String, (PathBuf, usize)> = BTreeMap::new();
    for path in markdown_files(dir)? {
        let body = fs::read_to_string(&path).map_err(|source| Error::Read {
            path: path.clone(),
            source,
        })?;
        for r in parse(&path, &body)? {
            if let Some((prev, line)) = seen.get(&r.id) {
                return Err(Error::Malformed(format!(
                    "{}:{}: {} already defined at {}:{line}",
                    r.path.display(),
                    r.line,
                    r.id,
                    prev.display()
                )));
            }
            seen.insert(r.id.clone(), (r.path.clone(), r.line));
            all.push(r);
        }
    }

    Ok(all)
}

/// The paths directly in `dir` whose names end in `.md`, sorted.
fn markdown_files(dir: &Path) -> Result<Vec<PathBuf>, Error> {
    let list = |source| Error::List {
        dir: dir.to_path_buf(),
        source,
    };
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir).map_err(list)? {
        let path = entry.map_err(list)?.path();
        if path.extension().is_some_and(|e| e == "md") {
            paths.push(path);
        }
    }
    paths.sort();

    Ok(paths)
}

/// Reads the requirements from one document. A requirement table is
/// one whose header leads with "ID" and names a "Requirement" or an
/// "Assumption" column; every other table (constraints, open
/// questions, glossaries) is passed by. Within a requirement table
/// every row must lead with a well-formed id and trace only real
/// invariants, or the row is reported with its line.
///
/// # Errors
///
/// Fails on the first malformed row.
pub fn parse(path: &Path, body: &str) -> Result<Vec<Requirement>, Error> {
    let mut out = Vec::new();
    for table in tables(body) {
        let cols = columns(&table.header);
        let is_requirements = cols.contains_key("Requirement") || cols.contains_key("Assumption");
        if table.header.first().map(String::as_str) != Some("ID") || !is_requirements {
            continue;
        }
        for row in &table.rows {
            out.push(requirement_of(path, row, &cols)?);
        }
    }

    Ok(out)
}

/// Maps each header name to its index.
fn columns(header: &[String]) -> BTreeMap<&str, usize> {
    header
        .iter()
        .enumerate()
        .map(|(i, h)| (h.as_str(), i))
        .collect()
}

/// Reads one table row.
fn requirement_of(
    path: &Path,
    row: &Row,
    cols: &BTreeMap<&str, usize>,
) -> Result<Requirement, Error> {
    let cell = |name: &str| {
        cols.get(name)
            .and_then(|i| row.cells.get(*i))
            .cloned()
            .unwrap_or_default()
    };
    let at = format!("{}:{}", path.display(), row.line);

    let id = cell("ID");
    if !is_requirement_id(&id) {
        return Err(Error::Malformed(format!(
            "{at}: malformed requirement id {id:?}"
        )));
    }
    let traces =
        parse_traces(&cell("Traces")).map_err(|e| Error::Malformed(format!("{at}: {id}: {e}")))?;

    Ok(Requirement {
        priority: cell("Pri"),
        verify: cell("Ver"),
        traces,
        text: cell("Requirement") + &cell("Assumption"),
        path: path.to_path_buf(),
        line: row.line,
        id,
    })
}

/// Reads a Traces cell: "—" or empty for none, else a comma-separated
/// list of invariants.
fn parse_traces(cell: &str) -> Result<Vec<String>, String> {
    if cell.is_empty() || cell == "—" {
        return Ok(Vec::new());
    }

    split_list(cell, is_invariant, "trace")
}

/// Reads a comma-separated cell whose every entry, trimmed, is
/// `valid`; the first entry that is not is reported as a malformed
/// `kind`.
fn split_list(cell: &str, valid: impl Fn(&str) -> bool, kind: &str) -> Result<Vec<String>, String> {
    cell.split(',')
        .map(|part| {
            let entry = part.trim();
            if valid(entry) {
                Ok(entry.to_owned())
            } else {
                Err(format!("malformed {kind} {entry:?}"))
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
