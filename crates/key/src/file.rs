//! Key files: on Linux a node keeps its keys in files only the
//! principal's OS user reads (the plan's ruling 2). A key file is
//! written once, `0600`, and never overwritten; one others can read is
//! refused rather than used.

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::KeyError;

/// Writes `secret` to a new file at `path` that only its owner reads.
/// The secret goes to a temporary file beside it first, synced, then
/// linked into place, which refuses an existing file; so a crash leaves
/// either no key file or a whole one, never a part, and a part a crash
/// left in the temporary file is replaced.
///
/// # Errors
///
/// Fails when the file exists or cannot be written.
pub fn write_secret(path: &Path, secret: &str) -> Result<(), KeyError> {
    let failed = |source| KeyError::File {
        path: path.to_path_buf(),
        source,
    };
    let temporary = staging(path);
    match fs::remove_file(&temporary) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(failed(e)),
        _ => {}
    }
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = options.open(&temporary).map_err(failed)?;
    file.write_all(secret.as_bytes()).map_err(failed)?;
    file.sync_all().map_err(failed)?;
    let linked = fs::hard_link(&temporary, path);
    let _ = fs::remove_file(&temporary);
    linked.map_err(failed)?;
    sync_directory(path).map_err(failed)
}

/// The temporary file a key file is written to before it is linked
/// into place.
pub fn staging(path: &Path) -> std::path::PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".new");
    path.with_file_name(name)
}

/// Syncs the directory holding `path`, so a link or rename there
/// survives a crash.
///
/// # Errors
///
/// Fails when the directory cannot be opened or synced.
pub fn sync_directory(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    if let Some(parent) = path.parent() {
        fs::File::open(parent)?.sync_all()?;
    }
    #[cfg(not(unix))]
    let _ = path;

    Ok(())
}

/// Reads the secret in the file at `path`.
///
/// # Errors
///
/// Fails when it cannot be read, or when others can read it.
pub fn read_secret(path: &Path) -> Result<String, KeyError> {
    let failed = |source| KeyError::File {
        path: path.to_path_buf(),
        source,
    };
    let text = fs::read_to_string(path).map_err(failed)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(path).map_err(failed)?.permissions().mode() & 0o777;
        if mode & 0o077 != 0 {
            return Err(KeyError::Permissions {
                path: path.to_path_buf(),
                mode,
            });
        }
    }

    Ok(text.trim().to_owned())
}

#[cfg(test)]
mod tests;
