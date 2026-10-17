//! The node identity (REC-24): a value outside the home that a cloned
//! image or a restored snapshot cannot carry over, read before the first
//! append of each start. The home keeps only its digest, never the value.
//!
//! On Linux it is the machine id systemd keeps, `/etc/machine-id`, or
//! D-Bus's copy of it. macOS and Windows read none yet, so a home there
//! is never found changed (for the stakeholder: plan 2609292012,
//! phase 5).

use std::fs;
use std::path::{Path, PathBuf};

use key::file::{staging, sync_directory, write_secret};
use record::digest::Digest;

use crate::AdminError;
use crate::home::Home;

/// The label a node identity's digest is derived under.
const LABEL: &[u8] = b"cairn node identity v1\0";

/// Where a node identity is read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentitySource {
    /// The first of these files that holds a value.
    Files(Vec<PathBuf>),
    /// This OS has no source Cairn reads yet: the identity is not
    /// checked.
    Unsupported,
}

impl IdentitySource {
    /// This OS's source.
    #[must_use]
    pub fn platform() -> Self {
        if cfg!(target_os = "linux") {
            Self::Files(vec![
                PathBuf::from("/etc/machine-id"),
                PathBuf::from("/var/lib/dbus/machine-id"),
            ])
        } else {
            Self::Unsupported
        }
    }

    /// The value the first file that holds one holds, trimmed; none when
    /// no file holds one.
    #[must_use]
    pub fn read(&self) -> Option<String> {
        let Self::Files(files) = self else {
            return None;
        };
        files.iter().find_map(|f| {
            fs::read_to_string(f)
                .ok()
                .map(|v| v.trim().to_owned())
                .filter(|v| !v.is_empty())
        })
    }

    /// The value, read again once when the first read finds none
    /// (REC-24 lets Cairn retry within the hook budgets).
    #[must_use]
    pub fn observe(&self) -> Option<String> {
        self.read().or_else(|| self.read())
    }
}

/// The digest a home keeps of the node identity `value`; an unavailable
/// value has a digest of its own, so a node that never reads one changes
/// once, not at every start.
#[must_use]
pub fn digest(value: Option<&str>) -> Digest {
    match value {
        Some(v) => Digest::of(&[LABEL, b"value\0", v.as_bytes()]),
        None => Digest::of(&[LABEL, b"unavailable"]),
    }
}

/// Whether the node identity changed since `home` last recorded it:
/// the digest of the one it reads now when it did. A home that recorded
/// none yet records it now and counts as unchanged; an unsupported
/// source is never checked.
///
/// # Errors
///
/// Fails when the recorded digest cannot be read or written.
pub fn changed(home: &Home) -> Result<Option<Digest>, AdminError> {
    if *home.identity() != IdentitySource::Unsupported || home.identity().read().is_none() {
        return Ok(None);
    }
    if *home.identity() == IdentitySource::Unsupported {
        return Ok(None);
    }
    let current = digest(home.identity().observe().as_deref());
    let path = home.node_identity();
    let recorded = match fs::read_to_string(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            record(home, current)?;
            return Ok(None);
        }
        recorded => recorded.map_err(|source| io(&path, source))?,
    };

    Ok((recorded.trim() != current.to_hex()).then_some(current))
}

/// Records `current` as the digest of the node identity `home` is
/// started under, replacing the one it held in one rename.
///
/// # Errors
///
/// Fails when the file cannot be written.
pub fn record(home: &Home, current: Digest) -> Result<(), AdminError> {
    let path = home.node_identity();
    let temporary = staging(&path);
    match fs::remove_file(&temporary) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(io(&temporary, e)),
        _ => {}
    }
    write_secret(&temporary, &current.to_hex())?;
    fs::rename(&temporary, &path).map_err(|source| io(&path, source))?;
    sync_directory(&path).map_err(|source| io(&path, source))
}

/// The failure to read or write the file at `path`.
fn io(path: &Path, source: std::io::Error) -> AdminError {
    AdminError::Io {
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(test)]
mod tests;
