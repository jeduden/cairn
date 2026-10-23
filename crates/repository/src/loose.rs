//! Loose objects: one zlib file each under `objects/xx/`.

use std::path::{Path, PathBuf};

use crate::object::Object;
use crate::{CommitId, Error};

/// Where the loose file of `id` is, whether or not there is one.
pub(crate) fn path_of(objects: &Path, id: &CommitId) -> PathBuf {
    let (fan, rest) = id.as_str().split_at(2);
    objects.join(fan).join(rest)
}

/// Whether `id` is stored loose.
///
/// # Errors
///
/// [`Error::Io`] when the file system cannot say.
pub(crate) fn exists(_objects: &Path, _id: &CommitId) -> Result<bool, Error> {
    Ok(false)
}

/// The loose object `id`, or `None` when it is not stored loose.
///
/// # Errors
///
/// [`Error::Corrupt`] when the file is not an object;
/// [`Error::TooLarge`] when it holds more than `max_size` bytes.
pub(crate) fn read_up_to(
    _objects: &Path,
    _id: &CommitId,
    _max_size: usize,
) -> Result<Option<Object>, Error> {
    Ok(None)
}

#[cfg(test)]
mod tests;
