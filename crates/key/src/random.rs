//! Where random bits come from: the OS's random source through
//! `getrandom`, behind a trait a test replaces with a counter.

use crate::KeyError;

/// A source of random bytes.
pub trait Entropy {
    /// Fills `buf` with random bytes.
    ///
    /// # Errors
    ///
    /// Fails when the source cannot supply them.
    fn fill(&mut self, buf: &mut [u8]) -> Result<(), KeyError>;
}

/// The OS's random source.
#[derive(Debug, Clone, Copy, Default)]
pub struct OsEntropy;

impl Entropy for OsEntropy {
    fn fill(&mut self, buf: &mut [u8]) -> Result<(), KeyError> {
        getrandom::fill(buf).map_err(|e| KeyError::Entropy(e.to_string()))
    }
}

/// `N` fresh random bytes from `entropy`.
///
/// # Errors
///
/// Fails when `entropy` cannot supply them.
pub fn draw<const N: usize>(entropy: &mut dyn Entropy) -> Result<[u8; N], KeyError> {
    let mut out = [0; N];
    entropy.fill(&mut out)?;

    Ok(out)
}

/// A deterministic stand-in for a test: each fill writes its own count,
/// 1, 2, 3 and on, into every byte.
#[cfg(test)]
#[derive(Debug, Default)]
pub struct Counted(u8);

#[cfg(test)]
impl Entropy for Counted {
    fn fill(&mut self, buf: &mut [u8]) -> Result<(), KeyError> {
        self.0 = self.0.wrapping_add(1);
        buf.fill(self.0);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
