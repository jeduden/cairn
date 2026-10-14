//! Key files: on Linux a node keeps its keys in files only the
//! principal's OS user reads (the plan's ruling 2). A key file is
//! written once, `0600`, and never overwritten; one others can read is
//! refused rather than used.

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::KeyError;

/// Writes `secret` to a new file at `path` that only its owner reads,
/// and syncs it to disk.
///
/// # Errors
///
/// Fails when the file exists or cannot be written.
pub fn write_secret(path: &Path, secret: &str) -> Result<(), KeyError> {
    let failed = |source| KeyError::File {
        path: path.to_path_buf(),
        source,
    };
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = options.open(path).map_err(failed)?;
    file.write_all(secret.as_bytes()).map_err(failed)?;

    file.sync_all().map_err(failed)
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
