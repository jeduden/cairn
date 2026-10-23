//! Where a clone keeps its files, and what format they are in.

use std::fs;
use std::path::{Path, PathBuf};

use crate::Error;
use crate::fsio::read_limited;

/// The largest `config` read.
const MAX_CONFIG: u64 = 1 << 20;

/// The largest `commondir` read.
const MAX_COMMONDIR: u64 = 4096;

/// A git directory opened for reading. A linked worktree keeps its own
/// `HEAD` apart from the objects, references and configuration of the
/// repository it belongs to, which `commondir` names.
#[derive(Debug)]
pub(crate) struct Repo {
    /// The directory `HEAD` is in.
    pub(crate) head_dir: PathBuf,
    /// The directory the objects, references and configuration are in.
    pub(crate) common_dir: PathBuf,
}

impl Repo {
    /// Opens the git directory `git_dir`, a repository's own or a linked
    /// worktree's.
    ///
    /// # Errors
    ///
    /// [`Error::UnsupportedObjectFormat`] for a repository that stores
    /// its objects in SHA-256; [`Error::Unsupported`] for one that keeps
    /// its references in reftable; [`Error::Io`] when `git_dir` cannot
    /// be read.
    pub(crate) fn open(git_dir: &Path) -> Result<Self, Error> {
        fs::metadata(git_dir).map_err(Error::io("a git directory"))?;
        let common_dir = match read_limited(&git_dir.join("commondir"), MAX_COMMONDIR)? {
            Some(bytes) => git_dir.join(String::from_utf8_lossy(&bytes).trim()),
            None => git_dir.to_path_buf(),
        };
        if let Some(config) = read_limited(&common_dir.join("config"), MAX_CONFIG)? {
            check_config(&String::from_utf8_lossy(&config))?;
        }
        Ok(Self {
            head_dir: git_dir.to_path_buf(),
            common_dir,
        })
    }

    /// The directory of the loose objects and packs.
    pub(crate) fn objects_dir(&self) -> PathBuf {
        self.common_dir.join("objects")
    }
}

/// The key and value of a configuration line, the key in lowercase and
/// the value unquoted, without its comment, in lowercase.
fn entry(line: &str) -> Option<(String, String)> {
    let (key, value) = line.split_once('=')?;
    let value = value.split(['#', ';']).next().unwrap_or("");
    let value = value.trim().trim_matches('"').to_ascii_lowercase();
    Some((key.trim().to_ascii_lowercase(), value))
}

/// Refuses an `extensions` setting this crate cannot read.
fn check_extension(key: &str, value: &str) -> Result<(), Error> {
    match key {
        "objectformat" if value != "sha1" => Err(Error::UnsupportedObjectFormat),
        "refstorage" if value != "files" => Err(Error::Unsupported("reference storage")),
        _ => Ok(()),
    }
}

/// Refuses a configuration that declares an object format or a
/// reference storage this crate does not read.
pub(crate) fn check_config(text: &str) -> Result<(), Error> {
    let mut section = String::new();
    for line in text.lines() {
        let mut rest = line.trim();
        if let Some(inner) = rest.strip_prefix('[') {
            let (name, after) = inner.split_once(']').unwrap_or((inner, ""));
            section = name.trim().to_ascii_lowercase();
            rest = after.trim();
        }
        if section == "extensions"
            && let Some((key, value)) = entry(rest)
        {
            check_extension(&key, &value)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
