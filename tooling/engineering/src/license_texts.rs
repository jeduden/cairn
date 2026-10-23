//! ENG-30: every crate in the workspace's dependency tree carries a
//! license text in its published source, the repository keeps no
//! substitute, and each text fits the canonical text of a license in
//! the crate's declared SPDX expression. The canonical texts are kept
//! here, one file each under `license_texts/canon`, with `{{}}` for
//! the holder or name a template fills in. The check reads only
//! `cargo metadata` and the sources cargo has already fetched.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{Checkout, dependencies};

/// The canonical text of each license the allow-list in `deny.toml`
/// and its exceptions grant, by SPDX id. The text of a license a
/// template fills in leaves the holder, year and name out.
const CANON: [(&str, &str); 8] = [
    (
        "Apache-2.0",
        include_str!("license_texts/canon/Apache-2.0.txt"),
    ),
    (
        "BSD-2-Clause",
        include_str!("license_texts/canon/BSD-2-Clause.txt"),
    ),
    (
        "BSD-3-Clause",
        include_str!("license_texts/canon/BSD-3-Clause.txt"),
    ),
    (
        "BlueOak-1.0.0",
        include_str!("license_texts/canon/BlueOak-1.0.0.txt"),
    ),
    ("ISC", include_str!("license_texts/canon/ISC.txt")),
    ("MIT", include_str!("license_texts/canon/MIT.txt")),
    ("NCSA", include_str!("license_texts/canon/NCSA.txt")),
    (
        "Unicode-3.0",
        include_str!("license_texts/canon/Unicode-3.0.txt"),
    ),
];

/// What a template fills in, in a canonical text.
const FREE: &str = "{{}}";

/// A crate from a registry, found in `cargo metadata`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub version: String,
    /// The SPDX expression the crate declares, if any.
    pub license: Option<String>,
    /// The directory holding the crate's fetched source.
    pub dir: PathBuf,
}

/// The canonical text of license `id`, if the tool keeps one.
#[must_use]
pub fn canonical(id: &str) -> Option<&'static str> {
    CANON.iter().find(|(i, _)| *i == id).map(|(_, text)| *text)
}

/// A text as the comparison sees it: lowercase, copyright lines gone,
/// whitespace collapsed to single spaces.
#[must_use]
pub fn normalize(text: &str) -> String {
    text.lines()
        .filter(|line| !copyright_line(line))
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Whether `line` states who holds the copyright and since when,
/// rather than wrapping a sentence that mentions the word. A clause that
/// starts "(c)", such as Apache-2.0's section 4(c), is license text.
fn copyright_line(line: &str) -> bool {
    let line = line.trim().to_lowercase();
    let stated = line.starts_with("copyright")
        && (line.contains("(c)") || line.contains('©') || line.chars().any(|c| c.is_ascii_digit()));

    stated || line.starts_with('©')
}

/// How many words a title may take before the canonical text: "MIT
/// License", "The MIT License (MIT)", "Apache License Version 2.0".
const TITLE_WORDS: usize = 8;

/// Whether `text` fits `canon`: it is the canonical text, ignoring case,
/// whitespace and copyright lines, with each [`FREE`] standing for any
/// run of words and a title before it. Nothing may follow it, so a term
/// added after the text does not fit, except Apache-2.0's optional
/// appendix.
#[must_use]
pub fn fits(text: &str, canon: &str) -> bool {
    let text = normalize(text);
    let canon = normalize(canon);
    let parts: Vec<&str> = canon.split(FREE).map(str::trim).collect();
    let Some(first) = text.find(parts[0]) else {
        return false;
    };
    if text[..first].split_whitespace().count() > TITLE_WORDS {
        return false;
    }
    let mut rest = &text[first + parts[0].len()..];
    for part in &parts[1..] {
        let Some(at) = rest.find(part) else {
            return false;
        };
        rest = &rest[at + part.len()..];
    }

    rest.trim().is_empty() || appendix(rest)
}

/// Whether `rest`, the words after Apache-2.0's text, are its appendix.
fn appendix(rest: &str) -> bool {
    rest.trim_start()
        .starts_with("appendix: how to apply the apache license")
}

/// An SPDX license expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Id(String),
    And(Vec<Expr>),
    Or(Vec<Expr>),
}

/// Parses an SPDX expression: `OR` binds looser than `AND`, parentheses
/// group, an exception (`WITH ...`) is dropped, and the legacy `/` counts
/// as `OR`.
///
/// # Errors
///
/// Fails on an empty expression or unbalanced parentheses.
pub fn parse(expression: &str) -> Result<Expr, String> {
    let spaced = expression
        .replace('(', " ( ")
        .replace(')', " ) ")
        .replace('/', " OR ");
    let mut tokens = spaced.split_whitespace().peekable();
    let expr = parse_or(&mut tokens)?;
    match tokens.next() {
        None => Ok(expr),
        Some(extra) => Err(format!("{expression:?}: unexpected {extra:?}")),
    }
}

type Tokens<'a> = std::iter::Peekable<std::str::SplitWhitespace<'a>>;

fn parse_or(tokens: &mut Tokens) -> Result<Expr, String> {
    let mut parts = vec![parse_and(tokens)?];
    while tokens.next_if_eq(&"OR").is_some() {
        parts.push(parse_and(tokens)?);
    }

    Ok(if parts.len() == 1 {
        parts.remove(0)
    } else {
        Expr::Or(parts)
    })
}

fn parse_and(tokens: &mut Tokens) -> Result<Expr, String> {
    let mut parts = vec![parse_term(tokens)?];
    while tokens.next_if_eq(&"AND").is_some() {
        parts.push(parse_term(tokens)?);
    }

    Ok(if parts.len() == 1 {
        parts.remove(0)
    } else {
        Expr::And(parts)
    })
}

fn parse_term(tokens: &mut Tokens) -> Result<Expr, String> {
    match tokens.next() {
        Some("(") => {
            let inner = parse_or(tokens)?;
            (tokens.next() == Some(")"))
                .then_some(inner)
                .ok_or_else(|| "missing closing parenthesis".to_owned())
        }
        Some(id @ ("AND" | "OR" | ")")) => Err(format!("unexpected {id:?}")),
        Some(id) => {
            if tokens.next_if_eq(&"WITH").is_some() {
                tokens.next().ok_or("WITH names no exception")?;
            }
            Ok(Expr::Id(id.to_owned()))
        }
        None => Err("expression ends early".to_owned()),
    }
}

/// What keeps `expr` from being satisfied when `met` says which licenses
/// are: the ids still unmet, `None` when it is satisfied.
fn unmet(expr: &Expr, met: &dyn Fn(&str) -> bool) -> Option<Vec<String>> {
    match expr {
        Expr::Id(id) => (!met(id)).then(|| vec![id.clone()]),
        Expr::And(parts) => {
            let all: Vec<String> = parts
                .iter()
                .filter_map(|p| unmet(p, met))
                .flatten()
                .collect();
            (!all.is_empty()).then_some(all)
        }
        Expr::Or(parts) => {
            let all: Option<Vec<Vec<String>>> = parts.iter().map(|p| unmet(p, met)).collect();
            all.map(|lists| lists.into_iter().flatten().collect())
        }
    }
}

/// Whether a file name is one a crate keeps a license text in.
#[must_use]
pub fn license_file_name(name: &str) -> bool {
    let name = name.to_lowercase();

    ["license", "licence", "copying", "unlicense"]
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

/// The registry crates `metadata` lists: every package with a source,
/// which leaves out the workspace's own crates.
///
/// # Errors
///
/// Fails when `metadata` is not cargo's JSON, or lists a crate without
/// a name, version or manifest path, which would go unchecked.
pub fn packages(metadata: &str) -> Result<Vec<Package>, String> {
    let value: serde_json::Value =
        serde_json::from_str(metadata).map_err(|e| format!("read cargo metadata: {e}"))?;
    let mut found = Vec::new();
    for p in value["packages"].as_array().into_iter().flatten() {
        if !p["source"].is_string() {
            continue;
        }
        let package = (|| {
            Some(Package {
                name: p["name"].as_str()?.to_owned(),
                version: p["version"].as_str()?.to_owned(),
                license: p["license"].as_str().map(str::to_owned),
                dir: Path::new(p["manifest_path"].as_str()?)
                    .parent()?
                    .to_path_buf(),
            })
        })();
        found.push(package.ok_or_else(|| {
            format!("cargo metadata lists a crate without a name, version or manifest path: {p}")
        })?);
    }
    found.sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));

    Ok(found)
}

/// The license files in `dir`, as (name, text), in name order.
///
/// # Errors
///
/// Fails when the directory cannot be read.
pub fn license_files(dir: &Path) -> Result<Vec<(String, String)>, String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("read {}: {e}", dir.display()))?;
    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("read {}: {e}", dir.display()))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if license_file_name(&name) && entry.path().is_file() {
            // Not UTF-8 means no canonical text fits it; lossy keeps it listed.
            let bytes = fs::read(entry.path()).map_err(|e| format!("read {name}: {e}"))?;
            files.push((name, String::from_utf8_lossy(&bytes).into_owned()));
        }
    }
    files.sort();

    Ok(files)
}

/// Problems with one crate: no license file, no license declared, an
/// expression that does not parse, or a part of the expression no file
/// fits, with a note for each license the tool keeps no text for.
#[must_use]
pub fn check_crate(package: &Package, files: &[(String, String)]) -> Vec<String> {
    let who = format!("{} {}", package.name, package.version);
    if files.is_empty() {
        return vec![format!("{who} ships no license file")];
    }
    let Some(expression) = &package.license else {
        return vec![format!("{who} declares no license")];
    };
    let expr = match parse(expression) {
        Ok(expr) => expr,
        Err(e) => return vec![format!("{who} declares a license that does not parse: {e}")],
    };
    let met = |id: &str| {
        canonical(id).is_some_and(|canon| files.iter().any(|(_, text)| fits(text, canon)))
    };

    unmet(&expr, &met)
        .map(|ids| {
            let no_text: Vec<&str> = ids
                .iter()
                .map(String::as_str)
                .filter(|id| canonical(id).is_none())
                .collect();
            let note = if no_text.is_empty() {
                String::new()
            } else {
                format!(" (the tool keeps no text for {})", no_text.join(", "))
            };
            format!(
                "{who} declares {expression}, but no license file fits {}{note}",
                ids.join(" or ")
            )
        })
        .into_iter()
        .collect()
}

/// Problems with every crate in `metadata`, reading each crate's
/// license files through `read`.
///
/// # Errors
///
/// Fails when `metadata` is not cargo's JSON.
pub fn check_tree(
    metadata: &str,
    read: impl Fn(&Path) -> Result<Vec<(String, String)>, String>,
) -> Result<Vec<String>, String> {
    let mut problems = Vec::new();
    for package in packages(metadata)? {
        match read(&package.dir) {
            Ok(files) => problems.extend(check_crate(&package, &files)),
            Err(e) => problems.push(e),
        }
    }

    Ok(problems)
}

/// Problems with the canonical texts themselves: a license `deny.toml`
/// allows, or grants by exception, that the tool keeps no text for.
#[must_use]
pub fn canon_covers(deny_toml: &str) -> Vec<String> {
    let mut wanted: BTreeSet<String> = deny_allow(deny_toml).into_iter().collect();
    wanted.extend(
        dependencies::license_exceptions(deny_toml)
            .into_values()
            .flatten(),
    );

    wanted
        .into_iter()
        .filter(|id| canonical(id).is_none())
        .map(|id| format!("deny.toml grants {id}, which the license tool keeps no text for"))
        .collect()
}

/// The licenses `deny.toml`'s top-level `allow = [...]` lists, on one
/// line or several.
fn deny_allow(deny_toml: &str) -> Vec<String> {
    let Some(at) = deny_toml
        .lines()
        .scan(0, |offset, line| {
            let start = *offset;
            *offset += line.len() + 1;
            Some((start, line))
        })
        .find_map(|(start, line)| line.trim().starts_with("allow = [").then_some(start))
    else {
        return Vec::new();
    };
    let body = &deny_toml[at..];
    let list = body
        .split_once('[')
        .and_then(|(_, rest)| rest.split_once(']'))
        .map_or("", |(list, _)| list);

    list.lines()
        .map(|l| l.split('#').next().unwrap_or_default())
        .flat_map(|l| l.split(','))
        .map(|l| l.trim().trim_matches('"').to_owned())
        .filter(|l| !l.is_empty())
        .collect()
}

/// Problems with license texts the repository keeps for a crate: a
/// license file under `licenses/`, `third_party/` or `vendor/` in a
/// directory named for a crate, or for it and its version, other than a
/// target's committed `THIRD_PARTY_LICENSES` (ENG-31). A directory the
/// check cannot read is a problem too. A crate that ships no text is
/// removed or replaced, never patched here.
#[must_use]
pub fn kept_texts(checkout: &Checkout, packages: &[Package]) -> Vec<String> {
    let names: BTreeSet<String> = packages
        .iter()
        .flat_map(|p| [p.name.clone(), format!("{}-{}", p.name, p.version)])
        .collect();
    let (mut files, mut found) = (Vec::new(), Vec::new());
    for top in ["licenses", "third_party", "vendor"] {
        walk(&checkout.root.join(top), &mut files, &mut found);
    }
    for path in files {
        let in_crate_dir = path
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|d| names.contains(d.to_string_lossy().as_ref()));
        let named = path
            .file_name()
            .is_some_and(|n| license_file_name(&n.to_string_lossy()));
        if in_crate_dir && named {
            found.push(format!(
                "the repository keeps a license text at {}",
                path.strip_prefix(&checkout.root).unwrap_or(&path).display()
            ));
        }
    }
    found.sort();

    found
}

/// Lists in `files` each file under `dir`, and notes in `problems` a
/// directory it cannot read. A directory that does not exist holds
/// nothing.
fn walk(dir: &Path, files: &mut Vec<PathBuf>, problems: &mut Vec<String>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return,
        Err(e) => {
            problems.push(format!("read {}: {e}", dir.display()));
            return;
        }
    };
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| Some(e.ok()?.path())).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            walk(&path, files, problems);
        } else {
            files.push(path);
        }
    }
}

#[cfg(test)]
mod tests;
