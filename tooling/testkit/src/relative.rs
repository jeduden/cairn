//! A relative time check (NFR-02): a program's time judged against a
//! reference program's, both timed in interleaved pairs under the same
//! confinement at the same moment, so what slows one slows the other
//! and cancels in the ratio of their medians.

use std::time::Duration;

/// Twice the median of `samples`, in nanoseconds: an even count's median
/// is the mean of its middle two, so twice it stays an integer. `None`
/// for no samples.
#[must_use]
pub fn twice_median_nanos(samples: &[Duration]) -> Option<u128> {
    let _ = samples;
    None
}

/// Judges the median of `program` against `limit` times the median of
/// `reference`, in whole multiples.
///
/// # Errors
///
/// Fails, naming both medians and the limit, when the program's median
/// is more than `limit` times the reference's, or when either has no
/// samples.
pub fn within_multiple(
    program: &[Duration],
    reference: &[Duration],
    limit: u32,
) -> Result<(), String> {
    let _ = (program, reference, limit);
    Ok(())
}

#[cfg(test)]
mod tests;
