//! Repository identity (LANE-02), read inside the core from a git
//! clone's own files.
//!
//! The crate starts no program and opens no socket (I4, SEC-01): it
//! reads loose objects, packfiles and references itself, and writes
//! nothing, so it needs no `git` binary and cannot be steered by a
//! hostile clone's hooks or configuration.

mod deadline;
mod delta;
mod error;
#[cfg(test)]
mod fixture;
mod fsio;
mod identity;
mod inflate;
mod locate;
mod loose;
mod object;
mod objects;
mod pack;
mod refs;
mod repo;
mod walk;

pub use deadline::Deadline;
pub use error::Error;
pub use identity::{CommitId, RepositoryIdentity};
pub use locate::git_dir;

use std::path::Path;

/// The root commit that names the repository whose git directory is
/// `git_dir`: the parentless commit with the lowest id among those
/// reachable from the default branch.
///
/// # Errors
///
/// See [`Error`].
pub fn root_commit(_git_dir: &Path, _deadline: &dyn Deadline) -> Result<Option<CommitId>, Error> {
    Ok(None)
}

/// Whether the clone whose git directory is `git_dir` holds the object
/// `id`, loose or packed.
///
/// # Errors
///
/// See [`Error`].
pub fn has_commit(_git_dir: &Path, _id: &CommitId) -> Result<bool, Error> {
    Ok(false)
}

/// The identity of the repository whose git directory is `git_dir`.
///
/// # Errors
///
/// See [`Error`].
pub fn identity(_git_dir: &Path, _deadline: &dyn Deadline) -> Result<RepositoryIdentity, Error> {
    Ok(RepositoryIdentity::Provisional)
}

#[cfg(test)]
mod tests;
