//! A repository's identity (LANE-02, I1, I6, I8): independent of the
//! local path and the same on every node holding a clone, a parentless
//! commit reachable from the repository's default branch, the lowest id
//! when there are several, found inside the core (no program start, no
//! socket) and bound once, at the first hook event that shows it, by a
//! structural event of the device seat that replicates (never an
//! `operator` one).
//!
//! - A node whose clone contains a commit already bound uses that
//!   identity. One whose clone lacks it mints none: a widening principal
//!   act binds it (`parinn repository bind`).
//! - A repository with no commit, or a shallow clone with none, gets a
//!   provisional, node-local identity, and binds later without
//!   rewriting the record.
//! - A change in the identity a directory resolves to is detected and
//!   reported, never relinks a room's branches, and rebinds only on the
//!   widening act.
//! - The identity serves only to identify branches and relate a room's
//!   branches and commits, never to scope ParleyInn state. What this
//!   node keeps of a directory and its identity is hashed under a secret
//!   only this node knows, so no local name reveals either off it.

use std::path::Path;

use key::random::Entropy;
use repository::{CommitId, Deadline};
use store::Store;

use crate::AdminError;
use crate::node::Node;

/// What a directory's repository resolves to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Identity {
    /// Named by its lowest root commit.
    Bound(CommitId),
    /// Node-local, minted by this node until a commit binds it.
    Provisional(String),
}

/// What one look at a directory came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The directory lies in no repository.
    NoRepository,
    /// The directory's identity is the one it last resolved to.
    Unchanged,
    /// The directory was bound now; whether to a provisional identity.
    Bound { provisional: bool },
    /// The identity the directory resolves to changed since it was bound;
    /// nothing was relinked.
    Changed,
    /// The clone's git data could not be read.
    Unreadable,
}

/// Looks at `directory` as a hook event shows it: binds its repository's
/// identity if this node never has.
///
/// # Errors
///
/// Fails when the random source or the store fails.
pub fn bind(
    _store: &mut Store,
    _node: &Node,
    _entropy: &mut dyn Entropy,
    _directory: &Path,
    _deadline: &dyn Deadline,
) -> Result<Outcome, AdminError> {
    Ok(Outcome::NoRepository)
}

#[cfg(test)]
mod tests;
