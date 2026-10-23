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

pub use deadline::Deadline;
pub use error::Error;
pub use identity::{CommitId, RepositoryIdentity};
pub use locate::git_dir;
