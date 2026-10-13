//! ENG-27: the drift registry covers every check that keeps the records
//! in step. The drift suite injects the cases; these checks hold the
//! registry itself, so they run without mdsmith.

use std::path::Path;

use drift::Case;
use scenario::Scenario;

/// The step a scenario opens with when it inspects the repository
/// itself.
pub const CHECKOUT_STEP: &str = "the repository checkout";

/// The gate tests outside the scenarios that ENG-27 holds to a drift
/// case: the parse gates of the SRS and the ADRs, the requirement–scenario gate, the Appendix B check, the
/// persona gates of Appendix C and §2.5, the finding ledger's check
/// that every closed domain-model finding stays in the text, and the
/// core executable's dependency reach (SEC-01), and the three isolation
/// gates of ENG-14: the scan for a read of the home around the guard,
/// the decoy home CI runs the suite with, and the scan of the agent
/// instruction files.
pub const GATE_TESTS: [&str; 11] = [
    "specification_parses",
    "decision_records_parse",
    "specification_and_features_agree",
    "appendix_b_matches_the_traces",
    "appendix_c_covers_every_requirement",
    "personas_match_the_agents",
    "finding_ledger_is_carried",
    "the_core_executable_links_only_crates_its_reach_allows",
    "only_the_isolation_guard_reads_the_home",
    "the_ci_test_job_runs_with_a_decoy_home",
    "the_agent_instructions_direct_no_destructive_command_at_real_state",
];

/// Refuses a case whose injection no longer finds its target, so a case
/// cannot rot into injecting nothing.
#[must_use]
pub fn drifts_apply(root: &Path, cases: &[Case]) -> Vec<String> {
    cases
        .iter()
        .filter_map(|c| {
            drift::validate(root, &c.injection)
                .err()
                .map(|e| format!("drift case {:?}: {e}", c.name))
        })
        .collect()
}

/// The ids of the non-pending scenarios that open on the checkout.
#[must_use]
pub fn checkout_scenarios(scenarios: &[Scenario]) -> Vec<String> {
    scenarios
        .iter()
        .filter(|s| !s.pending && s.steps.first().is_some_and(|step| step == CHECKOUT_STEP))
        .map(|s| s.id.clone())
        .collect()
}

/// Requires a drift case guarding each of `names`.
#[must_use]
pub fn guarded(cases: &[Case], names: &[impl AsRef<str>]) -> Vec<String> {
    names
        .iter()
        .map(AsRef::as_ref)
        .filter(|name| !cases.iter().any(|c| c.guards == *name))
        .map(|name| format!("{name} has no drift case"))
        .collect()
}

#[cfg(test)]
mod tests;
