//! A node: the writers it holds, its key set, and what it derives.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::record::{Address, Event, Segment};
use crate::{Error, Result};

/// One node's record and key set. Its files live under one directory.
pub struct Node {
    dir: PathBuf,
    key_set: BTreeSet<String>,
}

impl Node {
    /// A node holding nothing yet, whose key set holds the seat keys of
    /// the writers named.
    pub fn new(dir: &Path, key_set: &[&str]) -> Result<Node> {
        Ok(Node {
            dir: dir.to_path_buf(),
            key_set: key_set.iter().map(|s| s.to_string()).collect(),
        })
    }

    /// Receives a segment: verifies it, merges it with what is held,
    /// and stores it.
    pub fn receive(&mut self, _segment: &Segment) -> Result<()> {
        Err(Error::new("receive not built"))
    }

    /// Applies an erasure request: this node's principal erases what
    /// the tombstone at `tombstone` names (PEER-11).
    pub fn apply_erasure(&mut self, _tombstone: &Address) -> Result<()> {
        Err(Error::new("erasure not built"))
    }

    /// Every held event, in (writer, seq) order.
    pub fn events(&self) -> Vec<&Event> {
        Vec::new()
    }

    /// Writes the held segments and rebuilds the search index and room
    /// state from them alone (ADM-08); returns the index file.
    pub fn rebuild(&self) -> Result<PathBuf> {
        Err(Error::new("rebuild not built"))
    }

    /// Every file the node stores.
    pub fn files(&self) -> Result<Vec<PathBuf>> {
        let _ = (&self.dir, &self.key_set);
        Err(Error::new("store not built"))
    }
}
