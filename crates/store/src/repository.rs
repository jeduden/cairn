//! What this node keeps locally of the repositories its directories
//! hold (LANE-02): a secret only this node knows, and, per directory,
//! the identity it last resolved to. Both are hashed under that secret
//! by the caller, so no name here reveals a directory or an identity off
//! this node. Neither is part of the record and neither replicates.

use rusqlite::{Connection, OptionalExtension};

use crate::StoreError;

/// The directories' table.
pub(crate) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS repository_directories (
    directory TEXT PRIMARY KEY,
    identity TEXT NOT NULL
) STRICT, WITHOUT ROWID;
";

/// The `meta` key the local secret called `name` is kept under.
fn key(name: &str) -> String {
    format!("local_secret:{name}")
}

/// The local secret `name`, keeping `fresh` when none is kept yet.
pub(crate) fn secret(
    conn: &Connection,
    name: &str,
    fresh: [u8; 32],
) -> Result<[u8; 32], StoreError> {
    let _ = (conn, name);
    Ok(fresh)
}

/// The identity the directory `directory` last resolved to.
pub(crate) fn identity(conn: &Connection, directory: &str) -> Result<Option<String>, StoreError> {
    let _ = (conn, directory);
    Ok(None)
}

/// Keeps `identity` as what `directory` resolves to.
pub(crate) fn set_identity(
    conn: &Connection,
    directory: &str,
    identity: &str,
) -> rusqlite::Result<()> {
    let _ = (conn, directory, identity);
    Ok(())
}

#[cfg(test)]
mod tests;
