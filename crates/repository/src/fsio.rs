//! Bounded reads of the small text files a clone keeps: a reference, a
//! configuration, a list of shallow commits. The only way this crate
//! opens a file is for reading.

use std::fs::File;
use std::io::{ErrorKind, Read};
use std::path::Path;

use crate::Error;

/// The bytes of the file at `path`, or `None` when there is no such
/// file.
///
/// # Errors
///
/// [`Error::TooLarge`] when the file is longer than `limit` bytes,
/// which is checked while reading, so a huge file is never held;
/// [`Error::Io`] when it cannot be read.
pub(crate) fn read_limited(path: &Path, limit: u64) -> Result<Option<Vec<u8>>, Error> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(Error::io("a repository file")(e)),
    };
    let mut data = Vec::new();
    file.take(limit.saturating_add(1))
        .read_to_end(&mut data)
        .map_err(Error::io("a repository file"))?;
    if data.len() as u64 > limit {
        return Err(Error::TooLarge("repository file"));
    }
    Ok(Some(data))
}

#[cfg(test)]
mod tests;
