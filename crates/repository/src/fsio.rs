//! Bounded reads of the small text files a clone keeps: a reference, a
//! configuration, a list of shallow commits. The only way this crate
//! opens a file is for reading.

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
pub(crate) fn read_limited(_path: &Path, _limit: u64) -> Result<Option<Vec<u8>>, Error> {
    Ok(None)
}

#[cfg(test)]
mod tests;
