//! Where the home is and what it holds (§8.1): `CAIRN_HOME` when set,
//! else `.cairn` in `HOME`. The home and its `keys` directory are
//! readable only by their owner.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use record::id::SeatId;

use crate::AdminError;

/// A Cairn home.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Home {
    root: PathBuf,
}

impl Home {
    /// The home `CAIRN_HOME` names, else `.cairn` in `HOME`; an empty
    /// variable counts as unset.
    ///
    /// # Errors
    ///
    /// Fails when neither names one.
    pub fn locate(cairn_home: Option<&OsStr>, home: Option<&OsStr>) -> Result<Self, AdminError> {
        let set = |v: Option<&OsStr>| v.filter(|v| !v.is_empty()).map(PathBuf::from);
        let root = set(cairn_home)
            .or_else(|| set(home).map(|h| h.join(".cairn")))
            .ok_or(AdminError::NoHome)?;

        Ok(Self { root })
    }

    /// The home's directory.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The store.
    #[must_use]
    pub fn store(&self) -> PathBuf {
        self.root.join("store.db")
    }

    /// The device key's file.
    #[must_use]
    pub fn device_key(&self) -> PathBuf {
        self.keys().join("device.key")
    }

    /// The device key's recovery key's file.
    #[must_use]
    pub fn recovery_key(&self) -> PathBuf {
        self.keys().join("recovery.key")
    }

    /// The file of the key of the seat `seat`.
    #[must_use]
    pub fn seat_key(&self, seat: SeatId) -> PathBuf {
        self.keys().join(format!("seat-{seat}.key"))
    }

    fn keys(&self) -> PathBuf {
        self.root.join("keys")
    }

    /// Makes the home and its `keys` directory, each readable only by
    /// its owner, where they do not exist yet.
    ///
    /// # Errors
    ///
    /// Fails when a directory cannot be made.
    pub fn create(&self) -> Result<(), AdminError> {
        let keys = self.keys();
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);

        builder
            .create(&keys)
            .map_err(|source| AdminError::Io { path: keys, source })
    }
}

#[cfg(test)]
mod tests;
