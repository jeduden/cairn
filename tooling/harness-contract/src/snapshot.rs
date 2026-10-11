//! The snapshots `record/snap.sh` takes of every transcript of a
//! harness session as each hook fires (spike S13): one line per
//! transcript, `<file> lines=<n> boundaries=<n> last=<type>`.

use std::collections::BTreeMap;

/// One transcript's counts at a hook's time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    pub lines: usize,
    pub boundaries: usize,
}

/// Each transcript's counts, by file name.
pub type Snapshot = BTreeMap<String, Counts>;

/// Parses a snapshot file.
///
/// # Errors
///
/// Fails on a line that lacks its `lines` or `boundaries` count.
pub fn parse(text: &str) -> Result<Snapshot, String> {
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let mut words = l.split_whitespace();
            let file = words.next().unwrap_or_default().to_owned();
            let count = |key: &str| {
                l.split_whitespace()
                    .find_map(|w| w.strip_prefix(key))
                    .and_then(|n| n.parse().ok())
                    .ok_or_else(|| format!("{file}: no {key}<count>"))
            };
            Ok((
                file.clone(),
                Counts {
                    lines: count("lines=")?,
                    boundaries: count("boundaries=")?,
                },
            ))
        })
        .collect()
}

/// How many compaction boundaries the snapshot's transcripts hold.
#[must_use]
pub fn boundaries(snapshot: &Snapshot) -> usize {
    snapshot.values().map(|c| c.boundaries).sum()
}

/// The compaction hooks among `hooks`, each an event name, its
/// `source` or `trigger`, and the snapshot taken as it fired, whose
/// transcripts already held the boundary of the compaction they fired
/// for. The `k`-th compaction's `SessionStart` (`source` = `compact`)
/// and the `PostCompact` after it may see only the `k - 1` earlier
/// boundaries.
#[must_use]
pub fn early_boundaries(hooks: &[(String, String, Snapshot)]) -> Vec<String> {
    let mut compactions = 0;
    let mut problems = Vec::new();
    for (event, source, snapshot) in hooks {
        let current = match (event.as_str(), source.as_str()) {
            ("SessionStart", "compact") => {
                compactions += 1;
                true
            }
            ("PostCompact", _) => compactions > 0,
            _ => false,
        };
        let seen = boundaries(snapshot);
        if current && seen >= compactions {
            problems.push(format!(
                "the {event} hook of compaction {compactions} saw {seen} boundaries"
            ));
        }
    }
    problems
}

#[cfg(test)]
mod tests;
