//! Checks on recorded hook inputs (§9.1): which fields each carries.

use serde_json::Value;

use crate::text;

/// The fields a hook input carries only when a subagent's run fired it.
pub const SUBAGENT_FIELDS: [&str; 3] = ["agent_id", "agent_type", "agent_transcript_path"];

/// One field a hook input must carry, and the value it must hold when
/// one is named.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expect {
    pub field: String,
    pub value: Option<String>,
}

/// Parses `"a" set to "b" and "c"`: each `and`-joined part names a
/// field, optionally with the value it must hold.
///
/// # Errors
///
/// Fails on a part that is not a quoted field name.
pub fn parse(text: &str) -> Result<Vec<Expect>, String> {
    text.split(" and ").map(parse_part).collect()
}

fn parse_part(part: &str) -> Result<Expect, String> {
    let (field, value) = match part.trim().split_once(" set to ") {
        Some((field, value)) => (field, Some(unquote(value)?)),
        None => (part.trim(), None),
    };
    Ok(Expect {
        field: unquote(field)?,
        value,
    })
}

fn unquote(s: &str) -> Result<String, String> {
    s.trim()
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .map(str::to_owned)
        .ok_or_else(|| format!("expected a quoted name, got {s:?}"))
}

/// The quoted names in `list`, such as `"a", "b" and "c"`.
///
/// # Errors
///
/// Fails when `list` quotes no name.
pub fn names(list: &str) -> Result<Vec<String>, String> {
    let names: Vec<String> = list
        .split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_owned)
        .collect();
    if names.is_empty() {
        return Err(format!("no quoted name in {list:?}"));
    }
    Ok(names)
}

/// Whether some input to `first` comes before some input to `then`.
#[must_use]
pub fn ordered(inputs: &[Value], first: &str, then: &str) -> bool {
    let at = |event: &str| {
        inputs
            .iter()
            .position(|i| text(i, "hook_event_name") == event)
    };
    let last_then = inputs
        .iter()
        .rposition(|i| text(i, "hook_event_name") == then);
    matches!((at(first), last_then), (Some(f), Some(t)) if f < t)
}

/// The `fields` in which each of `others` differs from `main`.
#[must_use]
pub fn identity_mismatches(main: &Value, others: &[&Value], fields: &[&str]) -> Vec<String> {
    others
        .iter()
        .flat_map(|i| {
            fields
                .iter()
                .filter(|f| text(i, f) != text(main, f))
                .map(|f| {
                    format!(
                        "{} carries {f} {}, not {}",
                        describe(i),
                        text(i, f),
                        text(main, f)
                    )
                })
        })
        .collect()
}

/// The inputs among `inputs` that the hook `event` received.
#[must_use]
pub fn of_event<'a>(inputs: &'a [Value], event: &str) -> Vec<&'a Value> {
    inputs
        .iter()
        .filter(|i| text(i, "hook_event_name") == event)
        .collect()
}

/// The names in `fields` that `input` does not carry.
#[must_use]
pub fn missing(input: &Value, fields: &[&str]) -> Vec<String> {
    fields
        .iter()
        .filter(|f| input.get(**f).is_none_or(Value::is_null))
        .map(|f| format!("{} carries no {f:?}", describe(input)))
        .collect()
}

/// Each of `expects` that `input` does not meet.
#[must_use]
pub fn unmet(input: &Value, expects: &[Expect]) -> Vec<String> {
    expects
        .iter()
        .filter_map(
            |e| match (input.get(&e.field).filter(|v| !v.is_null()), &e.value) {
                (None, _) => Some(format!("{} carries no {:?}", describe(input), e.field)),
                (Some(got), Some(want)) if got.as_str() != Some(want.as_str()) => Some(format!(
                    "{} carries {:?} = {got}, want {want:?}",
                    describe(input),
                    e.field
                )),
                _ => None,
            },
        )
        .collect()
}

/// The subagent fields `input` carries.
#[must_use]
pub fn subagent_fields(input: &Value) -> Vec<String> {
    SUBAGENT_FIELDS
        .iter()
        .filter(|f| input.get(**f).is_some())
        .map(|f| format!("{} carries {f:?}", describe(input)))
        .collect()
}

fn describe(input: &Value) -> String {
    format!("the {} hook input", text(input, "hook_event_name"))
}

#[cfg(test)]
mod tests;
