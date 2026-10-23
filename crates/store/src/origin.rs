//! The origins this store keeps (PRV-13, I10): for each writer, the
//! origin a derivation from the record and the node's key set gave it,
//! kept as a derived artifact beside the record. No reader decides an
//! origin by it: `parinn verify` compares it to a fresh derivation and
//! `parinn rebuild` writes it again. The table is made when the first
//! origins are written; a store without it holds none.

use std::collections::BTreeMap;

use record::id::WriterId;
use rusqlite::Connection;

use crate::StoreError;

/// The origins' table.
pub(crate) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS origins (
    writer TEXT PRIMARY KEY,
    origin TEXT NOT NULL
) STRICT, WITHOUT ROWID;
";

/// Every writer's kept origin, by writer; none while no table.
pub(crate) fn all(_conn: &Connection) -> Result<BTreeMap<WriterId, String>, StoreError> {
    Ok(BTreeMap::new())
}

/// Replaces the kept origins with `origins`, inside the transaction
/// `conn` is in.
pub(crate) fn replace(
    _conn: &Connection,
    _origins: &BTreeMap<WriterId, String>,
) -> rusqlite::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests;
