//! Reads a unified diff, as `git diff` prints it, for the paths it
//! changes and the requirement ids its changed lines name.

use std::collections::BTreeSet;

/// Every path the diff changes, both sides of a rename, sorted and
/// each once. It reads only the `diff --git a/X b/Y` headers.
#[must_use]
pub fn changed_paths(diff: &str) -> Vec<String> {
    let mut out = BTreeSet::new();
    for line in diff.lines() {
        let Some(rest) = line.strip_prefix("diff --git a/") else {
            continue;
        };
        if let Some((a, b)) = rest.split_once(" b/") {
            out.insert(a.to_owned());
            out.insert(b.to_owned());
        }
    }

    out.into_iter().collect()
}

/// Every requirement id an added or removed line names, sorted and each
/// once. File headers (`+++`, `---`) are not changed lines.
#[must_use]
pub fn mentioned_ids(diff: &str) -> Vec<String> {
    let mut out = BTreeSet::new();
    let changed = diff.lines().filter(|l| {
        (l.starts_with('+') || l.starts_with('-')) && !l.starts_with("+++") && !l.starts_with("---")
    });
    for line in changed {
        let tokens = line.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-'));
        out.extend(
            tokens
                .filter(|t| srs::is_requirement_id(t))
                .map(str::to_owned),
        );
    }

    out.into_iter().collect()
}

#[cfg(test)]
mod tests;
