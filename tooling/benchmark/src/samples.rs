//! The file the samples travel in.

use std::collections::BTreeMap;

/// Each benchmark's samples, in nanoseconds, by benchmark name.
pub type Samples = BTreeMap<String, Vec<u64>>;
