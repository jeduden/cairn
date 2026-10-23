//! Finding the git directory of the repository a directory is in.

use std::fs::{self, Metadata};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::Error;
use crate::fsio::read_limited;

/// The largest `.git` file read.
const MAX_GITDIR_FILE: u64 = 4096;

const BAD_GITDIR: Error = Error::Corrupt("gitdir file names no directory");

/// The directory a `.git` file in `dir` names with `gitdir: <path>`.
fn gitdir_file(dir: &Path) -> Result<PathBuf, Error> {
    let bytes = read_limited(dir, ".git", MAX_GITDIR_FILE)?.unwrap_or_default();
    let text = String::from_utf8_lossy(&bytes);
    let target = text
        .lines()
        .next()
        .and_then(|line| line.trim().strip_prefix("gitdir:"))
        .map(str::trim)
        .filter(|target| !target.is_empty())
        .ok_or(BAD_GITDIR)?;
    let path = dir.join(target);
    match fs::symlink_metadata(&path) {
        Ok(meta) if meta.is_symlink() => {
            Err(Error::Unsupported("symbolic link as a git directory"))
        }
        Ok(meta) if meta.is_dir() => Ok(path),
        Ok(_) => Err(BAD_GITDIR),
        Err(e) if e.kind() == ErrorKind::NotFound => Err(BAD_GITDIR),
        Err(e) => Err(Error::io("a git directory")(e)),
    }
}

/// The git directory a `.git` entry of `dir`, with `meta`, stands for.
fn from_dot_git(dir: &Path, dot_git: &Path, meta: &Metadata) -> Result<Option<PathBuf>, Error> {
    if meta.is_symlink() {
        Err(Error::Unsupported("symbolic link as .git"))
    } else if meta.is_dir() {
        Ok(Some(dot_git.to_path_buf()))
    } else if meta.is_file() {
        gitdir_file(dir).map(Some)
    } else {
        Ok(None)
    }
}

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
/// `.git` file that names no directory; [`Error::TooLarge`] for one
/// that is not a few bytes long; [`Error::Io`] when the file system
/// cannot say.
pub fn git_dir(directory: &Path, ceiling: Option<&Path>) -> Result<Option<PathBuf>, Error> {
    let _ = ceiling;
    for dir in directory.ancestors() {
        let dot_git = dir.join(".git");
        match fs::symlink_metadata(&dot_git) {
            Ok(meta) => {
                if let Some(found) = from_dot_git(dir, &dot_git, &meta)? {
                    return Ok(Some(found));
                }
            }
            Err(e) if matches!(e.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) => {}
            Err(e) => return Err(Error::io("a git directory")(e)),
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests;
