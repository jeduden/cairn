use super::*;
use serde_json::json;

#[test]
fn a_main_transcript_s_lines_belong_to_its_harness_session_s_main_run() {
    let lines = [
        json!({"type": "mode"}),
        json!({"type": "user", "uuid": "u1", "sessionId": "s1"}),
    ];
    let defaults = Defaults::of(&lines, "stem");

    let main = RunKey {
        agent_id: None,
        harness_session: "s1".into(),
    };
    assert_eq!(defaults.own(), main);
    assert_eq!(defaults.key(&lines[0]), main);
    assert_eq!(defaults.key(&lines[1]), main);
}

#[test]
fn a_subagent_transcript_s_lines_belong_to_its_agent_s_run() {
    let lines = [
        json!({"type": "user", "uuid": "u1", "sessionId": "s1", "agentId": "a1"}),
        json!({"type": "last-prompt"}),
        json!({"type": "user", "uuid": "u2", "sessionId": "s1", "agentId": "a2"}),
    ];
    let defaults = Defaults::of(&lines, "agent-a1");

    let sub = |a: &str| RunKey {
        agent_id: Some(a.into()),
        harness_session: "s1".into(),
    };
    assert_eq!(defaults.own(), sub("a1"));
    assert_eq!(defaults.key(&lines[1]), sub("a1"));
    // A transcript holding several agents splits by run.
    assert_eq!(defaults.key(&lines[2]), sub("a2"));
}

#[test]
fn a_transcript_naming_no_harness_session_takes_its_stem() {
    let lines = [
        json!({"type": "user", "uuid": "u", "sessionId": ""}),
        json!("x"),
    ];

    let defaults = Defaults::of(&lines, "main-run");

    assert_eq!(defaults.own().harness_session, "main-run");
    assert_eq!(defaults.key(&lines[1]).agent_id, None);
}
