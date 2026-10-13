//! Checks on recorded transcripts: one JSON value per line, as the
//! harness wrote it (ASM-02, ASM-04, ASM-06, ASM-11 to ASM-15).

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::{at, text};

/// The marker of a slash command's output in a `user` line.
pub const COMMAND_OUTPUT: &str = "<local-command-stdout>";

/// The hooks whose additional context the harness recorded as a hook
/// context attachment.
#[must_use]
pub fn hook_context_events(lines: &[Value]) -> BTreeSet<String> {
    lines
        .iter()
        .filter(|l| text(l, "attachment.type") == "hook_additional_context")
        .map(|l| text(l, "attachment.hookEvent").to_owned())
        .collect()
}

/// The hooks whose output a slash command's output repeats: each of
/// its lines that starts `<Hook> [<command>]`.
#[must_use]
pub fn echoed_hooks(lines: &[Value]) -> BTreeSet<String> {
    lines
        .iter()
        .filter(|l| text(l, "type") == "user")
        .map(|l| text(l, "message.content"))
        .filter(|c| c.contains(COMMAND_OUTPUT))
        .flat_map(str::lines)
        .filter_map(|line| line.split_once(" [").map(|(hook, _)| hook))
        .filter(|hook| !hook.is_empty() && hook.chars().all(|c| c.is_ascii_alphabetic()))
        .map(str::to_owned)
        .collect()
}

/// The assistant lines that carry other than one content block.
#[must_use]
pub fn multi_block(lines: &[Value]) -> Vec<String> {
    assistant(lines)
        .filter(|l| {
            at(l, "message.content")
                .and_then(Value::as_array)
                .map(Vec::len)
                != Some(1)
        })
        .map(|l| {
            format!(
                "assistant line {} carries other than one content block",
                text(l, "uuid")
            )
        })
        .collect()
}

/// How many assistant lines each API message spans, by `message.id`.
#[must_use]
pub fn lines_per_message(lines: &[Value]) -> BTreeMap<String, usize> {
    let mut count = BTreeMap::new();
    for l in assistant(lines).filter(|l| !text(l, "message.id").is_empty()) {
        *count.entry(text(l, "message.id").to_owned()).or_insert(0) += 1;
    }
    count
}

/// Whether one API message spans at least `n` lines that each carry a
/// tool call.
#[must_use]
pub fn spans_tool_calls(lines: &[Value], n: usize) -> bool {
    let calls: Vec<Value> = assistant(lines)
        .filter(|l| blocks(l).any(|b| text(b, "type") == "tool_use"))
        .cloned()
        .collect();
    lines_per_message(&calls).values().any(|c| *c >= n)
}

/// Whether `lines` hold a compaction.
#[must_use]
pub fn has_compaction(lines: &[Value]) -> bool {
    compaction(lines).is_some()
}

/// Whether `hook`'s additional context follows the first compaction in
/// `lines`.
#[must_use]
pub fn context_after_compaction(lines: &[Value], hook: &str) -> bool {
    compaction(lines).is_some_and(|b| hook_context_events(&lines[b..]).contains(hook))
}

/// How many of `hook`'s context attachments holding `needle` each
/// stretch between compactions records, the first stretch first.
#[must_use]
pub fn context_per_compaction(lines: &[Value], hook: &str, needle: &str) -> Vec<usize> {
    let mut counts = vec![0];
    for l in lines {
        if text(l, "subtype") == "compact_boundary" {
            counts.push(0);
        } else if text(l, "attachment.type") == "hook_additional_context"
            && text(l, "attachment.hookEvent") == hook
            && l.to_string().contains(needle)
            && let Some(last) = counts.last_mut()
        {
            *last += 1;
        }
    }
    counts
}

/// Where a line holding some text was recorded.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Shown {
    /// A slash command's output, which the model reads.
    CommandOutput,
    /// The record of a hook's own run, which the model does not read.
    HookAttachment,
    /// Any other line, by its type and subtype.
    Other(String),
}

/// Where each line holding `needle` was recorded.
#[must_use]
pub fn where_shown(lines: &[Value], needle: &str) -> BTreeSet<Shown> {
    lines
        .iter()
        .filter(|l| l.to_string().contains(needle))
        .map(shown)
        .collect()
}

fn shown(line: &Value) -> Shown {
    let kind = text(line, "type");
    let sub = match text(line, "attachment.type") {
        "" => text(line, "subtype"),
        a => a,
    };
    match (kind, sub) {
        ("attachment", "hook_success" | "hook_non_blocking_error")
        | ("system", "stop_hook_summary") => Shown::HookAttachment,
        ("user", _) if text(line, "message.content").contains(COMMAND_OUTPUT) => {
            Shown::CommandOutput
        }
        ("system", "local_command") => Shown::CommandOutput,
        (kind, "") => Shown::Other(kind.to_owned()),
        (kind, sub) => Shown::Other(format!("{kind}/{sub}")),
    }
}

/// The text of each model reply after the first line other than an
/// assistant line that holds `needle`: the assistant lines that follow
/// it, the harness's refusals left out.
#[must_use]
pub fn model_replies_after<'a>(lines: &'a [Value], needle: &str) -> Vec<&'a str> {
    let Some(first) = lines
        .iter()
        .position(|l| text(l, "type") != "assistant" && l.to_string().contains(needle))
    else {
        return Vec::new();
    };
    model_replies(&lines[first..]).collect()
}

/// The text of each refusal the harness recorded in place of a model
/// reply:
/// an assistant line marked `isApiErrorMessage`.
#[must_use]
pub fn refusals(lines: &[Value]) -> Vec<&str> {
    assistant(lines)
        .filter(|l| is_refusal(l))
        .map(reply_text)
        .collect()
}

/// Whether a model reply followed the harness's first refusal.
#[must_use]
pub fn model_replied_after_refusal(lines: &[Value]) -> bool {
    lines
        .iter()
        .position(|l| text(l, "type") == "assistant" && is_refusal(l))
        .is_some_and(|first| model_replies(&lines[first..]).next().is_some())
}

fn model_replies(lines: &[Value]) -> impl Iterator<Item = &str> {
    assistant(lines).filter(|l| !is_refusal(l)).map(reply_text)
}

fn is_refusal(line: &Value) -> bool {
    line.get("isApiErrorMessage") == Some(&Value::Bool(true))
}

fn reply_text(line: &Value) -> &str {
    blocks(line)
        .map(|b| text(b, "text"))
        .next()
        .unwrap_or_default()
}

fn compaction(lines: &[Value]) -> Option<usize> {
    lines
        .iter()
        .position(|l| text(l, "subtype") == "compact_boundary")
}

/// The API messages whose lines do not all repeat one `usage`.
#[must_use]
pub fn usage_spread(lines: &[Value]) -> Vec<String> {
    let mut usage: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for l in assistant(lines) {
        let u = at(l, "message.usage")
            .map(Value::to_string)
            .unwrap_or_default();
        usage.entry(text(l, "message.id")).or_default().insert(u);
    }
    usage
        .into_iter()
        .filter(|(_, u)| u.len() > 1)
        .map(|(id, _)| format!("message {id} repeats differing usage"))
        .collect()
}

fn assistant(lines: &[Value]) -> impl Iterator<Item = &Value> {
    lines.iter().filter(|l| text(l, "type") == "assistant")
}

/// What [`tool_results`] found: how many results it checked, the
/// problems with their output, and those with the line they name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToolResults {
    pub checked: usize,
    pub output: Vec<String>,
    pub source: Vec<String>,
}

/// Checks each tool result of a call to `tool`: it carries the output in
/// `message.content` and in `toolUseResult`, and its
/// `sourceToolAssistantUUID` names the line carrying the call.
#[must_use]
pub fn tool_results(lines: &[Value], tool: &str) -> ToolResults {
    let calls: BTreeMap<&str, &str> = lines
        .iter()
        .flat_map(|l| blocks(l).map(move |b| (b, text(l, "uuid"))))
        .filter(|(b, _)| text(b, "type") == "tool_use" && text(b, "name") == tool)
        .map(|(b, uuid)| (text(b, "id"), uuid))
        .collect();
    let mut found = ToolResults::default();
    for l in lines {
        for b in blocks(l).filter(|b| text(b, "type") == "tool_result") {
            let Some(call) = calls.get(text(b, "tool_use_id")) else {
                continue;
            };
            found.checked += 1;
            found.output.extend(output_problems(l, b));
            if text(l, "sourceToolAssistantUUID") != *call {
                found.source.push(format!(
                    "the result of {} does not name its call's line {call}",
                    text(b, "tool_use_id")
                ));
            }
        }
    }
    found
}

fn output_problems(line: &Value, block: &Value) -> Vec<String> {
    let id = text(block, "tool_use_id");
    let mut problems = Vec::new();
    if block.get("content").is_none() {
        problems.push(format!(
            "the result of {id} has no output in message.content"
        ));
    }
    if line.get("toolUseResult").is_none() {
        problems.push(format!("the result of {id} carries no toolUseResult"));
    }
    problems
}

fn blocks(line: &Value) -> impl Iterator<Item = &Value> {
    at(line, "message.content")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}

/// How many lines are of type `kind`, and how many of those carry a
/// `uuid` and a `timestamp`.
#[must_use]
pub fn keys_by_type(lines: &[Value], kind: &str) -> (usize, usize, usize) {
    lines
        .iter()
        .filter(|l| text(l, "type") == kind)
        .fold((0, 0, 0), |(n, u, t), l| {
            (
                n + 1,
                u + usize::from(l.get("uuid").is_some()),
                t + usize::from(l.get("timestamp").is_some()),
            )
        })
}

/// How many thinking blocks there are, and those that keep their text
/// or lose their signature.
#[must_use]
pub fn thinking(lines: &[Value]) -> (usize, Vec<String>) {
    let all: Vec<&Value> = lines
        .iter()
        .flat_map(blocks)
        .filter(|b| text(b, "type") == "thinking")
        .collect();
    let problems = all
        .iter()
        .filter(|b| !text(b, "thinking").is_empty() || text(b, "signature").is_empty())
        .map(|b| {
            format!(
                "a thinking block keeps text {:?} or no signature",
                text(b, "thinking")
            )
        })
        .collect();
    (all.len(), problems)
}

/// The first attachment line whose content holds `marker`.
#[must_use]
pub fn attachment_with<'a>(lines: &'a [Value], marker: &str) -> Option<&'a Value> {
    lines
        .iter()
        .filter(|l| text(l, "type") == "attachment")
        .find(|l| at(l, "attachment").is_some_and(|a| a.to_string().contains(marker)))
}

/// Checks that the first line `after` appended with a parent continues
/// from a line of `before` other than its last: the transcript branched.
#[must_use]
pub fn branched(before: &[Value], after: &[Value]) -> Vec<String> {
    let earlier: Vec<&str> = before
        .iter()
        .map(|l| text(l, "uuid"))
        .filter(|u| !u.is_empty())
        .collect();
    let Some(parent) = after
        .iter()
        .skip(before.len())
        .map(|l| text(l, "parentUuid"))
        .find(|p| !p.is_empty())
    else {
        return vec!["no line with a parent was appended".into()];
    };
    match earlier.iter().position(|u| *u == parent) {
        None => vec![format!(
            "the first appended line continues from {parent}, not an earlier line"
        )],
        Some(i) if i + 1 == earlier.len() => {
            vec![format!(
                "the first appended line continues from the last line {parent}"
            )]
        }
        Some(_) => Vec::new(),
    }
}

/// The types of the lines `after` appended to `before`.
#[must_use]
pub fn appended_types(before: &[Value], after: &[Value]) -> BTreeSet<String> {
    after
        .iter()
        .skip(before.len())
        .map(|l| {
            let sub = text(l, "subtype");
            if sub.is_empty() { text(l, "type") } else { sub }.to_owned()
        })
        .collect()
}

#[cfg(test)]
mod tests;
