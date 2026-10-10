//! Reads the domain model's finding ledger and checks that every closed
//! finding theme is still carried in the text. A theme is closed as
//! fixed, deferred (an open question and a milestone) or accepted (a
//! residual risk), and each closed theme cites the sentences that close
//! it; a sentence that leaves its file reopens the theme. The check
//! keeps a decision from living only in a review note, and a trim from
//! dropping a decided rule silently. It is repository tooling and never
//! links into a shipped executable.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io;

use serde::Deserialize;

/// The theme is still open.
pub const OPEN: &str = "open";
/// The theme is closed by a fix in the text.
pub const FIXED: &str = "fixed";
/// The theme waits on an open question and a milestone.
pub const DEFERRED: &str = "deferred";
/// The theme rests on a residual risk.
pub const ACCEPTED: &str = "accepted";

/// Every finding theme the reviews raised.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct Ledger {
    #[serde(default)]
    pub themes: Vec<Theme>,
}

/// One recurring finding and how it was closed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Theme {
    pub id: String,
    pub title: String,
    pub status: String,
    /// Where a deferred theme waits: an open question.
    pub oq: String,
    /// Where a deferred theme waits: a milestone.
    pub milestone: String,
    /// The residual risk an accepted theme rests on.
    pub risk: String,
    /// The sentences that close the theme.
    pub texts: Vec<Text>,
}

/// One sentence a closed theme cites, by file relative to the
/// repository root.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Text {
    pub file: String,
    pub quote: String,
}

/// What reading or checking the ledger can fail with.
#[derive(Debug)]
pub enum Error {
    /// The ledger itself could not be read.
    Read { path: String, source: io::Error },
    /// The ledger is not the JSON this reader takes.
    Parse(serde_json::Error),
    /// A file a closed theme cites could not be read.
    Cited {
        theme: String,
        file: String,
        source: io::Error,
    },
    /// Themes whose identity, status or closure is malformed.
    Malformed(Vec<String>),
    /// Closed themes whose texts left their files, as `id: file: quote`.
    NotCarried(Vec<String>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => write!(f, "ledger: read {path}: {source}"),
            Self::Parse(source) => write!(f, "ledger: parse: {source}"),
            Self::Cited {
                theme,
                file,
                source,
            } => write!(f, "ledger: {theme} cites {file}: {source}"),
            Self::Malformed(problems) => write!(f, "ledger: malformed: {}", problems.join("\n")),
            Self::NotCarried(missing) => {
                write!(
                    f,
                    "closed findings no longer carried in the text:\n{}",
                    missing.join("\n")
                )
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read { source, .. } | Self::Cited { source, .. } => Some(source),
            Self::Parse(source) => Some(source),
            Self::Malformed(_) | Self::NotCarried(_) => None,
        }
    }
}

/// Reads a ledger from its JSON.
///
/// # Errors
///
/// Fails when `body` is not a ledger.
pub fn parse(body: &str) -> Result<Ledger, Error> {
    serde_json::from_str(body).map_err(Error::Parse)
}

/// Reads the ledger at `path` through `read`, which maps a path
/// relative to the repository root to the file's text.
///
/// # Errors
///
/// Fails when the ledger cannot be read or parsed.
pub fn load(read: impl Fn(&str) -> io::Result<String>, path: &str) -> Result<Ledger, Error> {
    let body = read(path).map_err(|source| Error::Read {
        path: path.to_owned(),
        source,
    })?;

    parse(&body)
}

/// Validates `ledger` and confirms that the repository `read` reaches
/// still carries every closed theme's texts.
///
/// # Errors
///
/// Fails when a theme is malformed, a cited file cannot be read, or a
/// cited text is gone.
pub fn check(ledger: &Ledger, read: impl Fn(&str) -> io::Result<String>) -> Result<(), Error> {
    let problems = ledger.validate();
    if !problems.is_empty() {
        return Err(Error::Malformed(problems));
    }
    let missing = ledger.missing(read)?;
    if !missing.is_empty() {
        return Err(Error::NotCarried(missing));
    }

    Ok(())
}

impl Ledger {
    /// Reports every theme whose identity, status or closure is
    /// malformed.
    #[must_use]
    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();
        let mut seen = BTreeSet::new();
        for t in &self.themes {
            if t.id.is_empty() {
                problems.push(format!("theme {:?} has no id", t.title));
                continue;
            }
            if !seen.insert(&t.id) {
                problems.push(format!("theme id {} used twice", t.id));
            }
            problems.extend(t.validate());
        }

        problems
    }

    /// Lists, as `id: file: quote`, every cited text of a closed theme
    /// that its file no longer carries. Whitespace differences, such as
    /// a rewrapped line, do not count.
    ///
    /// # Errors
    ///
    /// Fails when a cited file cannot be read.
    pub fn missing(&self, read: impl Fn(&str) -> io::Result<String>) -> Result<Vec<String>, Error> {
        let mut out = Vec::new();
        let mut texts: BTreeMap<&str, String> = BTreeMap::new();
        for t in self.themes.iter().filter(|t| t.status != OPEN) {
            for x in &t.texts {
                if !texts.contains_key(x.file.as_str()) {
                    let body = read(&x.file).map_err(|source| Error::Cited {
                        theme: t.id.clone(),
                        file: x.file.clone(),
                        source,
                    })?;
                    texts.insert(&x.file, normalize(&body));
                }
                if !texts[x.file.as_str()].contains(&normalize(&x.quote)) {
                    out.push(format!("{}: {}: {}", t.id, x.file, x.quote));
                }
            }
        }

        Ok(out)
    }
}

impl Theme {
    /// Checks one theme's status and the closure it requires.
    fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();
        match self.status.as_str() {
            OPEN => return problems,
            FIXED => {}
            DEFERRED if !numbered(&self.oq, "OQ-") || !numbered(&self.milestone, "M") => {
                problems.push(format!(
                    "{}: deferred without an OQ-n and an Mn milestone",
                    self.id
                ));
            }
            ACCEPTED if !numbered(&self.risk, "R") => {
                problems.push(format!("{}: accepted without a residual risk Rn", self.id));
            }
            DEFERRED | ACCEPTED => {}
            other => return vec![format!("{}: unknown status {other:?}", self.id)],
        }
        if self.texts.is_empty() {
            problems.push(format!("{}: {} but cites no text", self.id, self.status));
        }
        for x in &self.texts {
            if x.file.is_empty() || x.quote.trim().is_empty() {
                problems.push(format!(
                    "{}: a cited text needs a file and a quote",
                    self.id
                ));
            }
        }

        problems
    }
}

/// Whether `s` is `prefix` followed by one or more digits.
fn numbered(s: &str, prefix: &str) -> bool {
    s.strip_prefix(prefix)
        .is_some_and(|rest| !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit()))
}

/// Collapses every run of whitespace to one space.
fn normalize(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests;
