use std::error::Error as _;

use audit::entry::{Counter, Kind, Occurrence};

use super::*;

#[test]
fn the_installed_hooks_parse_and_name_their_operation() {
    for (name, hook, operation) in [
        ("SessionStart", Hook::SessionStart, "hook SessionStart"),
        (
            "UserPromptSubmit",
            Hook::UserPromptSubmit,
            "hook UserPromptSubmit",
        ),
        (
            "PermissionRequest",
            Hook::PermissionRequest,
            "hook PermissionRequest",
        ),
    ] {
        assert_eq!(Hook::parse(name), Some(hook));
        assert_eq!(hook.operation(), operation);
    }
    for name in ["Stop", "sessionstart", ""] {
        assert_eq!(Hook::parse(name), None, "{name}");
    }
}

#[test]
fn a_hook_input_that_is_a_json_object_injects_nothing() {
    let input = r#"{"session_id":"s","hook_event_name":"SessionStart","source":"startup"}"#;

    assert_eq!(handle(Hook::SessionStart, input), Ok(String::new()));
}

#[test]
fn a_hook_input_that_is_no_json_object_is_rejected() {
    for input in ["", "{\"session_id\": ", "[1]", "\"text\"", "{} {}"] {
        assert_eq!(
            handle(Hook::UserPromptSubmit, input),
            Err(Failure::Malformed),
            "{input:?}"
        );
    }
}

#[test]
fn a_panic_in_the_handler_is_caught_as_its_own_failure() {
    let caught = guarded(|| -> Result<String, Failure> { panic!("boom") });

    assert_eq!(caught, Err(Failure::Panicked));
    assert_eq!(guarded(|| Ok("out".to_owned())), Ok("out".to_owned()));
    assert_eq!(guarded(|| Err(Failure::Malformed)), Err(Failure::Malformed));
}

#[test]
fn each_failure_is_audited_with_its_own_kind_and_counter() {
    let cases = [
        (
            Failure::UnknownHook,
            Kind::Rejected,
            Counter::HookRejected,
            "no such hook",
        ),
        (
            Failure::Malformed,
            Kind::Rejected,
            Counter::HookRejected,
            "not a JSON object",
        ),
        (
            Failure::Panicked,
            Kind::Failed,
            Counter::HookPanicked,
            "panic recovered",
        ),
        (
            Failure::TooLarge,
            Kind::Rejected,
            Counter::HookInputTooLarge,
            "hook input exceeds 1 MiB",
        ),
        (
            Failure::TooDeep,
            Kind::Rejected,
            Counter::HookInputTooDeep,
            "hook input nesting exceeds depth 64",
        ),
    ];
    for (failure, kind, counter, detail) in cases {
        assert_eq!(
            failure.occurrence("hook SessionStart"),
            Occurrence::Degraded {
                kind,
                counter,
                operation: "hook SessionStart",
                detail,
            }
        );
        assert_eq!(failure.to_string(), detail);
        assert!(failure.source().is_none());
    }
}

/// A JSON object nested `depth` levels deep.
fn nested(depth: usize) -> String {
    format!("{}{}", "{\"a\":".repeat(depth - 1) + "{", "}".repeat(depth))
}

#[test]
fn a_hook_input_past_1_mib_is_refused_before_it_is_parsed() {
    let fits = format!("{{\"p\":\"{}\"}}", "x".repeat(HOOK_INPUT_LIMIT - 8));
    assert_eq!(fits.len(), HOOK_INPUT_LIMIT);
    assert_eq!(handle(Hook::UserPromptSubmit, &fits), Ok(String::new()));
    let over = format!("{fits} ");
    assert_eq!(handle(Hook::UserPromptSubmit, &over), Err(Failure::TooLarge));
}

#[test]
fn a_hook_input_nested_past_depth_64_is_refused_before_it_is_parsed() {
    assert_eq!(handle(Hook::UserPromptSubmit, &nested(64)), Ok(String::new()));
    assert_eq!(handle(Hook::UserPromptSubmit, &nested(65)), Err(Failure::TooDeep));
    // Deep enough that the parser's own limit would refuse it as malformed.
    assert_eq!(handle(Hook::UserPromptSubmit, &nested(200)), Err(Failure::TooDeep));
}

#[test]
fn depth_counts_brackets_outside_strings_only() {
    assert_eq!(depth(""), 0);
    assert_eq!(depth("{}"), 1);
    assert_eq!(depth(r#"{"a":[1,{"b":[]}]}"#), 4);
    assert_eq!(depth(r#"{"a":"[[[[{{"}"#), 1);
    assert_eq!(depth(r#"{"a":"\"[[["}"#), 1);
    assert_eq!(depth(r#"{"a":"\\"}"#), 1);
    assert_eq!(depth("}}}{"), 1);
}
