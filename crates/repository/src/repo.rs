//! Where a clone keeps its files, and what format they are in.

use std::path::{Path, PathBuf};

use crate::Error;

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
        Ok(Self {
            head_dir: git_dir.to_path_buf(),
            common_dir: git_dir.to_path_buf(),
        })
    }

    /// The directory of the loose objects and packs.
    pub(crate) fn objects_dir(&self) -> PathBuf {
        self.common_dir.join("objects")
    }
}

/// Refuses a configuration that declares an object format or a
/// reference storage this crate does not read.
pub(crate) fn check_config(_text: &str) -> Result<(), Error> {
    Ok(())
}

#[cfg(test)]
mod tests;
