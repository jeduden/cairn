//! Administration of the home (§8.1, §9.5): where the home is, the
//! node's first start in it, the seats the node certifies, the
//! principal acts its CLI takes, and `cairn verify`'s check of the
//! whole record.

use std::fmt;
use std::path::PathBuf;

pub mod act;
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
    /// A directory or file of the home lets other OS users in (SEC-02).
    LoosePermissions { path: PathBuf, mode: u32 },
    /// A directory or file of the home belongs to OS user `uid`, not
    /// the `running` one (SEC-02, I8).
    OtherOsUser {
        path: PathBuf,
        uid: u32,
        running: u32,
    },
    /// A symbolic link inside the home, which could lead out of it.
    SymbolicLink { path: PathBuf },
    /// The running OS user cannot be learnt.
    OsUser(std::io::Error),
    /// The home's home id binding does not hold one home id (SEC-03).
    HomeIdBinding { path: PathBuf },
    /// The environment supplies another home id than the home's.
    HomeIdMismatch,
    /// The home is bound to a home id and the environment supplies none.
    HomeIdMissing,
}

impl fmt::Display for AdminError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoHome => f.write_str("no home: set CAIRN_HOME or HOME"),
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::Key(e) => write!(f, "{e}"),
            Self::Store(e) => write!(f, "{e}"),
            Self::Node(why) => f.write_str(why),
            Self::LoosePermissions { path, mode } => write!(
                f,
                "{}: mode {mode:04o} lets other OS users in; the home wants owner-only permissions",
                path.display()
            ),
            Self::OtherOsUser { path, uid, running } => write!(
                f,
                "{}: belongs to UID {uid}, not the running UID {running}",
                path.display()
            ),
            Self::SymbolicLink { path } => {
                write!(f, "{}: a symbolic link inside the home", path.display())
            }
            Self::OsUser(e) => write!(f, "learn the running OS user: {e}"),
            Self::HomeIdBinding { path } => {
                write!(f, "{}: want {{\"id\": \"<home id>\"}}", path.display())
            }
            Self::HomeIdMismatch => {
                f.write_str("the home id CAIRN_HOME_ID supplies does not match the home's")
            }
            Self::HomeIdMissing => {
                f.write_str("the home is bound to a home id, and CAIRN_HOME_ID supplies none")
            }
        }
    }
}

impl std::error::Error for AdminError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } | Self::OsUser(source) => Some(source),
            Self::Key(e) => Some(e),
            Self::Store(e) => Some(e),
            Self::NoHome
            | Self::Node(_)
            | Self::LoosePermissions { .. }
            | Self::OtherOsUser { .. }
            | Self::SymbolicLink { .. }
            | Self::HomeIdBinding { .. }
            | Self::HomeIdMismatch
            | Self::HomeIdMissing => None,
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
