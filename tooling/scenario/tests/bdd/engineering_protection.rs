//! The ENG-21 steps: main only moves through reviewed, signed pull
//! requests.
//!
//! Branch protection is a setting of the forge, so "the repository's
//! branch protection for main" is read from the ruleset the repository
//! commits, `.github/rulesets/main.json`, and from CODEOWNERS. That the
//! forge holds the same ruleset is the nightly `protection` job's
//! check; applying the ruleset is the stakeholder's act.
//!
//! The two-approval step reads the same records: the ruleset requires
//! the `Security approvals` check and its workflow reports it.

use cucumber::{given, then};
use engineering::{Checkout, branch_protection, codeowners, outcome};
use serde_json::Value;

use crate::World;

/// The ruleset the scenario read.
#[derive(Debug, Default)]
struct Protection {
    ruleset: Value,
}

fn protection(w: &mut World) -> &mut Protection {
    w.section::<Protection>()
}

fn checkout() -> Checkout {
    Checkout::new(&testkit::repo_root())
}

#[given(regex = r"^the repository's branch protection for main$")]
fn branch_protection_for_main(w: &mut World) -> Result<(), String> {
    protection(w).ruleset = branch_protection::load(&checkout())?;
    Ok(())
}

#[then(regex = r"^direct pushes are rejected$")]
fn direct_pushes_rejected(w: &mut World) -> Result<(), String> {
    outcome(branch_protection::direct_pushes_rejected(
        &protection(w).ruleset,
    ))
}

#[then(regex = r"^commits must be signed$")]
fn commits_signed(w: &mut World) -> Result<(), String> {
    outcome(branch_protection::commits_signed(&protection(w).ruleset))
}

#[then(regex = r"^every pull request needs an approval from a reviewer other than its author$")]
fn approval_from_another_reviewer(w: &mut World) -> Result<(), String> {
    outcome(branch_protection::approval_from_another_reviewer(
        &protection(w).ruleset,
    ))
}

#[then(
    regex = r"^CODEOWNERS names the stakeholder on the requirement text and on every path that enforces it$"
)]
fn stakeholder_owns_the_enforcing_paths(_w: &mut World) -> Result<(), String> {
    outcome(codeowners::stakeholder_owns_the_enforcing_paths(&checkout()))
}

#[then(regex = r"^CODEOWNERS names no code owner on any other path$")]
fn no_owner_on_any_other_path(_w: &mut World) -> Result<(), String> {
    outcome(codeowners::no_owner_on_any_other_path(&checkout()))
}

/// Two approvals for some paths only is not a number a ruleset or
/// CODEOWNERS holds, so the `Security approvals` check counts them: the
/// ruleset requires it, the workflow reports it from main, and
/// CODEOWNERS gives the security-sensitive crates to the security
/// reviewer whose approval it asks for.
#[then(
    regex = r"^changes to security-sensitive crates need two approvals, one from the designated security reviewer$"
)]
fn security_crates_need_two_approvals(w: &mut World) -> Result<(), String> {
    let mut problems = codeowners::security_crates_owned_by_the_security_reviewer(&checkout());
    problems.extend(branch_protection::security_crates_need_two_approvals(
        &protection(w).ruleset,
    ));
    problems.extend(
        match checkout().read(branch_protection::SECURITY_WORKFLOW) {
            Ok(workflow) => branch_protection::security_check_reported(&workflow),
            Err(e) => vec![e],
        },
    );
    outcome(problems)
}
