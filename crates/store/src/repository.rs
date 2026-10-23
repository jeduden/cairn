//! What this node keeps locally of the repositories its worktrees hold
//! (LANE-02): per worktree, the identity it last resolved to. Both are
//! named by a keyed digest the caller made, so no name here reveals a
//! worktree or an identity off this node. Neither is part of the record
//! and neither replicates. The rows whose key holds no `:` are derived
//! from the record's bind events and acts (I10; `parinn rebuild` writes
//! them again); the rows whose key holds one are node-local markers of
//! what this node already reported, and are not derived.

use std::collections::BTreeMap;

use rusqlite::{Connection, OptionalExtension};

use crate::StoreError;

/// The worktrees' table.
pub(crate) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS repository_worktrees (
    worktree TEXT PRIMARY KEY,
    identity TEXT NOT NULL
) STRICT, WITHOUT ROWID;
";

/// Whether the store has made the worktrees' table.
const HAS_TABLE: &str = "SELECT count(*) FROM sqlite_master WHERE name = 'repository_worktrees'";

/// The identity the worktree `worktree` last resolved to.
pub(crate) fn identity(conn: &Connection, worktree: &str) -> Result<Option<String>, StoreError> {
    if !conn.query_row(HAS_TABLE, [], |r| r.get::<_, bool>(0))? {
        return Ok(None);
    }

    Ok(conn
        .query_row(
            "SELECT identity FROM repository_worktrees WHERE worktree = ?1",
            [worktree],
            |r| r.get(0),
        )
        .optional()?)
}

/// Keeps `identity` as what `worktree` resolves to.
pub(crate) fn set_identity(
    conn: &Connection,
    worktree: &str,
    identity: &str,
) -> rusqlite::Result<()> {
    conn.execute_batch(SCHEMA)?;
    conn.execute(
        "INSERT OR REPLACE INTO repository_worktrees (worktree, identity) VALUES (?1, ?2)",
        [worktree, identity],
    )?;

    Ok(())
}

/// Forgets what `worktree` resolved to.
pub(crate) fn forget_identity(conn: &Connection, worktree: &str) -> rusqlite::Result<()> {
    conn.execute_batch(SCHEMA)?;
    conn.execute(
        "DELETE FROM repository_worktrees WHERE worktree = ?1",
        [worktree],
    )
    .map(|_| ())
}

/// The derived rows are those whose key holds no `:`; the rest are
/// markers.
const DERIVED: &str = "SELECT worktree, identity FROM repository_worktrees
    WHERE instr(worktree, ':') = 0 ORDER BY worktree";
const DELETE_DERIVED: &str = "DELETE FROM repository_worktrees WHERE instr(worktree, ':') = 0";
const INSERT: &str =
    "INSERT OR REPLACE INTO repository_worktrees (worktree, identity) VALUES (?1, ?2)";

/// The derived rows: every worktree whose key holds no `:`.
pub(crate) fn identities(conn: &Connection) -> Result<BTreeMap<String, String>, StoreError> {
    if !conn.query_row(HAS_TABLE, [], |r| r.get::<_, bool>(0))? {
        return Ok(BTreeMap::new());
    }
    let mut statement = conn.prepare(DERIVED)?;
    let rows = statement.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;

    Ok(rows.collect::<Result<_, _>>()?)
}

/// Replaces the derived rows with `identities`, inside the transaction
/// `conn` is in, leaving the markers alone.
pub(crate) fn replace(
    conn: &Connection,
    identities: &BTreeMap<String, String>,
) -> rusqlite::Result<()> {
    conn.execute_batch(SCHEMA)?;
    conn.execute(DELETE_DERIVED, [])?;
    for (worktree, identity) in identities {
        conn.execute(INSERT, [worktree, identity])?;
    }

    Ok(())
}

#[cfg(test)]
mod tests;
