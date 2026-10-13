//! ENG-14: no agent instruction file directs a destructive command at
//! non-isolated state. The files are the agent instructions the
//! repository carries: every `CLAUDE.md` and `AGENTS.md`, and
//! everything under a `.claude/` directory (agents, skills, settings).
//! A statement in them directs a destructive command when it names a
//! way to delete or overwrite, names the real home (`~`, `HOME`,
//! `CAIRN_HOME`, Claude Code's configuration directory), and neither
//! forbids the act nor first points `HOME` at a temporary directory.

use std::path::Path;

use crate::Checkout;

/// Lists the instruction files under `root`, as paths relative to it,
/// sorted. Another checkout under `.claude/worktrees` is no part of
/// this one, nor is a file a developer keeps for themselves
/// (`settings.local.json`).
///
/// # Errors
///
/// Fails when a directory cannot be listed.
pub fn instruction_files(_root: &Path) -> Result<Vec<String>, String> {
    Ok(Vec::new())
}

/// The statements of `body`, the text of the file `file`, that direct
/// a destructive command at non-isolated state, one line each:
/// `file:line: ...`.
#[must_use]
pub fn directives(_file: &str, _body: &str) -> Vec<String> {
    Vec::new()
}

/// Requires every instruction file of the checkout to be free of a
/// destructive command aimed at non-isolated state.
#[must_use]
pub fn no_destructive_instructions(_c: &Checkout) -> Vec<String> {
    Vec::new()
}

#[cfg(test)]
mod tests;
