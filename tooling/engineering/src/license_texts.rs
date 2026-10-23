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
/// rather than wrapping a sentence that mentions the word.
fn copyright_line(line: &str) -> bool {
    let line = line.trim().to_lowercase();
    let stated = line.starts_with("copyright")
        && (line.contains("(c)") || line.contains('©') || line.chars().any(|c| c.is_ascii_digit()));

    stated || line.starts_with("(c)") || line.starts_with('©')
}

/// Whether `text` fits `canon`: the canonical text appears in it, ignoring
/// case, whitespace and copyright lines, with each [`FREE`] standing for
/// any run of words.
#[must_use]
pub fn fits(text: &str, canon: &str) -> bool {
    let text = normalize(text);
    let canon = normalize(canon);
    let mut rest = text.as_str();
    for part in canon.split(FREE) {
        let part = part.trim();
        let Some(at) = rest.find(part) else {
            return false;
        };
        rest = &rest[at + part.len()..];
    }

    true
}

/// The licenses an SPDX expression offers, as groups that must all be
/// satisfied: `A AND B` is two groups, `A OR B` one group of two
/// choices. An exception (`WITH ...`) is dropped, and the legacy `/`
/// counts as `OR`.
#[must_use]
pub fn choices(expression: &str) -> Vec<Vec<String>> {
    expression
        .replace(['(', ')'], " ")
        .split(" AND ")
        .map(|group| {
            group
                .split(" OR ")
                .flat_map(|choice| choice.split('/'))
                .filter_map(|choice| choice.split_whitespace().next())
                .map(str::to_owned)
                .collect()
        })
        .collect()
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
/// Fails when `metadata` is not cargo's JSON.
pub fn packages(metadata: &str) -> Result<Vec<Package>, String> {
    let value: serde_json::Value =
        serde_json::from_str(metadata).map_err(|e| format!("read cargo metadata: {e}"))?;
    let mut found: Vec<Package> = value["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|p| p["source"].is_string())
        .filter_map(|p| {
            Some(Package {
                name: p["name"].as_str()?.to_owned(),
                version: p["version"].as_str()?.to_owned(),
                license: p["license"].as_str().map(str::to_owned),
                dir: Path::new(p["manifest_path"].as_str()?)
                    .parent()?
                    .to_path_buf(),
            })
        })
        .collect();
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

/// Problems with one crate: no license file, no license declared, a
/// license in its expression with no canonical text, or a part of its
/// expression no file fits.
#[must_use]
pub fn check_crate(package: &Package, files: &[(String, String)]) -> Vec<String> {
    let who = format!("{} {}", package.name, package.version);
    if files.is_empty() {
        return vec![format!("{who} ships no license file")];
    }
    let Some(expression) = &package.license else {
        return vec![format!("{who} declares no license")];
    };
    choices(expression)
        .iter()
        .filter_map(|group| {
            let known: Vec<&str> = group
                .iter()
                .map(String::as_str)
                .filter(|id| canonical(id).is_some())
                .collect();
            if known.is_empty() {
                return Some(format!(
                    "{who} declares {expression}, none of {} has a canonical text",
                    group.join(", ")
                ));
            }
            let satisfied = known.iter().any(|id| {
                canonical(id).is_some_and(|canon| files.iter().any(|(_, text)| fits(text, canon)))
            });

            (!satisfied).then(|| {
                format!(
                    "{who} declares {expression}, but no license file fits {}",
                    known.join(" or ")
                )
            })
        })
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

/// The licenses `deny.toml`'s `allow = [...]` line lists.
fn deny_allow(deny_toml: &str) -> Vec<String> {
    deny_toml
        .lines()
        .find_map(|line| line.trim().strip_prefix("allow = ["))
        .and_then(|rest| rest.split_once(']'))
        .map(|(list, _)| {
            list.split(',')
                .map(|l| l.trim().trim_matches('"').to_owned())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Problems with license texts the repository keeps for a crate: a
/// license file under `licenses/`, `third_party/` or `vendor/` in a
/// directory named for a crate, other than a target's committed
/// `THIRD_PARTY_LICENSES` (ENG-31). A crate that ships no text is
/// removed or replaced, never patched here.
#[must_use]
pub fn kept_texts(checkout: &Checkout, packages: &[Package]) -> Vec<String> {
    let names: BTreeSet<&str> = packages.iter().map(|p| p.name.as_str()).collect();
    let mut found = Vec::new();
    for top in ["licenses", "third_party", "vendor"] {
        walk(&checkout.root.join(top), &mut |path| {
            let in_crate_dir = path.parent().and_then(Path::file_name).is_some_and(|d| {
                let d = d.to_string_lossy();
                names.contains(d.as_ref())
                    || names
                        .iter()
                        .any(|n| d.strip_prefix(n).is_some_and(|v| v.starts_with('-')))
            });
            let named = path
                .file_name()
                .is_some_and(|n| license_file_name(&n.to_string_lossy()));
            if in_crate_dir && named {
                found.push(format!(
                    "the repository keeps a license text at {}",
                    path.strip_prefix(&checkout.root).unwrap_or(path).display()
                ));
            }
        });
    }
    found.sort();

    found
}

/// Calls `visit` on each file under `dir`.
fn walk(dir: &Path, visit: &mut impl FnMut(&Path)) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| Some(e.ok()?.path())).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            walk(&path, visit);
        } else {
            visit(&path);
        }
    }
}

#[cfg(test)]
mod tests;
