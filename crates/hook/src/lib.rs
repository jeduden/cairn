//! The hook handlers' entry point (§9.1): it names the hooks Cairn
//! installs, checks each hook input, and catches a handler's panic, so
//! `cairn hook` fails open on every path (ENG-04, I9). Each failure is
//! typed for the counter its audit entry feeds (OPS-01). The handlers
//! themselves, with the schema each hook input is checked against
//! (SEC-16), arrive with phase 9 of plan 2609292012; until then a
//! hook input that is a JSON object injects nothing.

use std::fmt;
use std::panic::{self, AssertUnwindSafe};

use audit::entry::{Counter, Kind, Occurrence};
use serde_json::Value;

/// A hook Cairn installs: only those through which it writes to an
/// agent (§9.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hook {
    SessionStart,
    UserPromptSubmit,
    PermissionRequest,
}

impl Hook {
    /// The hook the harness names `name`.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "SessionStart" => Some(Self::SessionStart),
            "UserPromptSubmit" => Some(Self::UserPromptSubmit),
            "PermissionRequest" => Some(Self::PermissionRequest),
            _ => None,
        }
    }

    /// The operation its audit entries name.
    #[must_use]
    pub fn operation(self) -> &'static str {
        match self {
            Self::SessionStart => "hook SessionStart",
            Self::UserPromptSubmit => "hook UserPromptSubmit",
            Self::PermissionRequest => "hook PermissionRequest",
        }
    }
}

/// Why a hook handler injected nothing; each feeds its own counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Failure {
    /// `cairn hook` named no hook Cairn installs.
    UnknownHook,
    /// The hook input is no JSON object.
    Malformed,
    /// The handler panicked, and the panic was caught.
    Panicked,
}

impl Failure {
    /// The audit entry it makes for `operation`.
    #[must_use]
    pub fn occurrence(self, operation: &'static str) -> Occurrence {
        let (kind, counter) = match self {
            Self::UnknownHook | Self::Malformed => (Kind::Rejected, Counter::HookRejected),
            Self::Panicked => (Kind::Failed, Counter::HookPanicked),
        };

        Occurrence::Degraded {
            kind,
            counter,
            operation,
            detail: self.detail(),
        }
    }

    /// Its fixed text.
    fn detail(self) -> &'static str {
        match self {
            Self::UnknownHook => "no such hook",
            Self::Malformed => "not a JSON object",
            Self::Panicked => "panic recovered",
        }
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.detail())
    }
}

impl std::error::Error for Failure {}

/// What `hook` writes to stdout for `hook_input`: nothing yet, as no
/// handler injects before phase 9.
///
/// # Errors
///
/// Fails when the hook input is no JSON object.
pub fn handle(_hook: Hook, hook_input: &str) -> Result<String, Failure> {
    match serde_json::from_str::<Value>(hook_input) {
        Ok(Value::Object(_)) => Ok(String::new()),
        _ => Err(Failure::Malformed),
    }
}

/// Runs `handler`, catching a panic in it as [`Failure::Panicked`] so
/// no panic leaves the hook's entry point (ENG-04). The caller keeps the
/// default panic message off stderr.
///
/// # Errors
///
/// What `handler` fails with, or the caught panic.
pub fn guarded(handler: impl FnOnce() -> Result<String, Failure>) -> Result<String, Failure> {
    panic::catch_unwind(AssertUnwindSafe(handler)).unwrap_or(Err(Failure::Panicked))
}

#[cfg(test)]
mod tests;
