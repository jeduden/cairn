//! The counters derived from the audit log (OPS-01, OPS-03): each
//! named counter counts the entries that name it, and its rise counts
//! those after the last `counter_ack` entry. Every counter Cairn names
//! so far is a failure counter (OQ-50 (i)). Counting reads no clock: the
//! same entries always give the same counts (I10).

use std::collections::BTreeMap;

use crate::entry::{COUNTER_ACK, Entry};

/// Each counter's count, and its rise since the counters were last
/// acknowledged; a counter with no entry is absent from both.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Counts {
    pub total: BTreeMap<String, u64>,
    pub since_ack: BTreeMap<String, u64>,
}

impl Counts {
    /// Whether any failure counter rose since the counters were last
    /// acknowledged, which fails `cairn doctor` (OPS-03).
    #[must_use]
    pub fn risen(&self) -> bool {
        !self.since_ack.is_empty()
    }
}

/// The counts `entries` give, taken in order.
#[must_use]
pub fn counts(entries: &[Entry]) -> Counts {
    let mut out = Counts::default();
    for e in entries {
        if e.kind == COUNTER_ACK {
            out.since_ack.clear();
        }
        if let Some(counter) = &e.counter {
            *out.total.entry(counter.clone()).or_default() += 1;
            *out.since_ack.entry(counter.clone()).or_default() += 1;
        }
    }

    out
}

#[cfg(test)]
mod tests;
