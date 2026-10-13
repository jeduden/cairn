//! The gate's decision (ENG-15): given the base's samples and the
//! head's for each benchmark, the head regressed when its median is more
//! than [`THRESHOLD_PERCENT`] slower than the base's *and* the
//! difference is statistically significant.
//!
//! # Why the median and the Mann–Whitney U test
//!
//! Timings are not normally distributed: they are skewed right, bounded
//! below by the work itself and spiked by whatever else the runner did
//! (preemption, a cold cache, a page fault). A test that assumes
//! normality, such as Welch's t-test, lets one spike move the mean and
//! inflate the variance, so it both misses real regressions and flags
//! noise. The gate uses statistics that a spike barely moves:
//!
//! - **Size**: the ratio of the medians. A spike moves a median by at
//!   most one position, where it can move a mean without bound. The
//!   medians are kept exact as twice their value, so the comparison
//!   against the threshold is integer arithmetic.
//! - **Significance**: the one-sided Mann–Whitney U test, which asks
//!   whether the head's samples tend to be larger than the base's. It
//!   works on ranks, so it assumes no distribution and an outlier counts
//!   as one rank however far it lies. Its normal approximation, with
//!   ties corrected, is a closed form: no resampling and no random seed,
//!   so the decision is a pure function of the samples, as derived
//!   tooling must be (I10's spirit, ENG-03). A bootstrap confidence
//!   interval on the ratio of medians was the alternative; with a fixed
//!   seed it is deterministic too, but its answer still depends on the
//!   seed and the resample count chosen, and it costs thousands of
//!   resamples per benchmark for no gain at the sample sizes the CI job
//!   takes.
//!
//! The level is one-sided α = 0.01 ([`Z_CRITICAL`]). The two conditions
//! guard each other: significance alone would fail a real 1% slowdown
//! measured precisely, and the threshold alone would fail a noisy run
//! whose medians drift apart by chance. The normal approximation holds
//! from about ten samples a side, so fewer is no decision, and no
//! decision fails ([`MIN_SAMPLES`]).
//!
//! The 10% is "more than": a head exactly 10% slower holds.
//!
//! # Benchmarks on one side only
//!
//! Nothing fails silently (I6). A benchmark the base sampled and the
//! head did not fails, unless the head's suite retired it by name, which
//! is a reviewed change to the suite. A benchmark only the head sampled
//! is new: it has nothing to be compared with, and the report says so.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::samples::Samples;
use crate::stats;

/// How much slower, in percent of the base's median, the head's median
/// may be before the gate looks at significance: ENG-15's 10%.
pub const THRESHOLD_PERCENT: u128 = 10;

/// The one-sided critical value of the standard normal at α = 0.01: a
/// Mann–Whitney z above it is a significant slowdown.
pub const Z_CRITICAL: f64 = 2.326_347_874_040_841;

/// The fewest samples a side needs for the normal approximation.
pub const MIN_SAMPLES: usize = 10;

/// What the report shows for a value a side does not have.
const NONE: &str = "–";

/// What the gate decided for one benchmark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Not more than the threshold slower, or not significantly.
    Held,
    /// More than the threshold slower, significantly: fails.
    Regressed,
    /// Sampled on the base, not on the head and not retired: fails.
    Missing,
    /// Not sampled on the head because the head's suite retired it.
    Retired,
    /// Sampled on the head only: nothing to compare with.
    New,
    /// A side has fewer than [`MIN_SAMPLES`]: no decision, so it fails.
    TooFew,
}

impl Status {
    /// The report's words for the status.
    #[must_use]
    pub fn describe(self) -> &'static str {
        match self {
            Self::Held => "held",
            Self::Regressed => "regressed",
            Self::Missing => "missing on the head",
            Self::Retired => "retired",
            Self::New => "new on the head",
            Self::TooFew => "too few samples",
        }
    }
}

/// One benchmark's comparison.
#[derive(Debug, Clone, PartialEq)]
pub struct Comparison {
    pub benchmark: String,
    pub status: Status,
    /// Twice the base's median, in nanoseconds, when the base sampled it.
    pub base: Option<u128>,
    /// Twice the head's median, in nanoseconds, when the head sampled it.
    pub head: Option<u128>,
    /// The Mann–Whitney z of the head against the base, when both
    /// sides had enough samples to compute it.
    pub z: Option<f64>,
}

impl Comparison {
    /// Whether this comparison fails the gate.
    #[must_use]
    pub fn fails(&self) -> bool {
        matches!(
            self.status,
            Status::Regressed | Status::Missing | Status::TooFew
        )
    }

    /// The change from the base's median to the head's, in percent.
    #[must_use]
    pub fn change_percent(&self) -> Option<f64> {
        match (self.base, self.head) {
            (Some(base), Some(head)) if base > 0 => Some((nanos(head) / nanos(base) - 1.0) * 100.0),
            _ => None,
        }
    }

    /// Why this comparison fails, naming its benchmark; `None` when it
    /// does not.
    #[must_use]
    pub fn failure(&self) -> Option<String> {
        let name = &self.benchmark;
        match self.status {
            Status::Regressed => Some(format!(
                "{name} regressed by {:.1}%: median {} on the base, {} on the head, z = {:.2} above {Z_CRITICAL:.2}",
                self.change_percent().unwrap_or_default(),
                micros(self.base),
                micros(self.head),
                self.z.unwrap_or_default(),
            )),
            Status::Missing => Some(format!(
                "{name} was sampled on the base but not on the head"
            )),
            Status::TooFew => Some(format!(
                "{name} has too few samples to decide: the gate needs {MIN_SAMPLES} on each side"
            )),
            Status::Held | Status::Retired | Status::New => None,
        }
    }
}

/// Compares every benchmark either side sampled, in name order.
/// `retired` names the benchmarks the head's suite retired.
#[must_use]
pub fn decide(base: &Samples, head: &Samples, retired: &[&str]) -> Vec<Comparison> {
    let names: BTreeSet<&String> = base.keys().chain(head.keys()).collect();

    names
        .into_iter()
        .map(|name| {
            compare(
                name,
                base.get(name).map(Vec::as_slice),
                head.get(name).map(Vec::as_slice),
                retired.contains(&name.as_str()),
            )
        })
        .collect()
}

/// Compares one benchmark's samples on each side.
fn compare(name: &str, base: Option<&[u64]>, head: Option<&[u64]>, retired: bool) -> Comparison {
    let mut c = Comparison {
        benchmark: name.to_owned(),
        status: Status::Held,
        base: base.and_then(stats::twice_median),
        head: head.and_then(stats::twice_median),
        z: None,
    };
    c.status = match (base, head) {
        (_, None) if retired => Status::Retired,
        (_, None) => Status::Missing,
        (None, Some(_)) => Status::New,
        (Some(base), Some(head)) if base.len().min(head.len()) < MIN_SAMPLES => Status::TooFew,
        (Some(base), Some(head)) => {
            let z = stats::mann_whitney_z(base, head);
            c.z = Some(z);
            if slower_than_threshold(c.base, c.head) && z > Z_CRITICAL {
                Status::Regressed
            } else {
                Status::Held
            }
        }
    };

    c
}

/// Whether the head's median is more than [`THRESHOLD_PERCENT`] above
/// the base's, in exact integer arithmetic.
fn slower_than_threshold(base: Option<u128>, head: Option<u128>) -> bool {
    match (base, head) {
        (Some(base), Some(head)) => {
            head.saturating_mul(100) > base.saturating_mul(100 + THRESHOLD_PERCENT)
        }
        _ => false,
    }
}

/// One line per failing comparison, each naming its benchmark; and one
/// when nothing was compared at all, since a gate over nothing proves
/// nothing (I6).
#[must_use]
pub fn failures(comparisons: &[Comparison]) -> Vec<String> {
    if comparisons.is_empty() {
        return vec!["no benchmark was sampled on the base or the head".to_owned()];
    }

    comparisons.iter().filter_map(Comparison::failure).collect()
}

/// The comparisons as a Markdown table, for the CI job's summary.
#[must_use]
pub fn report(comparisons: &[Comparison]) -> String {
    let mut out = String::from(
        "| Benchmark | Base median | Head median | Change | z | Result |\n\
         | --- | --- | --- | --- | --- | --- |\n",
    );
    for c in comparisons {
        let change = c
            .change_percent()
            .map_or_else(|| NONE.to_owned(), |p| format!("{p:+.1}%"));
        let z = c.z.map_or_else(|| NONE.to_owned(), |z| format!("{z:.2}"));
        let _ = writeln!(
            out,
            "| {} | {} | {} | {change} | {z} | {} |",
            c.benchmark,
            micros(c.base),
            micros(c.head),
            c.status.describe(),
        );
    }

    out
}

/// Twice a median, in nanoseconds, as the median in nanoseconds. Only
/// the report and the change it shows read it, rounded to a tenth.
fn nanos(twice: u128) -> f64 {
    twice as f64 / 2.0
}

/// Twice a median, in nanoseconds, as microseconds for the report.
fn micros(twice: Option<u128>) -> String {
    twice.map_or_else(
        || NONE.to_owned(),
        |t| format!("{:.1} µs", nanos(t) / 1000.0),
    )
}

#[cfg(test)]
mod tests;
