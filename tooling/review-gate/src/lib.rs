//! The review gate between a reviewing agent and the approval it may
//! earn (ENG-21, ENG-28). The agent only writes a structured review
//! outcome; this crate, run by the workflow step that alone holds the
//! reviewer app's key, decides what review to post. It approves only an
//! approving outcome with no blocking finding, on the head the agent
//! read, once CI passed there. It is repository tooling and never
//! links into a shipped executable.

pub mod cli;

use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Write as _;

use serde::{Deserialize, Deserializer, Serialize};

/// The `verdict` that asks for approval.
pub const APPROVE_VERDICT: &str = "approve";
/// The `verdict` that asks for changes.
pub const REQUEST_CHANGES_VERDICT: &str = "request_changes";

/// A finding that withholds approval whatever the verdict says.
pub const BLOCKING: &str = "blocking";
/// A finding that is reported but withholds nothing.
pub const NIT: &str = "nit";

/// The review event that approves, as GitHub names it.
pub const APPROVE: &str = "APPROVE";
/// The review event that requests changes, as GitHub names it.
pub const REQUEST_CHANGES: &str = "REQUEST_CHANGES";

/// The check the branch ruleset requires: the job that passes only when
/// every CI job passed.
const CI_CHECK: &str = "CI";

/// The reviewing agent's structured output.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ReviewOutcome {
    pub verdict: String,
    pub summary: String,
    #[serde(deserialize_with = "null_as_default")]
    pub findings: Vec<Finding>,
}

/// One problem the agent found, at a path and line of the reviewed
/// tree; line 0 means the whole file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Finding {
    pub path: String,
    pub line: i64,
    pub severity: String,
    pub body: String,
}

/// One check run on the reviewed head, as GitHub reports it. A commit
/// can carry several runs of one check, when CI ran on it twice;
/// `started_at`, an RFC 3339 UTC time, orders them, and `id`, which
/// GitHub assigns in creation order, breaks a tie within one second.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct CheckRun {
    pub id: i64,
    #[serde(deserialize_with = "null_as_default")]
    pub name: String,
    #[serde(deserialize_with = "null_as_default")]
    pub status: String,
    #[serde(deserialize_with = "null_as_default")]
    pub conclusion: String,
    #[serde(deserialize_with = "null_as_default")]
    pub started_at: String,
}

/// Everything the gate decides from.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Input {
    pub outcome: ReviewOutcome,
    /// The head commit the agent read.
    pub reviewed: String,
    /// The pull request's head commit now.
    pub head: String,
    pub check_runs: Vec<CheckRun>,
}

/// The pull request review to post, in the shape GitHub's
/// create-review endpoint takes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Review {
    pub commit_id: String,
    pub event: String,
    pub body: String,
}

/// What the gate refuses.
#[derive(Debug)]
pub enum Error {
    /// The agent's output does not fit the review outcome's schema.
    Outcome(String),
    /// A line of the check runs is not a check run.
    CheckRun {
        line: usize,
        source: serde_json::Error,
    },
    /// The required CI check has not passed on the reviewed head.
    Ci(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Outcome(msg) => write!(f, "review: malformed review outcome: {msg}"),
            Self::CheckRun { line, source } => write!(f, "review: check run line {line}: {source}"),
            Self::Ci(head) => write!(f, "review: CI has not passed on the reviewed head: {head}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CheckRun { source, .. } => Some(source),
            Self::Outcome(_) | Self::Ci(_) => None,
        }
    }
}

/// Reads a JSON `null` as the type's default, the way GitHub reports a
/// running check's conclusion.
fn null_as_default<'de, D: Deserializer<'de>, T: Default + Deserialize<'de>>(
    d: D,
) -> Result<T, D::Error> {
    Option::<T>::deserialize(d).map(Option::unwrap_or_default)
}

/// Decodes the agent's output strictly: one object, no unknown field,
/// and every value one the schema names.
///
/// # Errors
///
/// Fails on anything the schema does not allow.
pub fn parse_outcome(data: &str) -> Result<ReviewOutcome, Error> {
    let outcome: ReviewOutcome =
        serde_json::from_str(data).map_err(|e| Error::Outcome(e.to_string()))?;
    validate(&outcome)?;

    Ok(outcome)
}

/// Checks every value in `outcome` is one the schema names.
fn validate(outcome: &ReviewOutcome) -> Result<(), Error> {
    if outcome.verdict != APPROVE_VERDICT && outcome.verdict != REQUEST_CHANGES_VERDICT {
        return Err(Error::Outcome(format!(
            "verdict {:?} is neither {APPROVE_VERDICT} nor {REQUEST_CHANGES_VERDICT}",
            outcome.verdict
        )));
    }
    if outcome.summary.trim().is_empty() {
        return Err(Error::Outcome("no summary".into()));
    }
    for (i, f) in outcome.findings.iter().enumerate() {
        if f.severity != BLOCKING && f.severity != NIT {
            return Err(Error::Outcome(format!(
                "finding {i}: severity {:?}",
                f.severity
            )));
        }
        if f.path.is_empty() || f.body.trim().is_empty() {
            return Err(Error::Outcome(format!(
                "finding {i} has no path or no body"
            )));
        }
        if f.line < 0 {
            return Err(Error::Outcome(format!("finding {i}: line {}", f.line)));
        }
    }

    Ok(())
}

/// Reads one check run per line, as `gh api --jq` prints them; blank
/// lines are skipped.
///
/// # Errors
///
/// Fails on a line that is not a check run.
pub fn parse_check_runs(data: &str) -> Result<Vec<CheckRun>, Error> {
    data.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(i, line)| {
            serde_json::from_str(line).map_err(|source| Error::CheckRun {
                line: i + 1,
                source,
            })
        })
        .collect()
}

/// Returns the review to post, or `None` when the pull request moved
/// past the reviewed head: the run for the newer head decides instead.
///
/// # Errors
///
/// A head whose CI has not passed is an error, so the step fails loudly
/// rather than posting anything.
pub fn decide(input: &Input) -> Result<Option<Review>, Error> {
    if input.head != input.reviewed {
        return Ok(None);
    }
    let runs = latest(&input.check_runs);
    if !runs.iter().any(|r| passed_ci(r)) {
        return Err(Error::Ci(input.reviewed.clone()));
    }
    let failed: Vec<&CheckRun> = runs.into_iter().filter(|r| failed(r)).collect();
    let approves = input.outcome.verdict == APPROVE_VERDICT
        && failed.is_empty()
        && !input.outcome.findings.iter().any(is_blocking);
    let event = if approves { APPROVE } else { REQUEST_CHANGES };

    Ok(Some(Review {
        commit_id: input.reviewed.clone(),
        event: event.to_owned(),
        body: render(&input.outcome, &failed, event, &input.reviewed),
    }))
}

/// Keeps the newest run of each check, in the order the checks first
/// appear. An older run that a newer one replaced, cancelled or failed,
/// says nothing about the head as it stands.
fn latest(runs: &[CheckRun]) -> Vec<&CheckRun> {
    let mut out: Vec<&CheckRun> = Vec::new();
    let mut at: BTreeMap<&str, usize> = BTreeMap::new();
    for r in runs {
        match at.get(r.name.as_str()) {
            None => {
                at.insert(&r.name, out.len());
                out.push(r);
            }
            Some(&i) if newer(r, out[i]) => out[i] = r,
            Some(_) => {}
        }
    }

    out
}

/// Whether run `a` started after run `b`, by start time and then by
/// run id.
fn newer(a: &CheckRun, b: &CheckRun) -> bool {
    (&a.started_at, a.id) > (&b.started_at, b.id)
}

/// Whether `r` is the required CI check, passed.
fn passed_ci(r: &CheckRun) -> bool {
    r.name == CI_CHECK && r.status == "completed" && r.conclusion == "success"
}

/// Whether `f` withholds approval.
fn is_blocking(f: &Finding) -> bool {
    f.severity == BLOCKING
}

/// Whether `r` completed and neither passed nor was skipped. A check
/// still running, this review among them, is not counted: the required
/// CI check has already passed.
fn failed(r: &CheckRun) -> bool {
    r.status == "completed" && !["success", "neutral", "skipped"].contains(&r.conclusion.as_str())
}

/// Writes the review body: the outcome and the head it covers, the
/// agent's summary, its blocking findings, then its nits, then the
/// checks that failed.
fn render(outcome: &ReviewOutcome, failed: &[&CheckRun], event: &str, head: &str) -> String {
    let title = if event == APPROVE {
        "Approved"
    } else {
        "Changes requested"
    };
    let short = head.get(..12).unwrap_or(head);
    let mut b = format!(
        "**{title}** by the review agent on `{short}`.\n\n{}\n",
        defuse_mentions(&outcome.summary)
    );
    for severity in [BLOCKING, NIT] {
        for f in outcome.findings.iter().filter(|f| f.severity == severity) {
            let _ = write!(
                b,
                "\n- {severity} `{}`: {}",
                location(f),
                defuse_mentions(&f.body)
            );
        }
    }
    for r in failed {
        let _ = write!(b, "\n- check `{}`: {}", r.name, r.conclusion);
    }

    b
}

/// Names a finding's file, and its line when it has one.
fn location(f: &Finding) -> String {
    if f.line == 0 {
        f.path.clone()
    } else {
        format!("{}:{}", f.path, f.line)
    }
}

/// Wraps each bare `@word` outside a code span in backticks, so a review
/// quoting a scenario tag such as `@I2` mentions no GitHub user of that
/// name. Text inside backticks and the `@` of an email address stay as
/// they are.
fn defuse_mentions(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut in_code = false;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let bare = !in_code
            && c == '@'
            && !(i > 0 && word_char(chars[i - 1]))
            && chars.get(i + 1).is_some_and(|next| word_char(*next));
        if !bare {
            in_code ^= c == '`';
            out.push(c);
            i += 1;
            continue;
        }
        let end = (i + 1..chars.len())
            .find(|&j| !word_char(chars[j]) && chars[j] != '-')
            .unwrap_or(chars.len());
        out.push('`');
        out.extend(&chars[i..end]);
        out.push('`');
        i = end;
    }

    out
}

/// Whether `c` can be part of a GitHub user name.
fn word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests;
