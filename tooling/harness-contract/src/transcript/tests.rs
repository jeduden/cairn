use super::*;
use serde_json::json;

fn context(hook: &str) -> Value {
    json!({"type": "attachment", "attachment": {"type": "hook_additional_context", "hookEvent": hook}})
}

fn reply(id: &str, uuid: &str, block: Value, usage: u64) -> Value {
    json!({"type": "assistant", "uuid": uuid,
           "message": {"id": id, "content": [block], "usage": {"output_tokens": usage}}})
}

#[test]
fn hook_context_events_name_each_hook_once() {
    let lines = [
        context("SessionStart"),
        context("SessionStart"),
        context("PostToolUse"),
        json!({"type": "attachment", "attachment": {"type": "skill_listing"}}),
    ];

    assert_eq!(
        hook_context_events(&lines),
        BTreeSet::from(["PostToolUse".to_owned(), "SessionStart".to_owned()])
    );
}

#[test]
fn echoed_hooks_read_command_output_lines() {
    let out = "<local-command-stdout>Compacted\nPreCompact [x.sh] failed: no\nnot a hook [y]\n [z]\nPostCompact [x.sh] completed</local-command-stdout>";
    let lines = [
        json!({"type": "user", "message": {"content": out}}),
        json!({"type": "user", "message": {"content": "SessionStart [x] typed by a person"}}),
        json!({"type": "assistant", "message": {"content": out}}),
    ];

    assert_eq!(
        echoed_hooks(&lines),
        BTreeSet::from(["PostCompact".to_owned(), "PreCompact".to_owned()])
    );
}

#[test]
fn multi_block_names_lines_with_other_than_one_block() {
    let two = json!({"type": "assistant", "uuid": "u2",
                     "message": {"content": [{"type": "text"}, {"type": "text"}]}});
    let none = json!({"type": "assistant", "uuid": "u3", "message": {}});
    let lines = [reply("m1", "u1", json!({"type": "text"}), 1), two, none];

    assert_eq!(
        multi_block(&lines),
        [
            "assistant line u2 carries other than one content block",
            "assistant line u3 carries other than one content block"
        ]
    );
}

#[test]
fn lines_per_message_and_usage_spread_group_by_message_id() {
    let lines = [
        reply("m1", "u1", json!({"type": "tool_use"}), 5),
        reply("m1", "u2", json!({"type": "tool_use"}), 5),
        reply("m2", "u3", json!({"type": "text"}), 1),
        reply("m2", "u4", json!({"type": "text"}), 2),
        json!({"type": "user"}),
    ];

    assert_eq!(
        lines_per_message(&lines),
        BTreeMap::from([("m1".to_owned(), 2), ("m2".to_owned(), 2)])
    );
    assert_eq!(usage_spread(&lines), ["message m2 repeats differing usage"]);
}

#[test]
fn tool_results_check_output_and_source() {
    let call = json!({"type": "assistant", "uuid": "call",
        "message": {"content": [{"type": "tool_use", "id": "t1", "name": "Bash"}]}});
    let other = json!({"type": "assistant", "uuid": "o",
        "message": {"content": [{"type": "tool_use", "id": "t2", "name": "Read"}]}});
    let good = json!({"type": "user", "sourceToolAssistantUUID": "call", "toolUseResult": {"stdout": "hi"},
        "message": {"content": [{"type": "tool_result", "tool_use_id": "t1", "content": "hi"}]}});
    let bad = json!({"type": "user", "sourceToolAssistantUUID": "o",
        "message": {"content": [{"type": "tool_result", "tool_use_id": "t1"}]}});
    let unrelated = json!({"type": "user",
        "message": {"content": [{"type": "tool_result", "tool_use_id": "t2"}]}});

    assert_eq!(
        tool_results(&[call.clone(), good], "Bash"),
        ToolResults {
            checked: 1,
            ..ToolResults::default()
        }
    );
    assert_eq!(
        tool_results(&[call, other, bad, unrelated], "Bash"),
        ToolResults {
            checked: 1,
            output: vec![
                "the result of t1 has no output in message.content".to_owned(),
                "the result of t1 carries no toolUseResult".to_owned()
            ],
            source: vec!["the result of t1 does not name its call's line call".to_owned()]
        }
    );
}

#[test]
fn keys_by_type_counts_uuid_and_timestamp() {
    let lines = [
        json!({"type": "queue-operation", "timestamp": "t"}),
        json!({"type": "queue-operation", "timestamp": "t", "uuid": "u"}),
        json!({"type": "last-prompt"}),
    ];

    assert_eq!(keys_by_type(&lines, "queue-operation"), (2, 1, 2));
    assert_eq!(keys_by_type(&lines, "last-prompt"), (1, 0, 0));
    assert_eq!(keys_by_type(&lines, "ai-title"), (0, 0, 0));
}

#[test]
fn thinking_counts_blocks_and_names_ones_that_keep_text() {
    let lines = [
        reply(
            "m",
            "a",
            json!({"type": "thinking", "thinking": "", "signature": "s"}),
            1,
        ),
        reply(
            "m",
            "b",
            json!({"type": "thinking", "thinking": "kept", "signature": "s"}),
            1,
        ),
        reply("m", "c", json!({"type": "thinking", "thinking": ""}), 1),
        reply("m", "d", json!({"type": "text", "text": "hi"}), 1),
    ];

    let (n, problems) = thinking(&lines);

    assert_eq!(n, 3);
    assert_eq!(problems.len(), 2);
    assert!(problems[0].contains("\"kept\""));
}

#[test]
fn attachment_with_finds_the_first_attachment_holding_a_marker() {
    let lines = [
        json!({"type": "assistant", "attachment": {"x": "MARK"}}),
        json!({"type": "attachment", "uuid": "1", "attachment": {"type": "date"}}),
        json!({"type": "attachment", "uuid": "2", "attachment": {"type": "instructions", "c": "the MARK here"}}),
    ];

    assert_eq!(attachment_with(&lines, "MARK"), Some(&lines[2]));
    assert_eq!(attachment_with(&lines, "absent"), None);
}

fn linked(uuid: &str, parent: &str) -> Value {
    json!({"type": "user", "uuid": uuid, "parentUuid": parent})
}

#[test]
fn branched_accepts_a_line_continuing_from_before_the_last() {
    let before = [
        linked("a", ""),
        json!({"type": "last-prompt"}),
        linked("b", "a"),
        linked("c", "b"),
    ];
    let mut after = before.to_vec();
    after.extend([json!({"type": "file-history-snapshot"}), linked("d", "b")]);

    assert!(branched(&before, &after).is_empty());
}

#[test]
fn branched_refuses_a_plain_append_or_a_stranger() {
    let before = [linked("a", ""), linked("b", "a")];
    let mut plain = before.to_vec();
    plain.push(linked("c", "b"));
    let mut stranger = before.to_vec();
    stranger.push(linked("c", "zz"));
    let mut parentless = before.to_vec();
    parentless.push(json!({"type": "last-prompt"}));

    assert_eq!(
        branched(&before, &plain),
        ["the first appended line continues from the last line b"]
    );
    assert_eq!(
        branched(&before, &stranger),
        ["the first appended line continues from zz, not an earlier line"]
    );
    assert_eq!(
        branched(&before, &parentless),
        ["no line with a parent was appended"]
    );
}

#[test]
fn appended_types_prefer_the_subtype() {
    let before = [json!({"type": "user"})];
    let after = [
        json!({"type": "user"}),
        json!({"type": "system", "subtype": "compact_boundary"}),
        json!({"type": "attachment"}),
    ];

    assert_eq!(
        appended_types(&before, &after),
        BTreeSet::from(["attachment".to_owned(), "compact_boundary".to_owned()])
    );
}

#[test]
fn lines_per_message_skips_lines_without_an_id() {
    let lines = [
        json!({"type": "assistant", "message": {"content": []}}),
        json!({"type": "assistant", "message": {"content": []}}),
    ];

    assert!(lines_per_message(&lines).is_empty());
}

#[test]
fn spans_tool_calls_needs_one_message_with_enough_calls() {
    let call = |id: &str, uuid: &str| reply(id, uuid, json!({"type": "tool_use"}), 1);
    let lines = [
        call("m1", "a"),
        reply("m1", "b", json!({"type": "text"}), 1),
        call("m2", "c"),
        call("m2", "d"),
    ];

    assert!(spans_tool_calls(&lines, 2));
    assert!(!spans_tool_calls(&lines, 3));
    assert!(!spans_tool_calls(&lines[..2], 2));
}

#[test]
fn context_after_compaction_looks_past_the_first_boundary() {
    let before = [context("SessionStart"), json!({"type": "user"})];
    let mut after = before.to_vec();
    after.extend([
        json!({"type": "system", "subtype": "compact_boundary"}),
        context("SessionStart"),
    ]);

    assert!(!has_compaction(&before));
    assert!(!context_after_compaction(&before, "SessionStart"));
    assert!(has_compaction(&after));
    assert!(context_after_compaction(&after, "SessionStart"));
    assert!(!context_after_compaction(&after, "PostCompact"));
}

fn hook_line(kind: &str, content: &str) -> Value {
    json!({"type": "attachment", "attachment": {"type": kind, "content": content}})
}

#[test]
fn where_shown_names_each_place_a_text_shows() {
    let lines = [
        hook_line("hook_success", "word"),
        hook_line("hook_non_blocking_error", "word"),
        json!({"type": "system", "subtype": "stop_hook_summary", "content": "word"}),
        json!({"type": "user", "message": {"content": "<local-command-stdout>word</local-command-stdout>"}}),
        json!({"type": "assistant", "message": {"content": [{"type": "text", "text": "word"}]}}),
        hook_line("hook_success", "other"),
    ];

    assert_eq!(
        where_shown(&lines, "word"),
        BTreeSet::from([
            Shown::HookAttachment,
            Shown::CommandOutput,
            Shown::Other("assistant".into())
        ])
    );
    assert!(where_shown(&lines, "absent").is_empty());
}

#[test]
fn where_shown_names_an_other_line_by_its_kind() {
    let lines = [
        json!({"type": "user", "message": {"content": "word"}}),
        hook_line("hook_additional_context", "word"),
    ];

    assert_eq!(
        where_shown(&lines, "word"),
        BTreeSet::from([
            Shown::Other("user".into()),
            Shown::Other("attachment/hook_additional_context".into())
        ])
    );
}

#[test]
fn context_per_compaction_counts_each_stretch() {
    let with = |hook: &str, text: &str| {
        json!({"type": "attachment",
               "attachment": {"type": "hook_additional_context", "hookEvent": hook, "content": [text]}})
    };
    let boundary = json!({"type": "system", "subtype": "compact_boundary"});
    let lines = [
        with("SessionStart", "word"),
        with("SessionStart", "other"),
        with("UserPromptSubmit", "word"),
        boundary.clone(),
        boundary,
        with("SessionStart", "word"),
        with("SessionStart", "word"),
    ];

    assert_eq!(
        context_per_compaction(&lines, "SessionStart", "word"),
        [1, 0, 2]
    );
    assert_eq!(context_per_compaction(&[], "SessionStart", "word"), [0]);
}

#[test]
fn where_shown_counts_a_local_command_line_as_command_output() {
    let lines = [json!({"type": "system", "subtype": "local_command",
                        "content": "<local-command-stderr>word</local-command-stderr>"})];

    assert_eq!(
        where_shown(&lines, "word"),
        BTreeSet::from([Shown::CommandOutput])
    );
}

fn said(text: &str) -> Value {
    json!({"type": "assistant", "message": {"content": [{"type": "text", "text": text}]}})
}

fn refused(text: &str) -> Value {
    json!({"type": "assistant", "isApiErrorMessage": true,
           "message": {"content": [{"type": "text", "text": text}]}})
}

#[test]
fn model_replies_after_reads_the_model_replies_that_follow_a_line_holding_the_text() {
    let lines = [
        said("before word"),
        json!({"type": "system", "subtype": "local_command", "content": "word"}),
        said("word"),
        refused("Prompt is too long"),
        json!({"type": "user", "message": {"content": "word again"}}),
        said("ok"),
    ];

    assert_eq!(model_replies_after(&lines, "word"), ["word", "ok"]);
    assert!(model_replies_after(&lines, "absent").is_empty());
}

#[test]
fn replies_after_skips_assistant_lines_holding_the_text() {
    let lines = [said("word"), said("ok")];

    assert!(model_replies_after(&lines, "word").is_empty());
}

#[test]
fn refusals_name_the_harness_errors_and_any_model_reply_after_the_first() {
    let lines = [
        said("file 1"),
        refused("Prompt is too long"),
        json!({"type": "user", "message": {"content": "next"}}),
        refused("Prompt is too long"),
    ];

    assert_eq!(
        refusals(&lines),
        ["Prompt is too long", "Prompt is too long"]
    );
    assert!(!model_replied_after_refusal(&lines));

    let answered = [refused("Prompt is too long"), said("ok")];
    assert!(model_replied_after_refusal(&answered));
    assert!(!model_replied_after_refusal(&[said("ok")]));
}
