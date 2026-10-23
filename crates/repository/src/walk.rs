//! The walk from the default branch's tip to the root commits behind it.

use std::collections::BTreeSet;

use crate::objects::Objects;
use crate::repo::Repo;
use crate::{CommitId, Deadline, Error};

/// The most commits one walk visits. The Linux kernel has about one and
/// a third million; the bound keeps the set of commits seen from
/// growing past what the deadline would stop anyway.
pub(crate) const MAX_COMMITS: usize = 2_000_000;

/// The largest `shallow` file read.
pub(crate) const MAX_SHALLOW: u64 = 16 << 20;

/// The commits `parent` headers name in a commit object's bytes.
///
/// # Errors
///
/// [`Error::Corrupt`] for a `parent` header that is not a commit id.
pub(crate) fn parents(_commit: &[u8]) -> Result<Vec<CommitId>, Error> {
    Ok(Vec::new())
}

/// The commits the clone's `shallow` file lists: where its history was
/// cut, whose parents it does not hold.
///
/// # Errors
///
/// [`Error::Corrupt`] for a line that is not a commit id;
/// [`Error::TooLarge`] for a file past `limit` bytes.
pub(crate) fn read_shallow(_repo: &Repo, _limit: u64) -> Result<BTreeSet<CommitId>, Error> {
    Ok(BTreeSet::new())
}

/// The parentless commit with the lowest id among those reachable from
/// `tip`. A commit in `shallow` is a boundary, not a root: its parents
/// are not followed, and it is not a candidate.
///
/// # Errors
///
/// [`Error::Missing`] for a parent the clone lacks and does not list as
/// shallow; [`Error::Corrupt`] for a commit that is not one;
/// [`Error::TooLarge`] past `max_commits`; [`Error::Deadline`] when the
/// deadline passes.
pub(crate) fn lowest_root(
    _objects: &Objects,
    _shallow: &BTreeSet<CommitId>,
    _tip: CommitId,
    _deadline: &dyn Deadline,
    _max_commits: usize,
) -> Result<Option<CommitId>, Error> {
    Ok(None)
}

#[cfg(test)]
mod tests;
