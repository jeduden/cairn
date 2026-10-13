//! The ENG-16 steps: CI gates on the static checks and on each
//! component's build-time reach evidence. Only the core's row can pass
//! today, and its checks also run as gate tests in the engineering
//! crate's `static_analysis` target. The outline stays pending: the
//! browser room view client and its TypeScript checks arrive with M7,
//! the launcher's, room-view, peer, publish and bridge components'
//! evidence with M7 to M9, and the trusted-text check on restore
//! content with TrustedText in plan 2609292013. Two steps stay unbound
//! until then: the client importing only the core's generated types,
//! and restore content built only from trusted text.

use cucumber::{given, then, when};
use engineering::review_workflow::{self, Workflow};
use engineering::{Checkout, outcome, static_analysis};

use crate::World;
use crate::engineering::eng;

/// The component whose evidence each row reads, and what that evidence
/// must show it never does.
const EVIDENCE: [(&str, &str); 1] = [(
    "core",
    "opens any socket, or starts a program other than its own kernel worker",
)];

/// What the ENG-16 steps track: the CI workflow and the component whose
/// reach evidence CI reads.
#[derive(Debug, Default)]
pub struct Reach {
    flow: Workflow,
    component: String,
}

fn reach(w: &mut World) -> &mut Reach {
    w.section::<Reach>()
}

#[given(regex = r"^the CI workflow$")]
fn the_ci_workflow(w: &mut World) -> Result<(), String> {
    let checkout = Checkout::new(&testkit::repo_root());
    let flow = review_workflow::parse(&checkout.read(static_analysis::CI_WORKFLOW)?);
    eng(w).checkout = Some(checkout);
    reach(w).flow = flow;
    Ok(())
}

#[when(
    regex = r#"^CI reads the build-time reach evidence for the "([^"]+)", dependencies and start-up code included$"#
)]
fn reads_the_evidence(w: &mut World, component: String) -> Result<(), String> {
    if !EVIDENCE.iter().any(|(c, _)| *c == component) {
        return Err(format!(
            "no build-time reach evidence for the {component} yet: only the core exists"
        ));
    }
    reach(w).component = component;
    Ok(())
}

#[then(
    regex = r"^the workflow gates on cargo clippy, cargo fmt, cargo-deny, cargo-audit, the browser room view client's strict TypeScript compile and lint, and the custom checks$"
)]
fn gates_on_the_static_checks(w: &mut World) -> Result<(), String> {
    outcome(static_analysis::ci_gates(&reach(w).flow))?;
    Err("no CI job runs the browser room view client's strict TypeScript compile and lint: the client arrives with M7".to_owned())
}

#[then(regex = r"^every crate that parses untrusted content or decides trust forbids unsafe code$")]
fn crates_forbid_unsafe(w: &mut World) -> Result<(), String> {
    outcome(static_analysis::crates_forbid_unsafe(eng(w).checkout()?))
}

#[then(
    regex = r"^a build-time check fails when code that computes derived artifacts reads a clock or randomness$"
)]
fn derived_code_gated(w: &mut World) -> Result<(), String> {
    let checkout = eng(w).checkout()?.clone();
    outcome(static_analysis::derived_code_gated(
        &checkout,
        &reach(w).flow,
    ))
}

#[then(
    regex = r#"^CI fails when the evidence for the "([^"]+)" is missing or shows that it (.+)$"#
)]
fn evidence_gated(w: &mut World, component: String, violation: String) -> Result<(), String> {
    let Some((_, want)) = EVIDENCE.iter().find(|(c, _)| *c == component) else {
        return Err(format!(
            "no build-time reach evidence for the {component} yet"
        ));
    };
    if *want != violation {
        return Err(format!(
            "the {component}'s evidence shows that it never {want}, not that it never {violation}"
        ));
    }
    let checkout = eng(w).checkout()?.clone();
    outcome(static_analysis::core_reach_evidence(
        &checkout,
        &reach(w).flow,
    ))
}
