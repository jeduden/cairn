//! ENG-21: main only moves through reviewed, signed pull requests.
//!
//! Branch protection is a setting of the forge, not a file of the
//! repository, so the repository commits the protection it wants as a
//! ruleset in GitHub's export format, [`RULESET`]. The scenario's steps
//! read that file, and CODEOWNERS (see [`crate::codeowners`]). The
//! `protection-check` executable compares the committed ruleset with
//! the live one, so the setting cannot weaken unseen. Applying the
//! committed ruleset is the stakeholder's act.

pub mod cli;

use serde_json::Value;

use crate::Checkout;

/// The committed ruleset for main, in GitHub's ruleset export format.
pub const RULESET: &str = ".github/rulesets/main.json";

/// The check every merge waits for: the job that passes only when every
/// CI job passed.
pub const CI_CHECK: &str = "CI";

/// The check that would hold the second approval security-sensitive
/// crates need. A ruleset holds one approval count for the whole
/// branch, so a per-path count has to come from a required check that
/// reads the pull request's reviews; no such check exists yet.
pub const SECURITY_CHECK: &str = "Security approvals";

/// Reads and parses the committed ruleset.
///
/// # Errors
///
/// Fails when the file cannot be read or is not JSON.
pub fn load(checkout: &Checkout) -> Result<Value, String> {
    parse(&checkout.read(RULESET)?)
}

/// Parses a ruleset, committed or fetched live.
///
/// # Errors
///
/// Fails when `text` is not a JSON object.
pub fn parse(text: &str) -> Result<Value, String> {
    let value: Value = serde_json::from_str(text).map_err(|e| format!("the ruleset: {e}"))?;
    if value.is_object() {
        Ok(value)
    } else {
        Err("the ruleset is not a JSON object".to_owned())
    }
}

/// The rule of `ruleset` with this `type`.
fn rule<'a>(ruleset: &'a Value, kind: &str) -> Option<&'a Value> {
    ruleset
        .get("rules")?
        .as_array()?
        .iter()
        .find(|r| r.get("type").and_then(Value::as_str) == Some(kind))
}

/// The parameter `name` of the rule `kind`.
fn param<'a>(ruleset: &'a Value, kind: &str, name: &str) -> Option<&'a Value> {
    rule(ruleset, kind)?.get("parameters")?.get(name)
}

/// The strings at `path` in `value`, empty when there are none.
fn strings<'a>(value: &'a Value, path: &[&str]) -> Vec<&'a str> {
    let mut at = Some(value);
    for key in path {
        at = at.and_then(|v| v.get(key));
    }
    at.and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

/// The contexts of the required status checks.
fn required_checks(ruleset: &Value) -> Vec<&str> {
    param(ruleset, "required_status_checks", "required_status_checks")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|c| c.get("context")?.as_str())
                .collect()
        })
        .unwrap_or_default()
}

/// Whether the ruleset has a rule of this type.
fn has(ruleset: &Value, kind: &str) -> bool {
    rule(ruleset, kind).is_some()
}

/// Problems that let a push reach main without a pull request: the
/// ruleset is not active, does not govern the default branch, lets an
/// actor bypass it, or lacks a rule that closes a way around the pull
/// request (deleting the branch, rewriting its history, merging
/// without CI).
#[must_use]
pub fn direct_pushes_rejected(ruleset: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    if ruleset.get("enforcement").and_then(Value::as_str) != Some("active") {
        problems.push("the ruleset's enforcement is not active".to_owned());
    }
    if ruleset.get("target").and_then(Value::as_str) != Some("branch") {
        problems.push("the ruleset does not target branches".to_owned());
    }
    let include = strings(ruleset, &["conditions", "ref_name", "include"]);
    if !include
        .iter()
        .any(|r| ["~DEFAULT_BRANCH", "refs/heads/main"].contains(r))
    {
        problems.push("the ruleset does not include the main branch".to_owned());
    }
    if !strings(ruleset, &["conditions", "ref_name", "exclude"]).is_empty() {
        problems.push("the ruleset excludes a branch from itself".to_owned());
    }
    if ruleset
        .get("bypass_actors")
        .and_then(Value::as_array)
        .is_some_and(|a| !a.is_empty())
    {
        problems.push("the ruleset lets an actor bypass it".to_owned());
    }
    for kind in ["pull_request", "non_fast_forward", "deletion"] {
        if !has(ruleset, kind) {
            problems.push(format!("the ruleset has no {kind} rule"));
        }
    }
    if !required_checks(ruleset).contains(&CI_CHECK) {
        problems.push(format!("the ruleset does not require the {CI_CHECK} check"));
    }

    problems
}

/// Problems with signed commits: the ruleset has no `required_signatures`
/// rule.
#[must_use]
pub fn commits_signed(ruleset: &Value) -> Vec<String> {
    if has(ruleset, "required_signatures") {
        Vec::new()
    } else {
        vec!["the ruleset has no required_signatures rule".to_owned()]
    }
}

/// Problems with the approval every pull request needs: at least one,
/// from a reviewer other than the author. GitHub never counts an
/// author's own approval, and `require_last_push_approval` keeps a
/// pusher of the last commits from approving them. A new push dismisses
/// earlier approvals, and a code owner's approval is required for what
/// CODEOWNERS owns.
#[must_use]
pub fn approval_from_another_reviewer(ruleset: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    let count = param(ruleset, "pull_request", "required_approving_review_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if count < 1 {
        problems.push("the pull_request rule requires no approving review".to_owned());
    }
    for flag in [
        "require_last_push_approval",
        "dismiss_stale_reviews_on_push",
        "require_code_owner_review",
    ] {
        if param(ruleset, "pull_request", flag).and_then(Value::as_bool) != Some(true) {
            problems.push(format!("the pull_request rule does not set {flag}"));
        }
    }

    problems
}

/// Problems with the second approval security-sensitive crates need.
/// The ruleset's approval count holds for the whole branch, and
/// CODEOWNERS asks one owner per path, so neither can say "two
/// approvals, one from the security reviewer" for some paths only. A
/// required check named [`SECURITY_CHECK`], which reads the reviews of
/// a pull request that touches those crates, can; without it the second
/// approval is a convention, not a rule.
#[must_use]
pub fn security_crates_need_two_approvals(ruleset: &Value) -> Vec<String> {
    if required_checks(ruleset).contains(&SECURITY_CHECK) {
        Vec::new()
    } else {
        vec![format!(
            "the ruleset requires no {SECURITY_CHECK:?} check, so nothing enforces two approvals on the security-sensitive crates"
        )]
    }
}

/// Problems where `live`, the ruleset GitHub holds, is weaker than
/// `committed`: an inactive ruleset, a missing rule, a lower approval
/// count, a flag switched off, a required check dropped, a branch
/// excluded or an actor allowed to bypass it. Stronger is not a
/// problem. GitHub shows `bypass_actors` only to a caller with write
/// access to the ruleset, so a live ruleset without it is not judged
/// on that.
#[must_use]
pub fn weaker_than(committed: &Value, live: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    if committed.get("enforcement") != live.get("enforcement") {
        problems.push(format!(
            "enforcement is {} live, {} committed",
            show(live.get("enforcement")),
            show(committed.get("enforcement"))
        ));
    }
    for include in strings(committed, &["conditions", "ref_name", "include"]) {
        if !strings(live, &["conditions", "ref_name", "include"]).contains(&include) {
            problems.push(format!("live does not include {include}"));
        }
    }
    for exclude in strings(live, &["conditions", "ref_name", "exclude"]) {
        if !strings(committed, &["conditions", "ref_name", "exclude"]).contains(&exclude) {
            problems.push(format!("live excludes {exclude}"));
        }
    }
    for kind in rule_types(committed) {
        if has(live, kind) {
            problems.extend(weaker_parameters(committed, live, kind));
        } else {
            problems.push(format!("live has no {kind} rule"));
        }
    }
    if live
        .get("bypass_actors")
        .and_then(Value::as_array)
        .is_some_and(|a| !a.is_empty())
    {
        problems.push("live lets an actor bypass the ruleset".to_owned());
    }

    problems
}

/// The types of the rules of `ruleset`.
fn rule_types(ruleset: &Value) -> Vec<&str> {
    ruleset
        .get("rules")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|r| r.get("type")?.as_str()).collect())
        .unwrap_or_default()
}

/// How a missing or odd value reads in a message.
fn show(value: Option<&Value>) -> String {
    value.map_or_else(|| "absent".to_owned(), Value::to_string)
}

/// Problems where the live rule `kind` has weaker parameters than the
/// committed one: a number below it, a `true` flag not true, a required
/// check missing.
fn weaker_parameters(committed: &Value, live: &Value, kind: &str) -> Vec<String> {
    let Some(wanted) = rule(committed, kind)
        .and_then(|r| r.get("parameters"))
        .and_then(Value::as_object)
    else {
        return Vec::new();
    };
    let mut problems = Vec::new();
    for (name, want) in wanted {
        let got = param(live, kind, name);
        if let Some(n) = want.as_u64() {
            if got.and_then(Value::as_u64) < Some(n) {
                problems.push(format!(
                    "live {kind} {name} is {}, committed {n}",
                    show(got)
                ));
            }
        } else if want.as_bool() == Some(true) && got.and_then(Value::as_bool) != Some(true) {
            problems.push(format!("live {kind} does not set {name}"));
        }
    }
    if kind == "required_status_checks" {
        let live_checks = required_checks(live);
        problems.extend(
            required_checks(committed)
                .into_iter()
                .filter(|c| !live_checks.contains(c))
                .map(|c| format!("live does not require the {c} check")),
        );
    }

    problems
}

#[cfg(test)]
mod tests;
