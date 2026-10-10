//! The personas: §2.5's table of personas and their agents, and
//! Appendix C's table of which personas each requirement serves.

use std::collections::{BTreeMap, BTreeSet};

use crate::{Error, Row, is_requirement_id, numbered_once, split_list, tables};

/// Whether `s` is one persona, `U1` to `U9`.
fn is_persona(s: &str) -> bool {
    numbered_once(s, 'U')
}

/// Reads Appendix C's coverage table in `body` (the table whose header
/// is "Requirements", "Personas"), keyed by requirement id, each value
/// the personas that requirement serves. An id listed twice, a
/// malformed id, an empty persona list, a persona outside U1–U9 or one
/// named twice in a row is reported with its line.
///
/// # Errors
///
/// Fails on a malformed row, or when `body` has no such table.
pub fn persona_coverage(body: &str) -> Result<BTreeMap<String, Vec<String>>, Error> {
    let table = tables(body)
        .into_iter()
        .find(|t| t.header == ["Requirements", "Personas"])
        .ok_or_else(|| Error::Malformed("no persona coverage table".into()))?;

    let mut out = BTreeMap::new();
    for row in &table.rows {
        let at = |msg: String| Error::Malformed(format!("line {}: {msg}", row.line));
        let [ids, personas] = row.cells.as_slice() else {
            return Err(at("malformed persona coverage row".into()));
        };
        let personas = split_personas(personas).map_err(at)?;
        for id in split_list(ids, is_requirement_id, "requirement id").map_err(at)? {
            if out.contains_key(&id) {
                return Err(at(format!("{id} listed twice")));
            }
            out.insert(id, personas.clone());
        }
    }

    Ok(out)
}

/// Reads a Personas cell: a comma-separated, non-empty list of U1–U9,
/// each named once.
fn split_personas(cell: &str) -> Result<Vec<String>, String> {
    let out = split_list(cell, is_persona, "persona")?;
    let mut seen = BTreeSet::new();
    match out.iter().find(|p| !seen.insert(p.as_str())) {
        Some(p) => Err(format!("persona {p} listed twice")),
        None => Ok(out),
    }
}

/// Reads §2.5's persona table in `body` (the table whose header leads
/// with "#", "Persona", "Agent"), keyed by persona, each value the
/// agent that encodes it under `.claude/agents`. A persona listed twice,
/// or an agent two personas name, is reported with its line.
///
/// # Errors
///
/// Fails on a malformed row, or when `body` has no such table.
pub fn persona_agents(body: &str) -> Result<BTreeMap<String, String>, Error> {
    let table = tables(body)
        .into_iter()
        .find(|t| t.header.len() >= 3 && t.header[..3] == ["#", "Persona", "Agent"])
        .ok_or_else(|| Error::Malformed("no persona table".into()))?;

    let mut out = BTreeMap::new();
    let mut named = BTreeSet::new();
    for row in &table.rows {
        let (persona, agent) = persona_row(row)?;
        if out.contains_key(&persona) {
            return Err(Error::Malformed(format!(
                "line {}: {persona} listed twice",
                row.line
            )));
        }
        if !named.insert(agent.clone()) {
            return Err(Error::Malformed(format!(
                "line {}: {agent} named twice",
                row.line
            )));
        }
        out.insert(persona, agent);
    }

    Ok(out)
}

/// Reads one §2.5 row: its persona, U1 to U9, and the agent its
/// backticked Agent cell names.
fn persona_row(row: &Row) -> Result<(String, String), Error> {
    let agent = |cell: &str| {
        let name = cell.strip_prefix('`')?.strip_suffix('`')?;
        let rest = name.strip_prefix("persona-")?;
        (!rest.is_empty() && rest.chars().all(|c| c.is_ascii_lowercase() || c == '-'))
            .then(|| name.to_owned())
    };
    match row.cells.as_slice() {
        [persona, _, cell, ..] if is_persona(persona) => agent(cell).map(|a| (persona.clone(), a)),
        _ => None,
    }
    .ok_or_else(|| Error::Malformed(format!("line {}: malformed persona row", row.line)))
}

#[cfg(test)]
mod tests;
