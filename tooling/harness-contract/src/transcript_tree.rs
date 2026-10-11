//! The harness's tree of transcripts (ASM-03, ASM-16):
//! `projects/<dir>/<session_id>.jsonl` for each harness session, and
//! `projects/<dir>/<session_id>/subagents/agent-<agent_id>.jsonl` with
//! a `.meta.json` beside it for each subagent.

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::recording::{entries, files, json, lines};
use crate::text;

/// One harness session's transcript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessSession {
    pub dir: String,
    pub id: String,
    pub transcript: PathBuf,
}

/// One subagent's transcript, under its harness session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subagent {
    pub harness_session: String,
    pub agent: String,
    pub transcript: PathBuf,
}

/// What a walk of a tree found: its sessions, its subagents, and the
/// transcripts that sit where neither belongs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tree {
    pub harness_sessions: Vec<HarnessSession>,
    pub subagents: Vec<Subagent>,
    pub strays: Vec<PathBuf>,
}

/// Walks the tree at `tree_dir`.
///
/// # Errors
///
/// Fails when a directory cannot be listed.
pub fn walk(tree_dir: &Path) -> Result<Tree, String> {
    let mut tree = Tree::default();
    for dir in entries(tree_dir)?.into_iter().filter(|p| p.is_dir()) {
        let name = file_name(&dir);
        for entry in entries(&dir)? {
            if entry.is_dir() {
                walk_harness_session(&entry, &mut tree)?;
            } else if let Some(id) = file_name(&entry).strip_suffix(".jsonl") {
                tree.harness_sessions.push(HarnessSession {
                    dir: name.clone(),
                    id: id.to_owned(),
                    transcript: entry.clone(),
                });
            }
        }
    }
    Ok(tree)
}

fn walk_harness_session(dir: &Path, tree: &mut Tree) -> Result<(), String> {
    let subagents = dir.join("subagents");
    for path in files(dir, ".jsonl")? {
        let agent = file_name(&path)
            .strip_prefix("agent-")
            .and_then(|n| n.strip_suffix(".jsonl"))
            .map(str::to_owned);
        match agent {
            Some(agent) if path.parent() == Some(subagents.as_path()) => {
                tree.subagents.push(Subagent {
                    harness_session: file_name(dir),
                    agent,
                    transcript: path,
                });
            }
            _ => tree.strays.push(path),
        }
    }
    Ok(())
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Checks that each line of `session`'s transcript names its harness
/// session.
///
/// # Errors
///
/// Fails when the transcript cannot be read.
pub fn harness_session_problems(session: &HarnessSession) -> Result<Vec<String>, String> {
    let lines = lines(&session.transcript)?;
    Ok(foreign(
        &lines,
        "sessionId",
        &session.id,
        &session.transcript,
    ))
}

/// Checks that each line of `subagent`'s transcript names its harness
/// session and its agent, and that the harness session has a
/// transcript of its own in `tree`.
///
/// # Errors
///
/// Fails when the transcript cannot be read.
pub fn subagent_problems(subagent: &Subagent, tree: &Tree) -> Result<Vec<String>, String> {
    let path = &subagent.transcript;
    let lines = lines(path)?;
    let mut problems = foreign(&lines, "sessionId", &subagent.harness_session, path);
    problems.extend(foreign(&lines, "agentId", &subagent.agent, path));
    if !tree
        .harness_sessions
        .iter()
        .any(|s| s.id == subagent.harness_session)
    {
        problems.push(format!(
            "{}: harness session {} has no transcript",
            path.display(),
            subagent.harness_session
        ));
    }
    Ok(problems)
}

fn foreign(lines: &[Value], field: &str, want: &str, path: &Path) -> Vec<String> {
    let named: Vec<&str> = lines
        .iter()
        .map(|l| text(l, field))
        .filter(|got| !got.is_empty())
        .collect();
    if named.is_empty() {
        return vec![format!("{}: no line names its {field}", path.display())];
    }
    named
        .into_iter()
        .filter(|got| *got != want)
        .map(|got| format!("{}: a line names {field} {got}, not {want}", path.display()))
        .collect()
}

/// The harness sessions whose id names more than one transcript in
/// `tree`.
#[must_use]
pub fn duplicates(tree: &Tree) -> Vec<String> {
    let mut seen = std::collections::BTreeMap::<&str, usize>::new();
    for s in &tree.harness_sessions {
        *seen.entry(s.id.as_str()).or_default() += 1;
    }
    seen.into_iter()
        .filter(|(_, n)| *n > 1)
        .map(|(id, n)| format!("harness session {id} has {n} transcripts"))
        .collect()
}

/// Checks that the harness sessions `inputs` record in each of `cwds`
/// are distinct, and that each wrote its transcript to `dir` in `tree`.
#[must_use]
pub fn shared_directory(inputs: &[Value], cwds: &[String], dir: &str, tree: &Tree) -> Vec<String> {
    let mut problems = Vec::new();
    let mut ids = Vec::new();
    for cwd in cwds {
        let Some(input) = inputs.iter().find(|i| text(i, "cwd") == cwd) else {
            problems.push(format!("no harness session recorded in {cwd}"));
            continue;
        };
        let id = text(input, "session_id");
        ids.push(id);
        let got = transcript_dir(input);
        if got != dir {
            problems.push(format!("{cwd} maps to {got}, not {dir}"));
        }
        if !tree
            .harness_sessions
            .iter()
            .any(|s| s.id == id && s.dir == dir)
        {
            problems.push(format!("the tree holds no transcript of {id} under {dir}"));
        }
    }
    ids.sort_unstable();
    ids.dedup();
    if ids.len() != cwds.len() {
        problems.push("the harness sessions are not distinct".into());
    }
    problems
}

/// Checks that a `.meta.json` beside `subagent`'s transcript names its
/// agent type.
#[must_use]
pub fn meta_problems(subagent: &Subagent) -> Vec<String> {
    let meta = subagent.transcript.with_extension("meta.json");
    match json(&meta) {
        Ok(v) if !text(&v, "agentType").is_empty() => Vec::new(),
        Ok(_) => vec![format!("{} names no agentType", meta.display())],
        Err(e) => vec![e],
    }
}

/// The transcript directory a hook input's `transcript_path` lies in.
#[must_use]
pub fn transcript_dir(input: &Value) -> String {
    Path::new(text(input, "transcript_path"))
        .parent()
        .map(file_name)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
