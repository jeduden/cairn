use super::*;
use serde_json::json;

fn expect(field: &str, value: Option<&str>) -> Expect {
    Expect {
        field: field.into(),
        value: value.map(Into::into),
    }
}

#[test]
fn parse_reads_fields_with_and_without_values() {
    assert_eq!(
        parse(r#""trigger" set to "manual" and "custom_instructions""#).unwrap(),
        [
            expect("trigger", Some("manual")),
            expect("custom_instructions", None)
        ]
    );
    assert_eq!(parse(r#""prompt""#).unwrap(), [expect("prompt", None)]);
}

#[test]
fn parse_refuses_an_unquoted_name_or_value() {
    assert!(parse("prompt").unwrap_err().contains("quoted"));
    assert!(parse(r#""source" set to resume"#).is_err());
}

#[test]
fn of_event_picks_the_named_hook() {
    let inputs = [
        json!({"hook_event_name": "SessionStart"}),
        json!({"hook_event_name": "Stop"}),
    ];

    assert_eq!(of_event(&inputs, "Stop"), [&inputs[1]]);
    assert!(of_event(&inputs, "PreCompact").is_empty());
}

#[test]
fn missing_names_each_absent_field() {
    let input = json!({"hook_event_name": "Stop", "cwd": "/w"});

    assert_eq!(
        missing(&input, &["cwd", "session_id"]),
        [r#"the Stop hook input carries no "session_id""#]
    );
}

#[test]
fn unmet_reports_absent_fields_and_wrong_values() {
    let input = json!({"hook_event_name": "SessionStart", "source": "resume"});

    assert!(unmet(&input, &[expect("source", Some("resume"))]).is_empty());
    assert_eq!(
        unmet(
            &input,
            &[expect("source", Some("fork")), expect("prompt", None)]
        ),
        [
            r#"the SessionStart hook input carries "source" = "resume", want "fork""#,
            r#"the SessionStart hook input carries no "prompt""#
        ]
    );
}

#[test]
fn subagent_fields_names_each_one_present() {
    let parent = json!({"hook_event_name": "PreCompact"});
    let child = json!({"hook_event_name": "PostToolUse", "agent_id": "a1", "agent_type": "x"});

    assert!(subagent_fields(&parent).is_empty());
    assert_eq!(
        subagent_fields(&child),
        [
            r#"the PostToolUse hook input carries "agent_id""#,
            r#"the PostToolUse hook input carries "agent_type""#
        ]
    );
}

#[test]
fn names_reads_a_quoted_list_and_refuses_none() {
    assert_eq!(names(r#""a", "b" and "c""#).unwrap(), ["a", "b", "c"]);
    assert!(names("a, b").unwrap_err().contains("no quoted name"));
}

#[test]
fn ordered_needs_first_before_then() {
    let inputs = [
        json!({"hook_event_name": "SessionStart"}),
        json!({"hook_event_name": "SubagentStart"}),
        json!({"hook_event_name": "PreCompact"}),
    ];

    assert!(ordered(&inputs, "SubagentStart", "PreCompact"));
    assert!(!ordered(&inputs, "PreCompact", "SubagentStart"));
    assert!(!ordered(&inputs, "SubagentStop", "PreCompact"));
}

#[test]
fn identity_mismatches_name_each_differing_field() {
    let main =
        json!({"hook_event_name": "SessionStart", "session_id": "s", "transcript_path": "t"});
    let same = json!({"hook_event_name": "PreCompact", "session_id": "s", "transcript_path": "t"});
    let other = json!({"hook_event_name": "PreCompact", "session_id": "x", "transcript_path": "t"});

    assert!(identity_mismatches(&main, &[&same], &["session_id", "transcript_path"]).is_empty());
    assert_eq!(
        identity_mismatches(&main, &[&other], &["session_id", "transcript_path"]),
        ["the PreCompact hook input carries session_id x, not s"]
    );
}

#[test]
fn a_null_field_counts_as_missing() {
    let input = json!({"hook_event_name": "PreCompact", "custom_instructions": null});

    assert_eq!(missing(&input, &["custom_instructions"]).len(), 1);
    assert_eq!(
        unmet(&input, &[expect("custom_instructions", None)]).len(),
        1
    );
}
