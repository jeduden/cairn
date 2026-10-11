//! Compares main's tree with the pull request's: which requirements and
//! scenarios changed, which the diff names, the route the review takes,
//! and the findings a machine can decide alone.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::{BLOCKING, Finding, Snapshot, diff, owners};

/// The invariants file ENG-29 guards.
pub const INVARIANTS: &str = "docs/srs/invariants.md";

/// The requirement whose text names the requirements marked for I2
/// review.
const ENG_29: &str = "ENG-29";

/// Where the decision records live.
const ADR_DIR: &str = "docs/adr/";

/// How deep a review goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Class {
    /// Only Markdown no stakeholder path holds: plans, notes, guides.
    Light,
    /// Anything else no stakeholder path holds: product code, scenarios.
    Standard,
    /// A path CODEOWNERS assigns: the SRS, its gates, CI, the agent
    /// instructions, the security-sensitive crates.
    Deep,
}

impl Class {
    /// The name the review workflow reads.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Standard => "standard",
            Self::Deep => "deep",
        }
    }
}

/// How one requirement or scenario differs between the two trees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Change {
    Added,
    Removed,
    Changed,
}

/// What the pull request does, as the bundle and the gate read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Analysis {
    pub class: Class,
    /// The paths CODEOWNERS assigns among the changed ones.
    pub owned: Vec<String>,
    pub paths: Vec<String>,
    pub requirements: BTreeMap<String, Change>,
    pub scenarios: BTreeMap<String, Change>,
    /// Every id the change touches: a changed requirement or scenario,
    /// or one a changed line names, that either tree defines.
    pub touched: Vec<String>,
    pub findings: Vec<Finding>,
}

/// Compares `base`, main's tree, with `head`, the pull request's, whose
/// diff against main is `diff`.
#[must_use]
pub fn analyse(base: &Snapshot, head: &Snapshot, diff: &str) -> Analysis {
    let paths = diff::changed_paths(diff);
    let patterns = owners::patterns(&base.codeowners);
    let owned: Vec<String> = paths
        .iter()
        .filter(|p| patterns.iter().any(|pat| owners::owns(pat, p)))
        .cloned()
        .collect();
    let requirements = requirement_changes(base, head);
    let scenarios = scenario_changes(base, head);
    let known: BTreeSet<&str> = base.ids().chain(head.ids()).collect();
    let touched: BTreeSet<String> = requirements
        .keys()
        .chain(scenarios.keys())
        .cloned()
        .chain(diff::mentioned_ids(diff))
        .filter(|id| known.contains(id.as_str()))
        .collect();
    let mut findings = ratchet(base, head);
    findings.extend(eng_29(base, head, &paths, &requirements));
    findings.sort();

    Analysis {
        class: classify(&paths, &owned),
        owned,
        paths,
        requirements,
        scenarios,
        touched: touched.into_iter().collect(),
        findings,
    }
}

/// The route: deep when a stakeholder path changed, light when only
/// Markdown did, standard otherwise.
fn classify(paths: &[String], owned: &[String]) -> Class {
    if !owned.is_empty() {
        Class::Deep
    } else if paths.iter().all(|p| p.ends_with(".md")) {
        Class::Light
    } else {
        Class::Standard
    }
}

/// Keys each of `items` by `id`, keeping what `value` reads of it.
fn keyed<'a, T, V>(
    items: &'a [T],
    id: impl Fn(&'a T) -> &'a str,
    value: impl Fn(&'a T) -> V,
) -> BTreeMap<&'a str, V> {
    items.iter().map(|i| (id(i), value(i))).collect()
}

/// How `base` and `head` differ, id by id.
fn changes<V: PartialEq>(
    base: &BTreeMap<&str, V>,
    head: &BTreeMap<&str, V>,
) -> BTreeMap<String, Change> {
    let ids: BTreeSet<&&str> = base.keys().chain(head.keys()).collect();
    ids.into_iter()
        .filter_map(|id| {
            let change = match (base.get(*id), head.get(*id)) {
                (None, _) => Change::Added,
                (_, None) => Change::Removed,
                (Some(a), Some(b)) if a != b => Change::Changed,
                _ => return None,
            };
            Some(((*id).to_owned(), change))
        })
        .collect()
}

/// The requirements whose row differs: priority, method, traces or
/// text. Where a row sits does not count.
fn requirement_changes(base: &Snapshot, head: &Snapshot) -> BTreeMap<String, Change> {
    let row = |r: &srs::Requirement| {
        (
            r.priority.clone(),
            r.verify.clone(),
            r.traces.clone(),
            r.text.clone(),
        )
    };
    changes(
        &keyed(&base.requirements, |r| r.id.as_str(), row),
        &keyed(&head.requirements, |r| r.id.as_str(), row),
    )
}

/// The scenarios whose name, tags or steps differ.
fn scenario_changes(base: &Snapshot, head: &Snapshot) -> BTreeMap<String, Change> {
    let body = |s: &scenario::Scenario| {
        (
            s.name.clone(),
            s.priority.clone(),
            s.invariants.clone(),
            s.pending,
            s.steps.clone(),
        )
    };
    changes(
        &keyed(&base.scenarios, |s| s.id.as_str(), body),
        &keyed(&head.scenarios, |s| s.id.as_str(), body),
    )
}

/// A scenario live on main stays live: the pull request may not delete
/// it or put it back on `@pending` (CLAUDE.md, Requirements and
/// Scenarios). The backlog only shrinks.
fn ratchet(base: &Snapshot, head: &Snapshot) -> Vec<Finding> {
    let now: BTreeMap<&str, &scenario::Scenario> =
        head.scenarios.iter().map(|s| (s.id.as_str(), s)).collect();
    base.scenarios
        .iter()
        .filter(|s| !s.pending)
        .filter_map(|was| match now.get(was.id.as_str()) {
            None => Some(Finding {
                path: was.path.display().to_string(),
                line: was.line,
                severity: BLOCKING,
                body: format!(
                    "the {} scenario is live on main and this change removes it",
                    was.id
                ),
            }),
            Some(is) if is.pending => Some(Finding {
                path: is.path.display().to_string(),
                line: is.line,
                severity: BLOCKING,
                body: format!(
                    "the {} scenario is live on main and this change puts it back on @pending",
                    was.id
                ),
            }),
            Some(_) => None,
        })
        .collect()
}

/// ENG-29: a change to an invariant, or to a requirement ENG-29 marks
/// for I2 review, lands only with an accepted decision record, changed
/// in the same pull request, that records the security reviewer.
fn eng_29(
    base: &Snapshot,
    head: &Snapshot,
    paths: &[String],
    requirements: &BTreeMap<String, Change>,
) -> Vec<Finding> {
    let marked = marked_for_i2_review(base);
    let mut guarded: Vec<String> = requirements
        .keys()
        .filter(|id| marked.contains(id))
        .cloned()
        .collect();
    if paths.iter().any(|p| p == INVARIANTS) {
        guarded.insert(0, "an invariant".to_owned());
    }
    if guarded.is_empty() {
        return Vec::new();
    }
    let recorded = paths
        .iter()
        .filter(|p| p.starts_with(ADR_DIR))
        .filter_map(|p| head.decisions.get(p).map(|body| (p, body)))
        .any(|(p, body)| records_security_review(Path::new(p), body));
    if recorded {
        return Vec::new();
    }

    vec![Finding {
        path: INVARIANTS.to_owned(),
        line: 0,
        severity: BLOCKING,
        body: format!(
            "this change touches {} but changes no accepted ADR under docs/adr that records the security reviewer's approval (ENG-29)",
            guarded.join(", ")
        ),
    }]
}

/// Whether `body` is an accepted decision record that names the
/// security reviewer.
fn records_security_review(path: &Path, body: &str) -> bool {
    adr::parse(path, body).is_ok_and(|a| a.status == adr::ACCEPTED)
        && body.to_lowercase().contains("security reviewer")
}

/// The ids ENG-29's row names besides itself: the requirements marked
/// for I2 review.
fn marked_for_i2_review(base: &Snapshot) -> Vec<String> {
    base.requirements
        .iter()
        .find(|r| r.id == ENG_29)
        .map(|r| diff::mentioned_ids(&format!("+{}", r.text)))
        .unwrap_or_default()
        .into_iter()
        .filter(|id| id != ENG_29)
        .collect()
}

#[cfg(test)]
mod tests;
