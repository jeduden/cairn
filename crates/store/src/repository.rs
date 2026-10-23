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

/// Whether the store has made the directories' table.
const HAS_TABLE: &str = "SELECT count(*) FROM sqlite_master WHERE name = 'repository_directories'";

/// The local secret `name`, keeping `fresh` when none is kept yet.
pub(crate) fn secret(
    conn: &Connection,
    name: &str,
    fresh: [u8; 32],
) -> Result<[u8; 32], StoreError> {
    let key = key(name);
    conn.execute(
        "INSERT OR IGNORE INTO meta (key, value) VALUES (?1, ?2)",
        (&key, hex(&fresh)),
    )?;
    let kept: String = conn.query_row("SELECT value FROM meta WHERE key = ?1", [&key], |r| {
        r.get(0)
    })?;

    unhex(&kept).ok_or_else(|| StoreError::Corrupt {
        address: None,
        what: "local secret".to_owned(),
    })
}

/// The identity the directory `directory` last resolved to.
pub(crate) fn identity(conn: &Connection, directory: &str) -> Result<Option<String>, StoreError> {
    if !conn.query_row(HAS_TABLE, [], |r| r.get::<_, bool>(0))? {
        return Ok(None);
    }

    Ok(conn
        .query_row(
            "SELECT identity FROM repository_directories WHERE directory = ?1",
            [directory],
            |r| r.get(0),
        )
        .optional()?)
}

/// Keeps `identity` as what `directory` resolves to.
pub(crate) fn set_identity(
    conn: &Connection,
    directory: &str,
    identity: &str,
) -> rusqlite::Result<()> {
    conn.execute_batch(SCHEMA)?;
    conn.execute(
        "INSERT OR REPLACE INTO repository_directories (directory, identity) VALUES (?1, ?2)",
        [directory, identity],
    )?;

    Ok(())
}

/// Forgets what `directory` resolved to.
pub(crate) fn forget_identity(conn: &Connection, directory: &str) -> rusqlite::Result<()> {
    conn.execute_batch(SCHEMA)?;
    conn.execute(
        "DELETE FROM repository_directories WHERE directory = ?1",
        [directory],
    )
    .map(|_| ())
}

/// `bytes` as lowercase hex.
fn hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The 32 bytes `text` holds as hex.
fn unhex(text: &str) -> Option<[u8; 32]> {
    if text.len() != 64 || !text.is_ascii() {
        return None;
    }
    let mut out = [0; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[2 * i..2 * i + 2], 16).ok()?;
    }

    Some(out)
}

#[cfg(test)]
mod tests;
