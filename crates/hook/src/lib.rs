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

/// The most bytes a hook input may hold (SEC-05).
pub const HOOK_INPUT_LIMIT: usize = 1 << 20;

/// The deepest a hook input's JSON may nest (SEC-05).
pub const NESTING_LIMIT: usize = 64;

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
    /// The hook input exceeds [`HOOK_INPUT_LIMIT`].
    TooLarge,
    /// The hook input nests deeper than [`NESTING_LIMIT`].
    TooDeep,
}

impl Failure {
    /// The audit entry it makes for `operation`.
    #[must_use]
    pub fn occurrence(self, operation: &'static str) -> Occurrence<'static> {
        let (kind, counter) = match self {
            Self::UnknownHook | Self::Malformed => (Kind::Rejected, Counter::HookRejected),
            Self::Panicked => (Kind::Failed, Counter::HookPanicked),
            Self::TooLarge => (Kind::Rejected, Counter::HookInputTooLarge),
            Self::TooDeep => (Kind::Rejected, Counter::HookInputTooDeep),
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
            Self::TooLarge => "hook input exceeds 1 MiB",
            Self::TooDeep => "hook input nesting exceeds depth 64",
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
/// handler injects before phase 9. The input limits (SEC-05) are
/// checked before the input is parsed.
///
/// # Errors
///
/// Fails when the hook input exceeds 1 MiB, nests deeper than 64
/// levels, or is no JSON object.
pub fn handle(_hook: Hook, hook_input: &str) -> Result<String, Failure> {
    if hook_input.len() > HOOK_INPUT_LIMIT {
        return Err(Failure::TooLarge);
    }
    if depth(hook_input) > NESTING_LIMIT {
        return Err(Failure::TooDeep);
    }
    match serde_json::from_str::<Value>(hook_input) {
        Ok(Value::Object(_)) => Ok(String::new()),
        _ => Err(Failure::Malformed),
    }
}

/// The deepest `text`'s brackets nest, counting only those outside
/// JSON strings; a stray closing bracket counts nothing.
fn depth(text: &str) -> usize {
    let (mut deepest, mut open) = (0, 0usize);
    let (mut in_string, mut escaped) = (false, false);
    for b in text.bytes() {
        if in_string {
            match (escaped, b) {
                (true, _) => escaped = false,
                (false, b'\\') => escaped = true,
                (false, b'"') => in_string = false,
                _ => {}
            }
            continue;
        }
        match b {
            b'"' => in_string = true,
            b'{' | b'[' => {
                open += 1;
                deepest = deepest.max(open);
            }
            b'}' | b']' => open = open.saturating_sub(1),
            _ => {}
        }
    }

    deepest
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
