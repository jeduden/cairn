//! ENG-18: every direct dependency is justified by exactly one accepted
//! ADR, with an allow-listed license, within the stated target.

use std::collections::{BTreeMap, BTreeSet};

use adr::Adr;

use crate::list;

/// The module paths `go.mod` requires without an `// indirect` marker,
/// in either the block or the one-line form.
#[must_use]
pub fn go_direct_deps(go_mod: &str) -> Vec<String> {
    let mut deps = Vec::new();
    let mut in_block = false;
    for line in go_mod.lines().map(str::trim) {
        let line = match line {
            "require (" => {
                in_block = true;
                continue;
            }
            ")" if in_block => {
                in_block = false;
                continue;
            }
            _ => match line.strip_prefix("require ") {
                Some(rest) => rest,
                None if in_block => line,
                None => continue,
            },
        };
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 2 && !line.contains("// indirect") {
            deps.push(fields[0].to_owned());
        }
    }

    deps
}

/// The crates the workspace's members depend on directly from a
/// registry, read from `cargo metadata --no-deps`: normal, build and
/// dev dependencies alike, and never a member reached by path.
///
/// # Errors
///
/// Fails when `metadata` is not cargo's JSON.
pub fn cargo_direct_deps(metadata: &str) -> Result<Vec<String>, String> {
    let value: serde_json::Value =
        serde_json::from_str(metadata).map_err(|e| format!("read cargo metadata: {e}"))?;
    let deps: BTreeSet<&str> = value["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|p| p["dependencies"].as_array().into_iter().flatten())
        .filter(|d| {
            d["source"]
                .as_str()
                .is_some_and(|s| s.starts_with("registry+"))
        })
        .filter_map(|d| d["name"].as_str())
        .collect();

    Ok(deps.into_iter().map(str::to_owned).collect())
}

/// The records whose decision is in force.
fn accepted(adrs: &[Adr]) -> impl Iterator<Item = &Adr> {
    adrs.iter().filter(|a| a.status == adr::ACCEPTED)
}

/// Finds, for each direct dependency, the one accepted record that
/// justifies it.
#[must_use]
pub fn deps_named_once(deps: &[String], adrs: &[Adr]) -> Vec<String> {
    let mut by: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for a in accepted(adrs) {
        for m in &a.modules {
            by.entry(&m.name).or_default().push(&a.id);
        }
    }

    deps.iter()
        .filter_map(|dep| {
            let ids = by.get(dep.as_str()).map(Vec::as_slice).unwrap_or_default();
            (ids.len() != 1).then(|| {
                format!(
                    "{dep} is named by {} accepted ADRs {}, want exactly one",
                    ids.len(),
                    list(ids)
                )
            })
        })
        .collect()
}

/// Refuses an accepted record that justifies a module the manifests no
/// longer require directly: a stale decision.
#[must_use]
pub fn modules_are_deps(deps: &[String], adrs: &[Adr]) -> Vec<String> {
    accepted(adrs)
        .flat_map(|a| a.modules.iter().map(move |m| (a, m)))
        .filter(|(_, m)| !deps.contains(&m.name))
        .map(|(a, m)| {
            format!(
                "{} names {}, which is not a direct dependency",
                a.id, m.name
            )
        })
        .collect()
}

/// Checks each accepted module row is filled in, and that its record
/// weighs alternatives.
#[must_use]
pub fn deps_justified(adrs: &[Adr]) -> Vec<String> {
    let mut problems = Vec::new();
    for a in accepted(adrs) {
        if !a.modules.is_empty() && a.sections.get("Alternatives").is_none_or(String::is_empty) {
            problems.push(format!("{} weighs no alternatives", a.id));
        }
        for m in &a.modules {
            for (column, value) in [
                ("Purpose", &m.purpose),
                ("License", &m.license),
                ("Maintenance", &m.maintenance),
            ] {
                if value.is_empty() {
                    problems.push(format!("{} leaves {column:?} empty for {}", a.id, m.name));
                }
            }
        }
    }

    problems
}

/// Checks every accepted module's license against `allowed`.
#[must_use]
pub fn licenses_allowed(adrs: &[Adr], allowed: &[String]) -> Vec<String> {
    accepted(adrs)
        .flat_map(|a| &a.modules)
        .filter(|m| !license_allowed(&m.license, allowed))
        .map(|m| {
            format!(
                "{} is licensed {:?}, not on the allow-list {}",
                m.name,
                m.license,
                list(allowed)
            )
        })
        .collect()
}

/// Whether `license` is on `allowed`. A family name admits only its
/// named permissive variants, never a bare family or any other id
/// sharing its prefix: "BSD" admits BSD-2-Clause and BSD-3-Clause, not
/// BSD-4-Clause or BSD-Protection. An SPDX `OR` expression, such as
/// "MIT OR Apache-2.0", offers a choice, so one allowed choice admits
/// it.
#[must_use]
pub fn license_allowed(license: &str, allowed: &[String]) -> bool {
    license.split(" OR ").any(|choice| {
        allowed.iter().any(|a| match a.as_str() {
            "BSD" => ["BSD-2-Clause", "BSD-3-Clause"].contains(&choice),
            a => choice == a,
        })
    })
}

/// The licenses ENG-18's text allows: the list in `allow-list (...)`.
///
/// # Errors
///
/// Fails when the text states no allow-list.
pub fn allow_list(eng18: &str) -> Result<Vec<String>, String> {
    let (_, rest) = eng18
        .split_once("allow-list (")
        .ok_or("ENG-18 does not state its license allow-list")?;
    let (list, _) = rest
        .split_once(')')
        .ok_or("ENG-18 does not state its license allow-list")?;

    Ok(list.split(", ").map(str::to_owned).collect())
}

/// The most direct dependencies ENG-18's text allows: the number in
/// `≤ N direct dependencies`.
///
/// # Errors
///
/// Fails when the text states no target.
pub fn dependency_target(eng18: &str) -> Result<usize, String> {
    eng18
        .split("≤ ")
        .skip(1)
        .find_map(|rest| {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            let stated = rest[digits.len()..].starts_with(" direct dependencies");
            stated.then(|| digits.parse().ok()).flatten()
        })
        .ok_or_else(|| "ENG-18 does not state its dependency target".to_owned())
}

/// Checks the direct dependencies against the target.
///
/// # Errors
///
/// Fails when there are more than `target`.
pub fn within_target(deps: &[String], target: usize) -> Result<(), String> {
    if deps.len() > target {
        return Err(format!(
            "{} direct dependencies, want at most {target}: {}",
            deps.len(),
            list(deps)
        ));
    }

    Ok(())
}

/// Checks `body`, the text of `file`, links every record in `adr_dir`
/// that names a module, the way its catalog renders them.
#[must_use]
pub fn lists_dependency_adrs(body: &str, file: &str, adr_dir: &str, adrs: &[Adr]) -> Vec<String> {
    adrs.iter()
        .filter(|a| !a.modules.is_empty())
        .filter_map(|a| {
            let name = a
                .path
                .file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_default();
            let link = format!("{adr_dir}/{name}");
            (!body.contains(&format!("({link})"))).then(|| format!("{file} does not link {link}"))
        })
        .collect()
}

#[cfg(test)]
pub(crate) mod tests;
