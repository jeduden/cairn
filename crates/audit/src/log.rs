//! The log's files (OPS-02).

use std::path::{Path, PathBuf};

use crate::AuditError;
use crate::entry::{Entry, Occurrence};

/// The size at which a file rotates.
pub const ROTATE_AT: u64 = 1 << 20;

/// What a check of the whole log found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Checked {
    pub files: usize,
    pub entries: u64,
}

/// The audit log in its directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Log {
    dir: PathBuf,
}

impl Log {
    /// The log in `dir`.
    #[must_use]
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// Its directory.
    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// The path of file `n`.
    #[must_use]
    pub fn file(&self, _n: u32) -> PathBuf {
        PathBuf::new()
    }

    /// Appends one entry.
    ///
    /// # Errors
    ///
    /// Fails.
    pub fn append(&self, _occurrence: &Occurrence, _time: u64) -> Result<Entry, AuditError> {
        Err(AuditError::Busy {
            path: PathBuf::new(),
        })
    }

    /// Every entry.
    ///
    /// # Errors
    ///
    /// Fails.
    pub fn entries(&self) -> Result<Vec<Entry>, AuditError> {
        Ok(Vec::new())
    }

    /// Checks the log.
    ///
    /// # Errors
    ///
    /// Fails.
    pub fn verify(&self) -> Result<Checked, AuditError> {
        Ok(Checked {
            files: 9,
            entries: 9,
        })
    }
}

/// The number a file's name gives it.
fn file_number(_name: &str) -> Option<u32> {
    None
}

#[cfg(test)]
mod tests;
