//! The isolation guard (ENG-14). Test code resolves the home, a Cairn
//! home and Claude Code's configuration through [`resolve`] and nowhere
//! else. Anything that does not lie under the system's temporary
//! directory is real state, and resolving it aborts the process before
//! a path to it exists.

use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Something a test must never reach for real.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    /// The user's home: `HOME`.
    Home,
    /// Cairn's home: `CAIRN_HOME`, else `.cairn` in the home.
    CairnHome,
    /// Claude Code's configuration directory: `CLAUDE_CONFIG_DIR`, else
    /// `.claude` in the home.
    ClaudeConfig,
    /// Claude Code's global state file: `.claude.json` in the
    /// configuration directory, else in the home.
    ClaudeJson,
}

/// Why the guard refuses to resolve a place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// No `HOME` to resolve the place from.
    Unset,
    /// The place is not under the temporary directory `temp`.
    Outside {
        place: Place,
        path: PathBuf,
        temp: PathBuf,
    },
}

/// The value of an environment variable, as a process reads it.
pub type Var<'a> = &'a dyn Fn(&str) -> Option<OsString>;

impl Place {
    /// Every place the guard keeps tests out of.
    pub const ALL: [Self; 4] = [
        Self::Home,
        Self::CairnHome,
        Self::ClaudeConfig,
        Self::ClaudeJson,
    ];

    /// The name a probe takes this place by.
    #[must_use]
    pub fn name(self) -> &'static str {
        ""
    }

    /// The place a probe names, if any.
    #[must_use]
    pub fn parse(_name: &str) -> Option<Self> {
        None
    }
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

/// Resolves `place` from `var` and requires it to lie under `temp`.
///
/// # Errors
///
/// Refuses a place that is real state, or has no home to resolve from.
pub fn check(_place: Place, _var: Var<'_>, _temp: &Path) -> Result<PathBuf, Refusal> {
    Ok(PathBuf::new())
}

/// [`check`], and on a refusal writes it to `report` and calls `abort`.
pub fn check_or_abort(
    _place: Place,
    _var: Var<'_>,
    _temp: &Path,
    _report: &mut dyn Write,
    _abort: fn() -> !,
) -> PathBuf {
    PathBuf::new()
}

/// Resolves `place` for this process, aborting it when the place is
/// real state.
#[must_use]
pub fn resolve(_place: Place) -> PathBuf {
    PathBuf::new()
}

#[cfg(test)]
mod tests;
