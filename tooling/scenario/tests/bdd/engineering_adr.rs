//! The ENG-18 and ENG-26 steps: what the engineering scenarios check in
//! the direct dependencies and the decision records under docs/adr.

use cucumber::{then, when};
use engineering::{decisions, dependencies, outcome};

use crate::World;
use crate::engineering::eng;

#[when(regex = r#"^the direct dependencies are read from "([^"]+)"$"#)]
fn read_direct_deps(w: &mut World, file: String) -> Result<(), String> {
    let deps = dependencies::go_direct_deps(&eng(w).checkout()?.read(&file)?);
    eng(w).deps = deps;
    Ok(())
}

#[when(regex = r#"^the ADRs are read from "([^"]+)"$"#)]
fn read_adrs(w: &mut World, dir: String) -> Result<(), String> {
    let adrs = adr::load(&eng(w).checkout()?.root.join(&dir)).map_err(|e| e.to_string())?;
    let e = eng(w);
    (e.adr_dir, e.adrs) = (dir, adrs);
    Ok(())
}

#[then(regex = r"^each direct dependency is named by exactly one accepted ADR$")]
fn deps_named_once(w: &mut World) -> Result<(), String> {
    let e = eng(w);
    outcome(dependencies::deps_named_once(&e.deps, &e.adrs))
}

#[then(regex = r"^every module an accepted ADR names is a direct dependency$")]
fn modules_are_deps(w: &mut World) -> Result<(), String> {
    let e = eng(w);
    outcome(dependencies::modules_are_deps(&e.deps, &e.adrs))
}

#[then(
    regex = r"^each named module has a purpose, a license and a maintenance status, and its ADR weighs alternatives$"
)]
fn deps_justified(w: &mut World) -> Result<(), String> {
    outcome(dependencies::deps_justified(&eng(w).adrs))
}

#[then(regex = r"^each license is on the allow-list the ENG-18 requirement states$")]
fn licenses_allowed(w: &mut World) -> Result<(), String> {
    let allowed = dependencies::allow_list(&eng(w).checkout()?.requirement("ENG-18")?.text)?;
    outcome(dependencies::licenses_allowed(&eng(w).adrs, &allowed))
}

#[then(regex = r"^the direct dependencies stay within the target the ENG-18 requirement states$")]
fn within_target(w: &mut World) -> Result<(), String> {
    let target = dependencies::dependency_target(&eng(w).checkout()?.requirement("ENG-18")?.text)?;
    dependencies::within_target(&eng(w).deps, target)
}

#[then(regex = r#"^"([^"]+)" lists every ADR that names a module$"#)]
fn lists_dependency_adrs(w: &mut World, file: String) -> Result<(), String> {
    let body = eng(w).checkout()?.read(&file)?;
    let e = eng(w);
    outcome(dependencies::lists_dependency_adrs(
        &body, &file, &e.adr_dir, &e.adrs,
    ))
}

#[then(regex = r"^every ADR has an id, a title, a status and a summary$")]
fn adrs_complete(w: &mut World) -> Result<(), String> {
    outcome(decisions::adrs_complete(&eng(w).adrs))
}

#[then(regex = r"^every ADR's file is named for its id$")]
fn adrs_named_for_id(w: &mut World) -> Result<(), String> {
    outcome(decisions::adrs_named_for_id(&eng(w).adrs))
}

#[then(regex = r"^every ADR's status is proposed, accepted or superseded$")]
fn adr_statuses_valid(w: &mut World) -> Result<(), String> {
    outcome(decisions::adr_statuses_valid(&eng(w).adrs))
}

#[then(regex = r"^no two ADRs share an id$")]
fn adr_ids_unique(w: &mut World) -> Result<(), String> {
    outcome(decisions::adr_ids_unique(&eng(w).adrs))
}

#[then(regex = r"^every superseded ADR names an ADR that exists as its successor$")]
fn superseded_names_successor(w: &mut World) -> Result<(), String> {
    outcome(decisions::superseded_names_successor(&eng(w).adrs))
}
