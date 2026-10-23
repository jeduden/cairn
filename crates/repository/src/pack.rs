//! Packfiles and their version 2 indexes, read through a reader that
//! can seek, so a unit test feeds bytes from memory.

use std::io::{Read, Seek};

use crate::object::Object;
use crate::{CommitId, Error};

/// An entry of a pack: a whole object, or a delta and what it is
/// against.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Entry {
    /// An object stored whole.
    Whole(Object),
    /// A delta against the entry `base` bytes... at this offset in the
    /// same pack.
    OffsetDelta { base: u64, delta: Vec<u8> },
    /// A delta against the object with this id, wherever it is.
    ReferenceDelta { base: CommitId, delta: Vec<u8> },
}

/// The offset in the pack of the object `id`, if the index lists it.
///
/// # Errors
///
/// [`Error::Unsupported`] for an index that is not version 2;
/// [`Error::Corrupt`] for one that contradicts its own size.
pub(crate) fn find_offset<R: Read + Seek>(
    _index: &mut R,
    _id: &CommitId,
) -> Result<Option<u64>, Error> {
    Ok(None)
}

/// Checks the pack's 12-byte header.
///
/// # Errors
///
/// [`Error::Corrupt`] when it is not a pack; [`Error::Unsupported`]
/// for a version other than 2 and 3.
pub(crate) fn check_header<R: Read + Seek>(_pack: &mut R) -> Result<(), Error> {
    Ok(())
}

/// The entry at `offset`, its data inflated to no more than `max_size`.
///
/// # Errors
///
/// [`Error::Corrupt`] for an entry that is cut short or contradicts its
/// own header; [`Error::TooLarge`] for one that declares more than
/// `max_size` bytes.
pub(crate) fn read_entry<R: Read + Seek>(
    _pack: &mut R,
    _offset: u64,
    _max_size: usize,
) -> Result<Entry, Error> {
    Err(Error::Corrupt("unimplemented"))
}

#[cfg(test)]
mod tests;
