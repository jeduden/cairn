//! Git's delta format: a result built from copies of a base and
//! inserted bytes.

use crate::Error;

/// The result of applying `delta` to `base`.
///
/// # Errors
///
/// [`Error::Corrupt`] when the delta does not fit its base or is cut
/// short; [`Error::TooLarge`] when it declares a result larger than
/// `max_size`, which is checked before anything is allocated.
pub(crate) fn apply(_base: &[u8], _delta: &[u8], _max_size: usize) -> Result<Vec<u8>, Error> {
    Err(Error::Corrupt("unimplemented"))
}

#[cfg(test)]
mod tests;
