//! The counters derived from the audit log (OPS-01, OPS-03).

use std::collections::BTreeMap;

use crate::entry::Entry;

/// Each counter's count.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Counts {
    pub total: BTreeMap<String, u64>,
    pub since_ack: BTreeMap<String, u64>,
}

impl Counts {
    /// Whether one rose.
    #[must_use]
    pub fn risen(&self) -> bool {
        false
    }
}

/// The counts `entries` give.
#[must_use]
pub fn counts(_entries: &[Entry]) -> Counts {
    Counts::default()
}

#[cfg(test)]
mod tests;
