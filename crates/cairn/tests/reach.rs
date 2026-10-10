//! The dependency half of the core executable's reach evidence (SEC-01,
//! ENG-16): no crate links into `cairn` that could open a socket or
//! start a program behind clippy's back. clippy's disallowed types,
//! in this crate's clippy.toml, hold its own code; this holds what it
//! links. A dependency joins `REACHABLE` in the same change that adds
//! it, under review, with its reach weighed.

#![allow(
    clippy::disallowed_types,
    reason = "the test runs cargo metadata; the shipped code starts no program"
)]
#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::collections::BTreeSet;
use std::process::Command;

use serde_json::Value;

/// The crates the core executable may link, by name.
const REACHABLE: [&str; 0] = [];

/// The packages `root` links: its normal dependencies and theirs, by
/// name. Dev and build dependencies never reach the executable.
fn linked(metadata: &Value, root: &str) -> Result<BTreeSet<String>, String> {
    let packages = metadata["packages"].as_array().ok_or("no packages")?;
    let name_of = |id: &str| {
        packages
            .iter()
            .find(|p| p["id"] == id)
            .and_then(|p| p["name"].as_str())
            .map(str::to_owned)
    };
    let nodes = metadata["resolve"]["nodes"]
        .as_array()
        .ok_or("no resolve")?;
    let root_id = packages
        .iter()
        .find(|p| p["name"] == root)
        .and_then(|p| p["id"].as_str())
        .ok_or(format!("no package {root}"))?;
    let (mut seen, mut todo) = (BTreeSet::new(), vec![root_id.to_owned()]);
    while let Some(id) = todo.pop() {
        let node = nodes.iter().find(|n| n["id"] == id.as_str());
        let deps = node
            .and_then(|n| n["deps"].as_array())
            .into_iter()
            .flatten();
        for dep in deps.filter(|d| is_normal(d)) {
            let pkg = dep["pkg"].as_str().unwrap_or_default().to_owned();
            if seen.insert(pkg.clone()) {
                todo.push(pkg);
            }
        }
    }

    Ok(seen.iter().filter_map(|id| name_of(id)).collect())
}

/// Whether a resolved dependency links into the executable: it has a
/// normal (kind null) edge.
fn is_normal(dep: &Value) -> bool {
    dep["dep_kinds"]
        .as_array()
        .is_some_and(|kinds| kinds.iter().any(|k| k["kind"].is_null()))
}

#[test]
fn the_core_executable_links_only_crates_its_reach_allows() {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let out = Command::new(cargo)
        .args(["metadata", "--format-version", "1", "--locked", "--offline"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let metadata: Value = serde_json::from_slice(&out.stdout).unwrap();

    let extra: Vec<String> = linked(&metadata, "cairn")
        .unwrap()
        .into_iter()
        .filter(|name| !REACHABLE.contains(&name.as_str()))
        .collect();

    assert!(
        extra.is_empty(),
        "cairn links crates its reach does not allow: {extra:?}"
    );
}

#[test]
fn linked_follows_normal_dependencies_only() {
    let metadata = serde_json::json!({
        "packages": [
            {"id": "a", "name": "cairn"},
            {"id": "b", "name": "lib"},
            {"id": "c", "name": "deep"},
            {"id": "d", "name": "devonly"},
        ],
        "resolve": {"nodes": [
            {"id": "a", "deps": [
                {"pkg": "b", "dep_kinds": [{"kind": null}]},
                {"pkg": "d", "dep_kinds": [{"kind": "dev"}]},
            ]},
            {"id": "b", "deps": [{"pkg": "c", "dep_kinds": [{"kind": null}]}]},
            {"id": "c", "deps": [{"pkg": "b", "dep_kinds": [{"kind": null}]}]},
        ]},
    });

    let got = linked(&metadata, "cairn").unwrap();

    assert_eq!(got, BTreeSet::from(["deep".to_owned(), "lib".to_owned()]));
    assert_eq!(
        linked(&metadata, "other"),
        Err("no package other".to_owned())
    );
    assert_eq!(
        linked(&serde_json::json!({}), "cairn"),
        Err("no packages".to_owned())
    );
}
