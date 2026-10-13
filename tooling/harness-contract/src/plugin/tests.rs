use super::*;
use serde_json::json;

fn names(set: &BTreeSet<String>) -> Vec<&str> {
    set.iter().map(String::as_str).collect()
}

#[test]
fn registered_lists_the_events_of_hooks_json() {
    let hooks = json!({"hooks": {
        "UserPromptSubmit": [{"hooks": [{"type": "command", "command": "x"}]}],
        "SessionStart": [{"hooks": [{"type": "command", "command": "x"}]}]
    }});

    assert_eq!(
        names(&registered(&hooks).unwrap()),
        ["SessionStart", "UserPromptSubmit"]
    );
}

#[test]
fn registered_refuses_a_document_without_hooks() {
    assert!(registered(&json!({})).unwrap_err().contains("no hooks"));
    assert!(
        registered(&json!({"hooks": {}}))
            .unwrap_err()
            .contains("no hooks")
    );
}

#[test]
fn fired_lists_each_event_once() {
    let inputs = [
        json!({"hook_event_name": "SessionStart"}),
        json!({"hook_event_name": "UserPromptSubmit"}),
        json!({"hook_event_name": "UserPromptSubmit"}),
        json!({"session_id": "s"}),
    ];

    assert_eq!(names(&fired(&inputs)), ["SessionStart", "UserPromptSubmit"]);
}

#[test]
fn unfired_names_each_registered_event_that_did_not_fire() {
    let registered: BTreeSet<String> = ["A", "B", "C"].map(String::from).into();
    let fired: BTreeSet<String> = ["B"].map(String::from).into();

    assert_eq!(
        unfired(&registered, &fired),
        [
            "A registered but did not fire",
            "C registered but did not fire"
        ]
    );
}

#[test]
fn tools_lists_the_tools_of_the_plugins_server() {
    let init = json!({"tools": [
        "Bash",
        "mcp__plugin_cairn_cairn__event_search",
        "mcp__plugin_other_cairn__event_search",
        "mcp__cairn__event_get"
    ]});

    assert_eq!(
        tools(&init, "cairn", "cairn"),
        ["mcp__plugin_cairn_cairn__event_search"]
    );
    assert!(tools(&json!({}), "cairn", "cairn").is_empty());
}

#[test]
fn connected_needs_the_plugins_server_connected() {
    let up = json!({"mcp_servers": [
        {"name": "plugin:cairn:cairn", "status": "connected", "source": "plugin"}
    ]});
    let failed = json!({"mcp_servers": [
        {"name": "plugin:cairn:cairn", "status": "failed"}
    ]});

    assert!(connected(&up, "cairn", "cairn"));
    assert!(!connected(&failed, "cairn", "cairn"));
    assert!(!connected(&up, "cairn", "other"));
    assert!(!connected(&json!({}), "cairn", "cairn"));
}

fn log() -> Vec<Value> {
    vec![
        json!({"kind": "start", "pid": 7}),
        json!({"kind": "in", "msg": {"method": "tools/call"}}),
        json!({"kind": "bind", "tool_use_id": "toolu_1", "run": "s1",
               "transcripts": ["/home/u/.claude/projects/-w-ws/s1.jsonl"]}),
        json!({"kind": "key", "run": "s1", "key": "K1"}),
        json!({"kind": "bind", "tool_use_id": "toolu_2", "run": "agent-a1",
               "transcripts": ["/home/u/.claude/projects/-w-ws/s1/subagents/agent-a1.jsonl"]}),
        json!({"kind": "key", "run": "agent-a1", "key": "K2"}),
    ]
}

#[test]
fn server_log_reads_processes_binds_and_keys() {
    let read = server_log(&log());

    assert_eq!(read.pids, [7].into());
    assert_eq!(
        read.keys,
        [("s1".into(), "K1".into()), ("agent-a1".into(), "K2".into())]
    );
    assert_eq!(read.binds.len(), 2);
    assert_eq!(read.binds[1].tool_use_id, "toolu_2");
    assert_eq!(
        read.binds[1].transcripts,
        ["/home/u/.claude/projects/-w-ws/s1/subagents/agent-a1.jsonl"]
    );
}

#[test]
fn unbound_keys_accepts_a_key_bound_through_its_runs_transcript() {
    assert!(unbound_keys(&server_log(&log())).is_empty());
}

#[test]
fn unbound_keys_names_a_key_with_no_bind_or_the_wrong_transcript() {
    let mut lines = log();
    lines.push(json!({"kind": "key", "run": "unnamed", "key": "K3"}));
    lines.push(
        json!({"kind": "bind", "tool_use_id": "toolu_3", "run": "s2",
                      "transcripts": ["/p/s1.jsonl", "/p/s2.jsonl"]}),
    );
    lines.push(json!({"kind": "key", "run": "s2", "key": "K4"}));

    assert_eq!(
        unbound_keys(&server_log(&lines)),
        [
            "the run-seat key of run unnamed was bound through no single transcript of that run",
            "the run-seat key of run s2 was bound through no single transcript of that run"
        ]
    );
}

#[test]
fn run_of_transcript_is_its_file_stem() {
    assert_eq!(
        run_of_transcript("/p/s1/subagents/agent-a1.jsonl"),
        "agent-a1"
    );
    assert_eq!(run_of_transcript("s1.jsonl"), "s1");
}

#[test]
fn carriers_names_each_place_a_needle_shows() {
    let places = [
        ("t.jsonl".to_owned(), "a K1 b".to_owned()),
        ("h.json".to_owned(), "nothing".to_owned()),
        ("u.json".to_owned(), "K2 and K1".to_owned()),
    ];
    let keys = ["K1".to_owned(), "K2".to_owned()];

    assert_eq!(
        carriers(&places, &keys),
        [
            "K1 shows in t.jsonl",
            "K1 shows in u.json",
            "K2 shows in u.json"
        ]
    );
    assert!(carriers(&places, &["K9".to_owned()]).is_empty());
}

#[test]
fn servers_lists_the_mcp_servers_a_plugin_bundles() {
    let mcp = json!({"mcpServers": {"cairn": {"command": "x"}, "b": {"command": "y"}}});

    assert_eq!(servers(&mcp).unwrap(), ["b", "cairn"]);
}

#[test]
fn servers_refuses_a_plugin_without_one() {
    assert!(servers(&json!({})).unwrap_err().contains("no MCP server"));
    assert!(
        servers(&json!({"mcpServers": {}}))
            .unwrap_err()
            .contains("no MCP server")
    );
}
