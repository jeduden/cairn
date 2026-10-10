//! Appendix B: which requirements serve each invariant, and how many
//! requirements each priority has.

use std::collections::BTreeMap;

use crate::{Error, Requirement, is_invariant, is_requirement_id, split_list, tables};

/// Requirement counts per priority, split the way Appendix B's count
/// table splits them: functional for §5–§6, engineering for §10. A
/// priority no requirement has is left out.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Counts {
    pub functional: BTreeMap<String, usize>,
    pub engineering: BTreeMap<String, usize>,
}

/// Reads Appendix B's coverage table in `body` (the table whose header
/// is "Invariant", "Requirements"), keyed by invariant, each value the
/// requirement ids listed for it.
///
/// # Errors
///
/// Fails on a malformed row, or when `body` has no such table.
pub fn invariant_coverage(body: &str) -> Result<BTreeMap<String, Vec<String>>, Error> {
    let table = tables(body)
        .into_iter()
        .find(|t| t.header == ["Invariant", "Requirements"])
        .ok_or_else(|| Error::Malformed("no invariant coverage table".into()))?;

    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for row in &table.rows {
        let malformed = || Error::Malformed(format!("line {}: malformed invariant row", row.line));
        let [invariant, ids] = row.cells.as_slice() else {
            return Err(malformed());
        };
        let invariant = leading_invariant(invariant).ok_or_else(malformed)?;
        let ids = split_list(ids, is_requirement_id, "requirement id")
            .map_err(|e| Error::Malformed(format!("line {}: {e}", row.line)))?;
        out.entry(invariant.to_owned()).or_default().extend(ids);
    }

    Ok(out)
}

/// The invariant a cell such as `**I1** — Nothing is lost` leads with.
fn leading_invariant(cell: &str) -> Option<&str> {
    let (invariant, _) = cell.strip_prefix("**")?.split_once("**")?;

    is_invariant(invariant).then_some(invariant)
}

/// Derives the same map as [`invariant_coverage`] from the
/// requirements' own Traces, for the families Appendix B covers (§5 and
/// §6): every family but NFR, ENG and ASM.
#[must_use]
pub fn traced_coverage(reqs: &[Requirement]) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for r in reqs.iter().filter(|r| in_appendix_b(&r.id)) {
        for invariant in &r.traces {
            out.entry(invariant.clone()).or_default().push(r.id.clone());
        }
    }

    out
}

fn in_appendix_b(id: &str) -> bool {
    !["NFR-", "ENG-", "ASM-"]
        .iter()
        .any(|family| id.starts_with(family))
}

/// Counts requirements by priority. NFR and ASM rows carry no priority
/// and are not counted.
#[must_use]
pub fn priority_counts(reqs: &[Requirement]) -> Counts {
    let mut out = Counts::default();
    for r in reqs.iter().filter(|r| !r.priority.is_empty()) {
        let class = if r.id.starts_with("ENG-") {
            &mut out.engineering
        } else {
            &mut out.functional
        };
        *class.entry(r.priority.clone()).or_default() += 1;
    }

    out
}

/// Reads Appendix B's requirement-count table in `body`: the rows P0,
/// P1 and P2, and their functional and engineering columns.
///
/// # Errors
///
/// Fails on a malformed count row, or when `body` has no such table.
pub fn stated_counts(body: &str) -> Result<Counts, Error> {
    let table = tables(body)
        .into_iter()
        .find(|t| t.header.len() == 3 && t.header[0] == "Priority")
        .ok_or_else(|| Error::Malformed("no requirement count table".into()))?;

    let mut out = Counts::default();
    for row in table.rows.iter().filter(|r| r.cells[0].starts_with('P')) {
        let malformed =
            |why: &str| Error::Malformed(format!("line {}: malformed count row{why}", row.line));
        let [priority, functional, engineering] = row.cells.as_slice() else {
            return Err(malformed(""));
        };
        let count = |cell: &str| {
            cell.parse::<usize>()
                .map_err(|e| malformed(&format!(": {cell:?}: {e}")))
        };
        // A zero is left out, the way priority_counts never creates an
        // entry for a priority no requirement has.
        for (class, n) in [
            (&mut out.functional, count(functional)?),
            (&mut out.engineering, count(engineering)?),
        ] {
            if n > 0 {
                class.insert(priority.clone(), n);
            }
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests;
