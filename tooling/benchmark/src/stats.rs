//! The two statistics the gate decides with: the median, kept exact as
//! twice its value, and the one-sided Mann–Whitney U test as a z score.
//! Both are pure and deterministic: no clock, no randomness, so the same
//! samples always yield the same decision.

/// Twice the median of `samples`: an even count's median is the mean of
/// its middle two, so twice it stays an integer and the gate compares
/// medians without rounding. `None` for no samples.
#[must_use]
pub fn twice_median(samples: &[u64]) -> Option<u128> {
    let _ = samples;
    None
}

/// The Mann–Whitney U statistic of `head` against `base` as a z score
/// under the normal approximation, with ties given their mean rank and
/// the variance corrected for them, and a continuity correction of one
/// half toward zero. Positive when the head's samples tend to be larger
/// than the base's. Zero when either side is empty or every sample ties,
/// since then nothing separates the two sides.
#[must_use]
pub fn mann_whitney_z(base: &[u64], head: &[u64]) -> f64 {
    let _ = (base, head);
    0.0
}

#[cfg(test)]
mod tests;
