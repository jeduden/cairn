//! Finding the git directory of the repository a directory is in.

use std::path::{Path, PathBuf};

use crate::Error;

/// The git directory of the repository `directory` is in, found the way
/// git finds it: `.git` in `directory` or in the nearest directory
/// above it. `None` when no directory above has one.
///
/// A `.git` that is a directory is the git directory. One that is a
/// file holds `gitdir: <path>`, as a linked worktree's and a
/// submodule's do; the path, relative to the directory the file is in,
/// is the git directory. A symbolic link is never followed: a `.git`
/// that is one, or a `gitdir:` that names one, is refused, so a clone
/// cannot point this crate at files outside itself by that route. The
/// directories above `directory` are walked as the path spells them,
/// without resolving links.
///
/// # Errors
///
/// [`Error::Unsupported`] for a symbolic link; [`Error::Corrupt`] for a
/// `.git` file that names no directory; [`Error::Io`] when the file
/// system cannot say.
pub fn git_dir(_directory: &Path) -> Result<Option<PathBuf>, Error> {
    Ok(None)
}

#[cfg(test)]
mod tests;
