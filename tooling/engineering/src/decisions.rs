//! ENG-26: every design decision lives in one ADR file, named for its
//! id, and a changed decision supersedes the record it replaces.

use std::collections::{BTreeMap, BTreeSet};

use adr::Adr;

/// Checks every record's identity fields.
#[must_use]
pub fn adrs_complete(adrs: &[Adr]) -> Vec<String> {
    let mut problems = Vec::new();
    for a in adrs {
        let path = a.path.display();
        let id_shape =
            a.id.strip_prefix("ADR-")
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()));
        if !id_shape {
            problems.push(format!("{path}: id {:?} is not ADR-<digits>", a.id));
        }
        for (field, value) in [
            ("title", &a.title),
            ("status", &a.status),
            ("summary", &a.summary),
        ] {
            if value.is_empty() {
                problems.push(format!("{path}: no {field}"));
            }
        }
    }

    problems
}

/// Checks each file name leads with its record's id, so the id alone
/// finds the file.
#[must_use]
pub fn adrs_named_for_id(adrs: &[Adr]) -> Vec<String> {
    adrs.iter()
        .filter(|a| {
            let name = a
                .path
                .file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_default();
            !name.starts_with(&format!("{}-", a.id))
        })
        .map(|a| format!("{} is not named for its id {}", a.path.display(), a.id))
        .collect()
}

/// Checks every status is one ENG-26 names.
#[must_use]
pub fn adr_statuses_valid(adrs: &[Adr]) -> Vec<String> {
    adrs.iter()
        .filter(|a| ![adr::PROPOSED, adr::ACCEPTED, adr::SUPERSEDED].contains(&a.status.as_str()))
        .map(|a| {
            format!(
                "{}: status {:?} is not proposed, accepted or superseded",
                a.id, a.status
            )
        })
        .collect()
}

/// Refuses two records with one id.
#[must_use]
pub fn adr_ids_unique(adrs: &[Adr]) -> Vec<String> {
    let mut seen = BTreeMap::new();
    let mut problems = Vec::new();
    for a in adrs {
        match seen.get(&a.id) {
            Some(prev) => problems.push(format!(
                "{}: id {} already used by {prev}",
                a.path.display(),
                a.id
            )),
            None => {
                seen.insert(&a.id, a.path.display().to_string());
            }
        }
    }

    problems
}

/// Checks a superseded record points at another record that exists, and
/// that a record naming a successor is itself marked superseded, so a
/// replaced decision never stays in force beside its replacement.
#[must_use]
pub fn superseded_names_successor(adrs: &[Adr]) -> Vec<String> {
    let ids: BTreeSet<&str> = adrs.iter().map(|a| a.id.as_str()).collect();
    let mut problems = Vec::new();
    for a in adrs {
        let superseded = a.status == adr::SUPERSEDED;
        if superseded && (!ids.contains(a.superseded_by.as_str()) || a.superseded_by == a.id) {
            problems.push(format!(
                "{} is superseded by {:?}, which is no other ADR",
                a.id, a.superseded_by
            ));
        } else if !superseded && !a.superseded_by.is_empty() {
            problems.push(format!(
                "{} names successor {} but its status is {:?}, not superseded",
                a.id, a.superseded_by, a.status
            ));
        }
    }

    problems
}

#[cfg(test)]
mod tests;
