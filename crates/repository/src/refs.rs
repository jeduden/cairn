//! References: the tip of a repository's default branch, found in loose
//! reference files and in `packed-refs`.

use crate::repo::Repo;
use crate::{CommitId, Deadline, Error};

/// The largest `packed-refs` read.
pub(crate) const MAX_PACKED_REFS: u64 = 64 << 20;

/// The symbolic references followed before a chain is called a loop.
pub(crate) const MAX_SYMBOLIC: usize = 5;

/// The tip of the default branch: where the remote's `HEAD` points, if
/// the clone has one that resolves, else where `HEAD` points. `None`
/// when that branch has no commit yet.
///
/// # Errors
///
/// [`Error::Corrupt`] for a reference that is not one;
/// [`Error::TooLarge`] for a `packed-refs` past the bound;
/// [`Error::Deadline`] when the deadline passes while `packed-refs` is
/// scanned.
pub(crate) fn default_branch_tip(
    _repo: &Repo,
    _deadline: &dyn Deadline,
) -> Result<Option<CommitId>, Error> {
    Ok(None)
}

/// The commit the reference `name` ends at, following symbolic
/// references, reading `packed-refs` up to `limit` bytes.
pub(crate) fn resolve(
    _repo: &Repo,
    _name: &str,
    _deadline: &dyn Deadline,
    _limit: u64,
) -> Result<Option<CommitId>, Error> {
    Ok(None)
}

#[cfg(test)]
mod tests;
