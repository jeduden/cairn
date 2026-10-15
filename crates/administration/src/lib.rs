//! Administration of the home (§8.1, §9.5): where the home is, the
//! node's first start in it, the seats the node certifies, and `cairn
//! verify`'s check of the whole record.

use std::fmt;
use std::path::PathBuf;

pub mod home;
pub mod node;
pub mod verify;

/// What can go wrong administering the home.
#[derive(Debug)]
pub enum AdminError {
    /// Neither `CAIRN_HOME` nor `HOME` names a home.
    NoHome,
    /// A directory of the home cannot be made.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// A key cannot be minted, kept or read, or a certificate signed.
    Key(key::KeyError),
    /// The store refused an operation.
    Store(store::StoreError),
    /// The home and the store disagree about the node.
    Node(&'static str),
}

impl fmt::Display for AdminError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoHome => f.write_str("no home: set CAIRN_HOME or HOME"),
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::Key(e) => write!(f, "{e}"),
            Self::Store(e) => write!(f, "{e}"),
            Self::Node(why) => f.write_str(why),
        }
    }
}

impl std::error::Error for AdminError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Key(e) => Some(e),
            Self::Store(e) => Some(e),
            Self::NoHome | Self::Node(_) => None,
        }
    }
}

impl From<key::KeyError> for AdminError {
    fn from(e: key::KeyError) -> Self {
        Self::Key(e)
    }
}

impl From<store::StoreError> for AdminError {
    fn from(e: store::StoreError) -> Self {
        Self::Store(e)
    }
}

#[cfg(test)]
mod tests;
