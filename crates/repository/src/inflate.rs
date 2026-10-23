//! zlib decompression (RFC 1950, RFC 1951), written here because the
//! core is at its dependency budget (ENG-18). It reads a stream and
//! stops at the end of the zlib data, never past it; every output is
//! bounded by a maximum size the caller sets, so a hostile object
//! cannot make the core allocate without limit.

use std::io::BufRead;

use crate::Error;

/// A canonical Huffman code, built from the lengths of its symbols.
pub(crate) struct Huffman;

impl Huffman {
    /// The code whose symbol `n` has `lengths[n]` bits, or an error
    /// when the lengths give more codes than the bits can tell apart.
    pub(crate) fn build(_lengths: &[u8]) -> Result<Self, Error> {
        Err(Error::Corrupt("unimplemented"))
    }
}

/// A reader of bits, least significant first, over a byte stream.
pub(crate) struct Bits<R> {
    reader: R,
}

impl<R: BufRead> Bits<R> {
    pub(crate) fn new(reader: R) -> Self {
        Self { reader }
    }

    /// The next symbol of `code`.
    pub(crate) fn decode(&mut self, _code: &Huffman) -> Result<u16, Error> {
        let _ = &mut self.reader;
        Err(Error::Corrupt("unimplemented"))
    }
}

/// The Adler-32 checksum of `data` (RFC 1950).
pub(crate) fn adler32(_data: &[u8]) -> u32 {
    0
}

/// The data of the zlib stream `reader` holds, read up to the end of
/// the stream and no further.
///
/// # Errors
///
/// [`Error::Corrupt`] when the stream is not valid zlib, is cut short
/// or fails its checksum; [`Error::TooLarge`] when the data would pass
/// `max_size` bytes, which is checked before each byte is stored.
pub(crate) fn zlib_inflate<R: BufRead>(_reader: R, _max_size: usize) -> Result<Vec<u8>, Error> {
    Err(Error::Corrupt("unimplemented"))
}

#[cfg(test)]
mod tests;
