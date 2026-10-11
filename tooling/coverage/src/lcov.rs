//! Line coverage as cargo-llvm-cov exports it in lcov: per source file,
//! each instrumented line and how often it ran.

use std::collections::BTreeMap;
use std::path::PathBuf;

/// Hits per instrumented line, per source file.
pub type Lines = BTreeMap<PathBuf, BTreeMap<u32, u64>>;

/// Reads the `SF:` and `DA:` records of an lcov report. Other records
/// are summaries llvm-cov derives from these, and are passed by.
///
/// # Errors
///
/// Fails on a `DA:` record that is not `line,hits`.
pub fn parse(text: &str) -> Result<Lines, String> {
    let mut out = Lines::new();
    let mut file = None;
    for (n, record) in (1..).zip(text.lines()) {
        if let Some(path) = record.strip_prefix("SF:") {
            file = Some(PathBuf::from(path));
            continue;
        }
        let Some(data) = record.strip_prefix("DA:") else {
            continue;
        };
        let malformed = || format!("lcov line {n}: malformed record {record:?}");
        let mut fields = data.split(',');
        let line = fields
            .next()
            .and_then(|f| f.parse::<u32>().ok())
            .ok_or_else(malformed)?;
        let hits = fields
            .next()
            .and_then(|f| f.parse::<u64>().ok())
            .ok_or_else(malformed)?;
        let file = file
            .clone()
            .ok_or_else(|| format!("lcov line {n}: a DA record before any SF record"))?;
        *out.entry(file).or_default().entry(line).or_default() += hits;
    }

    Ok(out)
}

/// The coverage of every test layer at once: each line any layer
/// instruments, with the hits of all layers added up.
#[must_use]
pub fn merge<'a>(layers: impl IntoIterator<Item = &'a Lines>) -> Lines {
    let mut out = Lines::new();
    for lines in layers {
        for (file, hits) in lines {
            let merged = out.entry(file.clone()).or_default();
            for (line, n) in hits {
                *merged.entry(*line).or_default() += n;
            }
        }
    }

    out
}

#[cfg(test)]
mod tests;
