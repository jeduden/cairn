//! The hook handlers' entry point (§9.1).

use std::fmt;

use audit::entry::Occurrence;

/// A hook Cairn installs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hook {
    SessionStart,
    UserPromptSubmit,
    PermissionRequest,
}

impl Hook {
    /// The hook named `name`.
    #[must_use]
    pub fn parse(_name: &str) -> Option<Self> {
        None
    }

    /// The operation its audit entries name.
    #[must_use]
    pub fn operation(self) -> &'static str {
        ""
    }
}

/// Why a hook handler failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Failure {
    UnknownHook,
    Malformed,
    Panicked,
}

impl Failure {
    /// The audit entry it makes.
    #[must_use]
    pub fn occurrence(self, _operation: &'static str) -> Occurrence {
        Occurrence::CounterAck
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl std::error::Error for Failure {}

/// Handles `hook_input`.
///
/// # Errors
///
/// Fails.
pub fn handle(_hook: Hook, _hook_input: &str) -> Result<String, Failure> {
    Err(Failure::Panicked)
}

/// Runs `handler`.
///
/// # Errors
///
/// Fails.
pub fn guarded(handler: impl FnOnce() -> Result<String, Failure>) -> Result<String, Failure> {
    handler()
}

#[cfg(test)]
mod tests;
