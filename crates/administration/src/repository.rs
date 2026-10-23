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

use key::random::{Entropy, draw};
use record::commitment::CommitmentKey;
use record::digest::Digest;
use record::event::{EventKind, Provenance};
use repository::{CommitId, Deadline};
use serde_json::{Value, json};
use store::{NewEvent, Store};

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
    /// The identity the directory resolves to changed since it was bound
    /// or last reported; nothing was relinked.
    Changed,
    /// It changed, and the change was reported already.
    StillChanged,
    /// The clone's git data could not be read.
    Unreadable,
    /// It could not be read, and that was reported already.
    StillUnreadable,
    /// The look ran out of its time; nothing was kept, so the next look
    /// tries again.
    TimedOut,
}

/// What the clone of a directory says its identity is now: a bound
/// commit, or none yet.
enum Current {
    Bound(CommitId),
    Provisional,
}

/// The label a local name is hashed under.
const LABEL: &[u8] = b"parinn repository v1\0";

/// The name this node keeps for `directory`: hashed under `secret`, so
/// it reveals neither the path nor anything of the repository.
fn directory_key(secret: &[u8; 32], directory: &Path) -> String {
    let resolved = std::fs::canonicalize(directory).unwrap_or_else(|_| directory.to_path_buf());

    Digest::of(&[
        LABEL,
        b"directory\0",
        secret,
        resolved.to_string_lossy().as_bytes(),
    ])
    .to_hex()
}

/// The name this node keeps for what a directory resolves to: a bound
/// commit's, or the one every provisional identity shares.
fn identity_key(secret: &[u8; 32], current: &Current) -> String {
    let what: &[u8] = match current {
        Current::Bound(id) => id.as_str().as_bytes(),
        Current::Provisional => b"provisional",
    };

    Digest::of(&[LABEL, b"identity\0", secret, what]).to_hex()
}

/// The commits this node has bound, by the structural events of its
/// device seat and by its principal's rebinds, lowest first.
fn bound_commits(store: &Store, node: &Node) -> Result<Vec<CommitId>, AdminError> {
    let mut bound: Vec<CommitId> = store
        .log(node.device_writer())?
        .iter()
        .filter(|s| {
            matches!(
                s.link.header.kind,
                EventKind::RepositoryBind | EventKind::PrincipalAct
            )
        })
        .filter_map(|s| s.content.as_ref()?["repository"].as_str())
        .filter_map(CommitId::parse)
        .collect();
    bound.sort();
    bound.dedup();

    Ok(bound)
}

/// What the clone at `git_dir` is now called: a commit this node bound
/// that the clone holds, else its lowest root commit, else nothing yet.
fn resolve(
    git_dir: &Path,
    bound: &[CommitId],
    deadline: &dyn Deadline,
) -> Result<Current, repository::Error> {
    for id in bound {
        if repository::has_commit(git_dir, id)? {
            return Ok(Current::Bound(id.clone()));
        }
    }

    Ok(match repository::root_commit(git_dir, deadline)? {
        Some(id) => Current::Bound(id),
        None => Current::Provisional,
    })
}

/// Reports `directory`'s clone unreadable the first time it is, and
/// never again until it reads.
fn unreadable(
    store: &mut Store,
    entropy: &mut dyn Entropy,
    directory: &Path,
) -> Result<Outcome, AdminError> {
    let secret = store.local_secret("repository", draw(entropy)?)?;
    let marker = format!("reported:{}", directory_key(&secret, directory));
    if store.directory_identity(&marker)?.as_deref() == Some(UNREADABLE) {
        return Ok(Outcome::StillUnreadable);
    }
    store.set_directory_identity(&marker, UNREADABLE)?;

    Ok(Outcome::Unreadable)
}

/// What a directory's marker holds while its clone is reported
/// unreadable; no identity key is this short.
const UNREADABLE: &str = "unreadable";

/// Looks at `directory` as a hook event shows it: binds its repository's
/// identity if this node never has. The search for the repository stops
/// before `ceiling`, the OS user's home directory, so a dotfiles
/// repository there is never bound for the directories below it
/// (`None` only in tests).
///
/// # Errors
///
/// Fails when the random source or the store fails.
pub fn bind(
    store: &mut Store,
    node: &Node,
    entropy: &mut dyn Entropy,
    directory: &Path,
    ceiling: Option<&Path>,
    deadline: &dyn Deadline,
) -> Result<Outcome, AdminError> {
    let git_dir = match repository::git_dir(directory, ceiling) {
        Ok(Some(git_dir)) => git_dir,
        Ok(None) => return Ok(Outcome::NoRepository),
        Err(_) => return unreadable(store, entropy, directory),
    };
    let bound = bound_commits(store, node)?;
    let Ok(current) = resolve(&git_dir, &bound, deadline) else {
        return unreadable(store, entropy, directory);
    };
    let secret = store.local_secret("repository", draw(entropy)?)?;
    let (directory, now) = (
        directory_key(&secret, directory),
        identity_key(&secret, &current),
    );
    match store.directory_identity(&directory)? {
        Some(kept) if kept == now => return Ok(Outcome::Unchanged),
        Some(_) => {
            let marker = format!("reported:{directory}");
            if store.directory_identity(&marker)?.as_deref() == Some(now.as_str()) {
                return Ok(Outcome::StillChanged);
            }
            store.set_directory_identity(&marker, &now)?;
            return Ok(Outcome::Changed);
        }
        None => {}
    }
    let provisional = matches!(current, Current::Provisional);
    let content = match &current {
        Current::Bound(id) if bound.contains(id) => None,
        Current::Bound(id) => Some(json!({"repository": id.as_str()})),
        Current::Provisional => Some(json!({
            "repository_provisional": Digest::of(&[&draw::<32>(entropy)?]).to_hex()[..32],
        })),
    };
    if let Some(content) = content {
        let event = NewEvent {
            kind: EventKind::RepositoryBind,
            provenance: Provenance::Structural,
            content,
            commitment_key: CommitmentKey::from_bytes(draw(entropy)?),
            transcript_timestamp: None,
        };
        store.append(node.device_writer(), &[event])?;
    }
    store.set_directory_identity(&directory, &now)?;

    Ok(Outcome::Bound { provisional })
}

/// What `parinn repository show` says of a directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shown {
    NoRepository,
    Unreadable,
    /// The clone's identity, which this node has not bound for the
    /// directory yet.
    Unbound(Option<CommitId>),
    /// The identity the directory is bound to and still resolves to.
    Bound(Option<CommitId>),
    /// The directory resolves to another identity than it was bound to.
    Changed(Option<CommitId>),
}

/// What `directory` is, as this node sees it, binding nothing.
///
/// # Errors
///
/// Fails when the store or the random source fails.
pub fn show(
    store: &mut Store,
    node: &Node,
    entropy: &mut dyn Entropy,
    directory: &Path,
    ceiling: Option<&Path>,
    deadline: &dyn Deadline,
) -> Result<Shown, AdminError> {
    let git_dir = match repository::git_dir(directory, ceiling) {
        Ok(Some(git_dir)) => git_dir,
        Ok(None) => return Ok(Shown::NoRepository),
        Err(_) => return Ok(Shown::Unreadable),
    };
    let bound = bound_commits(store, node)?;
    let Ok(current) = resolve(&git_dir, &bound, deadline) else {
        return Ok(Shown::Unreadable);
    };
    let secret = store.local_secret("repository", draw(entropy)?)?;
    let kept = store.directory_identity(&directory_key(&secret, directory))?;
    let commit = match &current {
        Current::Bound(id) => Some(id.clone()),
        Current::Provisional => None,
    };

    Ok(match kept {
        None => Shown::Unbound(commit),
        Some(kept) if kept == identity_key(&secret, &current) => Shown::Bound(commit),
        Some(_) => Shown::Changed(commit),
    })
}

/// What a widening bind will bind a directory to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rebind {
    /// The commit it binds to; none for a provisional identity.
    pub commit: Option<CommitId>,
    /// The node-local id of a provisional identity.
    pub provisional: Option<String>,
    directory: String,
    identity: String,
}

impl Rebind {
    /// The identity the act records: a commit, or a provisional id.
    #[must_use]
    pub fn content(&self) -> Value {
        match (&self.commit, &self.provisional) {
            (Some(id), _) => json!({"repository": id.as_str()}),
            (None, Some(id)) => json!({"repository_provisional": id}),
            (None, None) => json!({}),
        }
    }
}

/// The identity a widening bind of `directory` will give it: `commit`
/// when the principal names one, else what its clone resolves to. Nothing
/// changes until [`apply`] runs, after the act is recorded.
///
/// # Errors
///
/// Fails when the store or the random source fails, when the directory
/// lies in no repository, or when its git data cannot be read.
pub fn plan(
    store: &mut Store,
    node: &Node,
    entropy: &mut dyn Entropy,
    directory: &Path,
    ceiling: Option<&Path>,
    commit: Option<CommitId>,
    deadline: &dyn Deadline,
) -> Result<Rebind, AdminError> {
    let git_dir = repository::git_dir(directory, ceiling)
        .map_err(|_| AdminError::Repository("the repository's git data cannot be read"))?
        .ok_or(AdminError::Repository(
            "the directory lies in no repository",
        ))?;
    let current = match commit {
        Some(id) => Current::Bound(id),
        None => resolve(&git_dir, &bound_commits(store, node)?, deadline)
            .map_err(|_| AdminError::Repository("the repository's git data cannot be read"))?,
    };
    let secret = store.local_secret("repository", draw(entropy)?)?;
    let provisional = matches!(current, Current::Provisional)
        .then(|| {
            Ok::<_, AdminError>(Digest::of(&[&draw::<32>(entropy)?]).to_hex()[..32].to_owned())
        })
        .transpose()?;

    Ok(Rebind {
        commit: match &current {
            Current::Bound(id) => Some(id.clone()),
            Current::Provisional => None,
        },
        provisional,
        directory: directory_key(&secret, directory),
        identity: identity_key(&secret, &current),
    })
}

/// Makes `rebind` what its directory resolves to.
///
/// # Errors
///
/// Fails when the store does.
pub fn apply(store: &mut Store, rebind: &Rebind) -> Result<(), AdminError> {
    store.set_directory_identity(&rebind.directory, &rebind.identity)?;

    Ok(())
}

#[cfg(test)]
mod tests;
