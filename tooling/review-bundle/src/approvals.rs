//! The `Security approvals` check (ENG-21): a change to a
//! security-sensitive crate needs two approvals, one of them from the
//! designated security reviewer.
//!
//! A ruleset asks for one approval count for the whole branch and
//! CODEOWNERS one owner per path, so neither can say "two, for these
//! paths". This check reads the pull request's changed files, its
//! reviews and main's CODEOWNERS, and fails when a change reaches a
//! crate directory CODEOWNERS assigns to the security reviewer without
//! them. The ruleset requires the check by name.
//!
//! The security-sensitive crates are the crate directories CODEOWNERS
//! names whole (`/crates/<name>/`); the `engineering` crate's gate
//! keeps that list equal to the one ENG-11 names. An owner counts by
//! user login; a team owner is never met, so naming one fails closed.

use serde_json::Value;

use crate::owners;

/// One review, as the forge reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Review {
    pub user: String,
    pub state: String,
    pub commit_id: String,
}

/// What the check found when it passes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The change reaches no security-sensitive crate.
    NotSecurity,
    /// It does, and two approvals including the owner's stand on the head.
    Approved,
}

/// The approvals a security-sensitive change needs.
const REQUIRED: usize = 2;

/// The security-sensitive crate directories in `codeowners`, each with
/// its owners: whole-directory patterns under `/crates/` with no glob.
#[must_use]
pub fn security_patterns(codeowners: &str) -> Vec<(String, Vec<String>)> {
    codeowners
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| {
            let mut fields = l.split_whitespace();
            let pattern = fields.next()?;
            let whole = pattern.starts_with("/crates/") && pattern.ends_with('/');
            (whole && !pattern.contains('*'))
                .then(|| (pattern.to_owned(), fields.map(str::to_owned).collect()))
        })
        .collect()
}

/// Lower-cased login of an owner or reviewer, without the `@`.
fn login(name: &str) -> String {
    name.trim_start_matches('@').to_ascii_lowercase()
}

/// The logins whose latest review of `head` approves, other than the
/// author's.
fn approvers(reviews: &[Review], author: &str, head: &str) -> Vec<String> {
    let mut latest: Vec<(String, &Review)> = Vec::new();
    for review in reviews.iter().filter(|r| {
        matches!(
            r.state.as_str(),
            "APPROVED" | "CHANGES_REQUESTED" | "DISMISSED"
        )
    }) {
        let who = login(&review.user);
        latest.retain(|(u, _)| *u != who);
        latest.push((who, review));
    }
    latest
        .into_iter()
        .filter(|(who, r)| r.state == "APPROVED" && r.commit_id == head && *who != login(author))
        .map(|(who, _)| who)
        .collect()
}

/// Decides the check for a pull request by `author` at `head` that
/// changes `files`, given its `reviews`. The error names what is
/// missing.
///
/// # Errors
///
/// Returns the shortfall when the change reaches a security-sensitive
/// crate without two approvals, one from each touched crate's owner.
pub fn check(
    codeowners: &str,
    files: &[String],
    author: &str,
    head: &str,
    reviews: &[Review],
) -> Result<Verdict, String> {
    let touched: Vec<(String, Vec<String>)> = security_patterns(codeowners)
        .into_iter()
        .filter(|(pattern, _)| files.iter().any(|f| owners::owns(pattern, f)))
        .collect();
    if touched.is_empty() {
        return Ok(Verdict::NotSecurity);
    }
    let approved = approvers(reviews, author, head);
    if approved.len() < REQUIRED {
        return Err(format!(
            "the change reaches a security-sensitive crate and needs {REQUIRED} approvals on {head}, not the author's; it has {}",
            approved.len()
        ));
    }
    for (pattern, owners) in &touched {
        if !owners.iter().any(|o| approved.contains(&login(o))) {
            return Err(format!(
                "{pattern} needs an approval on {head} from one of {}",
                owners.join(", ")
            ));
        }
    }

    Ok(Verdict::Approved)
}

/// Reads the forge's list of a pull request's reviews.
///
/// # Errors
///
/// Returns why the text is not an array of reviews with a user login,
/// a state and a commit.
pub fn parse_reviews(json: &str) -> Result<Vec<Review>, String> {
    let value: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let list = value.as_array().ok_or("reviews are not an array")?;
    list.iter()
        .map(|r| {
            let text = |v: Option<&Value>| v.and_then(Value::as_str).map(str::to_owned);
            Ok(Review {
                user: text(r.pointer("/user/login")).ok_or("a review without a user login")?,
                state: text(r.get("state")).ok_or("a review without a state")?,
                commit_id: text(r.get("commit_id")).ok_or("a review without a commit")?,
            })
        })
        .collect()
}

/// Reads the forge's list of a pull request's changed files; a rename
/// counts under both names.
///
/// # Errors
///
/// Returns why the text is not an array of files with a name.
pub fn parse_files(json: &str) -> Result<Vec<String>, String> {
    let value: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let list = value.as_array().ok_or("files are not an array")?;
    let mut files = Vec::new();
    for f in list {
        let name = f
            .get("filename")
            .and_then(Value::as_str)
            .ok_or("a file without a name")?;
        files.push(name.to_owned());
        if let Some(old) = f.get("previous_filename").and_then(Value::as_str) {
            files.push(old.to_owned());
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests;
