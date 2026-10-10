//! Registers, for every check that keeps the specification, its
//! scenarios and the repository's records in step, the drift that check
//! exists to catch (ENG-27). A case is data: one injection into a copy
//! of the repository, the check to run on it, and the message the check
//! must fail with. The `suite` test target injects each case; the
//! ENG-27 scenario checks the registry itself. It is repository tooling
//! and never links into a shipped executable.

mod cases;

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

pub use cases::cases;

/// What an [`Injection`] does to its file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// Swaps the first occurrence of `old` for `new`.
    Replace,
    /// Deletes the file.
    Remove,
    /// Copies the file to `new`, which must not exist yet.
    Copy,
}

/// One change to a file, its path relative to the repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Injection {
    pub op: Op,
    pub file: &'static str,
    pub old: &'static str,
    pub new: &'static str,
}

impl Injection {
    /// Swaps the first `old` in `file` for `new`.
    #[must_use]
    pub fn replace(file: &'static str, old: &'static str, new: &'static str) -> Self {
        Self {
            op: Op::Replace,
            file,
            old,
            new,
        }
    }

    /// Deletes `file`.
    #[must_use]
    pub fn remove(file: &'static str) -> Self {
        Self {
            op: Op::Remove,
            file,
            old: "",
            new: "",
        }
    }

    /// Copies `file` to `to`.
    #[must_use]
    pub fn copy(file: &'static str, to: &'static str) -> Self {
        Self {
            op: Op::Copy,
            file,
            old: "",
            new: to,
        }
    }
}

/// The command a case runs in the injected copy.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Check {
    pub tool: String,
    pub args: Vec<String>,
}

impl Check {
    fn new(tool: &str, args: &[&str]) -> Self {
        Self {
            tool: tool.to_owned(),
            args: args.iter().map(|a| (*a).to_owned()).collect(),
        }
    }
}

impl fmt::Display for Check {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.tool, self.args.join(" "))
    }
}

/// One registered drift.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Case {
    /// Says what drifted.
    pub name: &'static str,
    /// The requirement id or gate test the case proves.
    pub guards: &'static str,
    pub injection: Injection,
    pub check: Check,
    /// Text the check must print when it fails.
    pub want: &'static str,
    /// Marks a drift the checks miss today. The suite requires the
    /// check to pass on it, so the mark fails the build the day a check
    /// starts catching it.
    pub known_gap: bool,
}

/// Runs the one scenario tagged `id` through the cucumber-rs runner.
#[must_use]
pub fn scenario(id: &str) -> Check {
    Check::new(
        "cargo",
        &[
            "test",
            "--locked",
            "-p",
            "scenario",
            "--test",
            "bdd",
            "--",
            "--tags",
            &format!("@{id}"),
        ],
    )
}

/// Runs the one test `name` in the `target` test target of `package`.
#[must_use]
pub fn cargo_test(package: &str, target: &str, name: &str) -> Check {
    Check::new(
        "cargo",
        &[
            "test", "--locked", "-p", package, "--test", target, "--", "--exact", name,
        ],
    )
}

/// Lints every Markdown file, generated sections included.
#[must_use]
pub fn mdsmith() -> Check {
    Check::new("mdsmith", &["check", "."])
}

/// What validating or applying an injection can fail with.
#[derive(Debug)]
pub enum Error {
    /// The file could not be read or written.
    Io { file: String, source: io::Error },
    /// The text a replacement swaps is not in the file.
    NotFound { file: String, old: String },
    /// A copy's target exists already.
    CopyTargetExists { file: String, new: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { file, source } => write!(f, "drift: {file}: {source}"),
            Self::NotFound { file, old } => write!(f, "drift: {file}: {old:?} not found"),
            Self::CopyTargetExists { file, new } => {
                write!(f, "drift: {file}: copy target {new} already exists")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::NotFound { .. } | Self::CopyTargetExists { .. } => None,
        }
    }
}

/// Reports whether `injection` can be applied to the repository at
/// `root` without changing anything, so a case whose target moved
/// fails loudly instead of injecting nothing.
///
/// # Errors
///
/// Fails when the file is missing, the text a replacement swaps is
/// gone, or a copy's target exists.
pub fn validate(root: &Path, injection: &Injection) -> Result<(), Error> {
    check_target(root, injection).map(drop)
}

/// Checks `injection` against the repository at `root` and returns the
/// file's current contents.
fn check_target(root: &Path, injection: &Injection) -> Result<String, Error> {
    let file = injection.file;
    let body = fs::read_to_string(root.join(file)).map_err(|source| Error::Io {
        file: file.into(),
        source,
    })?;
    match injection.op {
        Op::Replace if injection.old.is_empty() || !body.contains(injection.old) => {
            Err(Error::NotFound {
                file: file.into(),
                old: injection.old.into(),
            })
        }
        Op::Copy if root.join(injection.new).exists() => Err(Error::CopyTargetExists {
            file: file.into(),
            new: injection.new.into(),
        }),
        Op::Replace | Op::Remove | Op::Copy => Ok(body),
    }
}

/// Makes `injection` in the repository at `root`.
///
/// # Errors
///
/// Fails as [`validate`] does, or when the file cannot be written.
pub fn apply(root: &Path, injection: &Injection) -> Result<(), Error> {
    let body = check_target(root, injection)?;
    let path = root.join(injection.file);
    let written = match injection.op {
        Op::Remove => fs::remove_file(path),
        Op::Copy => fs::write(root.join(injection.new), body),
        Op::Replace => fs::write(path, body.replacen(injection.old, injection.new, 1)),
    };

    written.map_err(|source| Error::Io {
        file: injection.file.into(),
        source,
    })
}

#[cfg(test)]
mod tests;
