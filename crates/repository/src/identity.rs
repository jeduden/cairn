//! The names a repository goes by: its bound commit or none yet.

/// A commit's id in git's SHA-1 object format: forty lowercase
/// hexadecimal digits.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommitId(String);

impl CommitId {
    /// The id `text` spells, or `None` unless it is exactly forty
    /// lowercase hexadecimal digits.
    #[must_use]
    pub fn parse(_text: &str) -> Option<Self> {
        None
    }

    /// The forty lowercase hexadecimal digits.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The id of the object whose twenty raw bytes are `bytes`.
    pub(crate) fn from_bytes(_bytes: &[u8; 20]) -> Self {
        Self(String::new())
    }

    /// The twenty raw bytes of the id.
    pub(crate) fn to_bytes(&self) -> [u8; 20] {
        [0; 20]
    }
}

/// What a directory's repository is called (LANE-02).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryIdentity {
    /// Named by its lowest root commit: the same on every node whose
    /// clone holds it, whatever the path.
    Bound(CommitId),
    /// No root commit is known yet: no commit, or a shallow clone whose
    /// history stops short of one. It carries nothing: the caller mints
    /// the node-local id and binds later, without rewriting the record.
    Provisional,
}

impl RepositoryIdentity {
    /// `Bound` when a root commit was found, `Provisional` otherwise.
    #[must_use]
    pub fn from_root(_root: Option<CommitId>) -> Self {
        Self::Provisional
    }
}

#[cfg(test)]
mod tests;
