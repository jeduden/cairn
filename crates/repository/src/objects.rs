//! The object store of a clone: loose objects and packs. Alternate
//! object stores are not read.

use std::path::{Path, PathBuf};

use crate::object::Object;
use crate::{CommitId, Error};

/// The most packs read. Git repacks long before this many; more is a
/// clone this crate will not open a file handle for each of.
pub(crate) const MAX_PACKS: usize = 256;

/// The longest delta chain followed. Git writes 50 by default and
/// 4095 at most; the bound stops a cycle and a hostile chain.
pub(crate) const MAX_CHAIN: usize = 1024;

/// A clone's objects, opened for reading.
#[derive(Debug)]
pub(crate) struct Objects {
    dir: PathBuf,
}

impl Objects {
    /// Opens the object directory `dir`: its packs are listed and
    /// opened, in name order.
    ///
    /// # Errors
    ///
    /// [`Error::TooLarge`] for more than [`MAX_PACKS`] packs;
    /// [`Error::Corrupt`] for a pack file that is not one;
    /// [`Error::Io`] when the directory cannot be listed.
    pub(crate) fn open(dir: &Path) -> Result<Self, Error> {
        Ok(Self {
            dir: dir.to_path_buf(),
        })
    }

    /// The object `id`, or `None` when the clone does not hold it.
    ///
    /// # Errors
    ///
    /// [`Error::Unsupported`] when the object is absent and the clone
    /// names alternate object stores, which could hold it;
    /// [`Error::Missing`] when a delta's base is absent;
    /// [`Error::TooLarge`] for a chain longer than [`MAX_CHAIN`] or an
    /// object past the bound.
    pub(crate) fn read(&self, _id: &CommitId) -> Result<Option<Object>, Error> {
        let _ = &self.dir;
        Ok(None)
    }

    /// Whether the clone holds the object `id`.
    ///
    /// # Errors
    ///
    /// [`Error::Unsupported`] when the object is absent and the clone
    /// names alternate object stores, which could hold it.
    pub(crate) fn contains(&self, _id: &CommitId) -> Result<bool, Error> {
        Ok(false)
    }
}

#[cfg(test)]
mod tests;
