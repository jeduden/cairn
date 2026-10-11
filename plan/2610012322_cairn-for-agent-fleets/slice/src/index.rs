//! The search index and room state in one SQLite file, derived only
//! from the events a node holds and its key set (I10, ADM-08).

use std::path::Path;

use crate::record::Address;
use crate::{Error, Result};

/// One indexed event's identity and trust, as the index holds them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub address: Address,
    pub provenance: String,
    pub origin: String,
    pub trust: String,
}

/// Every indexed event, in (writer, seq) order.
pub fn rows(_path: &Path) -> Result<Vec<Row>> {
    Err(Error::new("index not built"))
}

/// The addresses whose text matches the phrase.
pub fn phrase_matches(_path: &Path, _phrase: &str) -> Result<Vec<Address>> {
    Err(Error::new("index not built"))
}

/// The standing branch links the index holds, as (branch, act).
pub fn branch_links(_path: &Path) -> Result<Vec<(String, Address)>> {
    Err(Error::new("index not built"))
}
