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
