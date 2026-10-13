//! What spike S4 recorded of the stub Cairn plugin on each target
//! (ASM-07, ASM-21): the hooks its `hooks/hooks.json` registers, the
//! hook inputs and the init message of each harness session it loaded
//! into, and the log of its stub MCP server, which binds each tool call
//! to its run and mints that run's stand-in run-seat key.

use std::collections::BTreeSet;

use serde_json::Value;

/// The hook events a plugin's `hooks/hooks.json` registers, in name
/// order.
///
/// # Errors
///
/// Fails when the document registers no hook.
pub fn registered(hooks: &Value) -> Result<BTreeSet<String>, String> {
    let events: BTreeSet<String> = hooks
        .get("hooks")
        .and_then(Value::as_object)
        .map(|map| map.keys().cloned().collect())
        .unwrap_or_default();
    if events.is_empty() {
        return Err("the plugin registers no hooks".into());
    }
    Ok(events)
}

/// The MCP servers a plugin's `.mcp.json` bundles, in name order.
///
/// # Errors
///
/// Fails when the document bundles no MCP server.
pub fn servers(mcp: &Value) -> Result<Vec<String>, String> {
    let names: Vec<String> = mcp
        .get("mcpServers")
        .and_then(Value::as_object)
        .map(|map| map.keys().cloned().collect())
        .unwrap_or_default();
    if names.is_empty() {
        return Err("the plugin bundles no MCP server".into());
    }
    Ok(names)
}

/// The hook events the recorded `inputs` fired, each once.
#[must_use]
pub fn fired(inputs: &[Value]) -> BTreeSet<String> {
    inputs
        .iter()
        .map(|i| crate::text(i, "hook_event_name"))
        .filter(|e| !e.is_empty())
        .map(String::from)
        .collect()
}

/// One problem per registered event that did not fire.
#[must_use]
pub fn unfired(registered: &BTreeSet<String>, fired: &BTreeSet<String>) -> Vec<String> {
    registered
        .difference(fired)
        .map(|e| format!("{e} registered but did not fire"))
        .collect()
}

/// The tools an init message lists for `server` of `plugin`.
#[must_use]
pub fn tools(init: &Value, plugin: &str, server: &str) -> Vec<String> {
    let prefix = format!("mcp__plugin_{plugin}_{server}__");
    init.get("tools")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|t| t.starts_with(&prefix))
        .map(String::from)
        .collect()
}

/// Whether an init message shows `server` of `plugin` connected.
#[must_use]
pub fn connected(init: &Value, plugin: &str, server: &str) -> bool {
    let name = format!("plugin:{plugin}:{server}");
    init.get("mcp_servers")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|s| crate::text(s, "name") == name && crate::text(s, "status") == "connected")
}

/// A tool call and the run the stub MCP server found for it.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct CallRun {
    pub tool_use_id: String,
    pub run: String,
    /// The transcripts that held the call's tool-use id when bound.
    pub transcripts: Vec<String>,
}

/// What a stub MCP server's log holds.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ServerLog {
    /// The processes that served, one per start.
    pub pids: BTreeSet<u64>,
    pub binds: Vec<CallRun>,
    /// Each run's stand-in run-seat key, as `(run, key)`, in minting
    /// order.
    pub keys: Vec<(String, String)>,
}

/// Reads a stub MCP server's log lines.
#[must_use]
pub fn server_log(lines: &[Value]) -> ServerLog {
    let mut log = ServerLog::default();
    for line in lines {
        match crate::text(line, "kind") {
            "start" => log.pids.extend(line.get("pid").and_then(Value::as_u64)),
            "bind" => log.binds.push(CallRun {
                tool_use_id: crate::text(line, "tool_use_id").into(),
                run: crate::text(line, "run").into(),
                transcripts: line
                    .get("transcripts")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect(),
            }),
            "key" => log.keys.push((
                crate::text(line, "run").into(),
                crate::text(line, "key").into(),
            )),
            _ => {}
        }
    }
    log
}

/// One problem per key whose run was not bound through exactly one
/// transcript, that run's own.
#[must_use]
pub fn unbound_keys(log: &ServerLog) -> Vec<String> {
    log.keys
        .iter()
        .filter(|(run, _)| {
            !log.binds.iter().any(|b| {
                &b.run == run
                    && matches!(b.transcripts.as_slice(), [only] if run_of_transcript(only) == run)
            })
        })
        .map(|(run, _)| {
            format!(
                "the run-seat key of run {run} was bound through no single transcript of that run"
            )
        })
        .collect()
}

/// The run a transcript records, named by its file stem: the harness
/// session id for the main agent's run, `agent-<id>` for a subagent's.
#[must_use]
pub fn run_of_transcript(path: &str) -> &str {
    let name = path.rsplit('/').next().unwrap_or(path);
    name.strip_suffix(".jsonl").unwrap_or(name)
}

/// One problem per needle per place whose text shows it, as
/// `<needle> shows in <place>`.
#[must_use]
pub fn carriers(places: &[(String, String)], needles: &[String]) -> Vec<String> {
    needles
        .iter()
        .flat_map(|needle| {
            places
                .iter()
                .filter(|(_, body)| body.contains(needle.as_str()))
                .map(move |(place, _)| format!("{needle} shows in {place}"))
        })
        .collect()
}

#[cfg(test)]
mod tests;
