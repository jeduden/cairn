//! Reading what the spikes recorded: `testdata/claude-code` holds one
//! directory per Claude Code version, each with `hooks/<flow>/` (one
//! hook input per file, numbered in firing order), `transcripts/` and
//! the measurements, beside the `record` kit that made them.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// The directory beside the versions that holds the recording kit.
pub const RECORD_KIT: &str = "record";

/// The recordings of every Claude Code version under `root`, in name
/// order.
///
/// # Errors
///
/// Fails when `root` cannot be listed or holds no version.
pub fn versions(root: &Path) -> Result<Vec<PathBuf>, String> {
    let versions: Vec<PathBuf> = entries(root)?
        .into_iter()
        .filter(|p| p.is_dir() && p.file_name().is_some_and(|n| n != RECORD_KIT))
        .collect();
    if versions.is_empty() {
        return Err(format!("{}: no recorded version", root.display()));
    }
    Ok(versions)
}

/// The entries directly in `dir`, in name order.
///
/// # Errors
///
/// Fails when `dir` cannot be listed.
pub fn entries(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let failed = |e: io::Error| format!("list {}: {e}", dir.display());
    let mut paths = fs::read_dir(dir)
        .map_err(failed)?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(failed)?;
    paths.sort();
    Ok(paths)
}

/// The files under `dir` whose names end in `suffix`, recursively, in
/// path order.
///
/// # Errors
///
/// Fails when a directory cannot be listed.
pub fn files(dir: &Path, suffix: &str) -> Result<Vec<PathBuf>, String> {
    let mut found = Vec::new();
    for path in entries(dir)? {
        if path.is_dir() {
            found.extend(files(&path, suffix)?);
        } else if path.to_string_lossy().ends_with(suffix) {
            found.push(path);
        }
    }
    Ok(found)
}

/// Reads the JSON document at `path`.
///
/// # Errors
///
/// Fails when the file cannot be read or is not JSON.
pub fn json(path: &Path) -> Result<Value, String> {
    let body = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_str(&body).map_err(|e| format!("{}: {e}", path.display()))
}

/// Reads the JSON Lines file at `path`, one value per non-empty line.
///
/// # Errors
///
/// Fails when the file cannot be read or a line is not JSON.
pub fn lines(path: &Path) -> Result<Vec<Value>, String> {
    let body = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    parse_lines(&body).map_err(|e| format!("{}: {e}", path.display()))
}

/// Parses JSON Lines, one value per non-empty line.
///
/// # Errors
///
/// Fails on the first line that is not JSON, naming it.
pub fn parse_lines(body: &str) -> Result<Vec<Value>, String> {
    body.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(n, line)| serde_json::from_str(line).map_err(|e| format!("line {}: {e}", n + 1)))
        .collect()
}

/// The hook inputs `version` recorded in `flow`, in firing order.
///
/// # Errors
///
/// Fails when no flow is named, the flow holds no hook input, or one
/// cannot be read.
pub fn hook_inputs(version: &Path, flow: &str) -> Result<Vec<Value>, String> {
    if flow.is_empty() {
        return Err("no hook flow named".into());
    }
    let dir = version.join("hooks").join(flow);
    let inputs = files(&dir, ".json")?
        .iter()
        .map(|p| json(p))
        .collect::<Result<Vec<_>, _>>()?;
    if inputs.is_empty() {
        return Err(format!("{}: no hook input", dir.display()));
    }
    Ok(inputs)
}

/// The string a version's `manifest` gives at `key`, a dotted path.
///
/// # Errors
///
/// Fails when the manifest names no string there.
pub fn named<'a>(manifest: &'a Value, key: &str) -> Result<&'a str, String> {
    crate::at(manifest, key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("the manifest names no {key}"))
}

/// The pairs of the object a `manifest` gives at `key`, each value a
/// string.
///
/// # Errors
///
/// Fails when the manifest names no such object, it is empty, or a
/// value is not a string.
pub fn pairs(manifest: &Value, key: &str) -> Result<Vec<(String, String)>, String> {
    let map = crate::at(manifest, key)
        .and_then(Value::as_object)
        .filter(|m| !m.is_empty())
        .ok_or_else(|| format!("the manifest names no {key}"))?;
    map.iter()
        .map(|(k, v)| {
            v.as_str()
                .map(|s| (k.clone(), s.to_owned()))
                .ok_or_else(|| format!("the manifest's {key}.{k} is not a string"))
        })
        .collect()
}

#[cfg(test)]
mod tests;
