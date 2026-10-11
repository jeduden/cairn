use std::fs;

use super::*;
use serde_json::json;
use testkit::TempDir;

fn write(dir: &Path, rel: &str, body: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

const S: &str = "-w-a/s1.jsonl";
const A: &str = "-w-a/s1/subagents/agent-x.jsonl";
const META: &str = "-w-a/s1/subagents/agent-x.meta.json";

fn sample() -> TempDir {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        S,
        "{\"sessionId\":\"s1\"}\n{\"type\":\"last-prompt\"}\n",
    );
    write(tmp.path(), A, "{\"sessionId\":\"s1\",\"agentId\":\"x\"}\n");
    write(tmp.path(), META, r#"{"agentType":"general-purpose"}"#);
    write(tmp.path(), "-w-a/memory/notes.md", "");
    write(tmp.path(), "-w-a/s1/stray.jsonl", "");
    write(tmp.path(), "-w-a/s1/subagents/agent-x.meta.json.bak", "");
    write(tmp.path(), "loose.txt", "");
    tmp
}

#[test]
fn walk_finds_sessions_subagents_and_strays() {
    let tmp = sample();

    let tree = walk(tmp.path()).unwrap();

    assert_eq!(
        tree.harness_sessions,
        [HarnessSession {
            dir: "-w-a".into(),
            id: "s1".into(),
            transcript: tmp.path().join(S)
        }]
    );
    assert_eq!(
        tree.subagents,
        [Subagent {
            harness_session: "s1".into(),
            agent: "x".into(),
            transcript: tmp.path().join(A)
        }]
    );
    assert_eq!(tree.strays, [tmp.path().join("-w-a/s1/stray.jsonl")]);
}

#[test]
fn walk_fails_on_a_missing_tree() {
    let tmp = TempDir::new().unwrap();

    assert!(walk(&tmp.path().join("gone")).is_err());
}

#[test]
fn a_sound_tree_has_no_problems() {
    let tmp = sample();
    let tree = walk(tmp.path()).unwrap();

    assert!(
        harness_session_problems(&tree.harness_sessions[0])
            .unwrap()
            .is_empty()
    );
    assert!(
        subagent_problems(&tree.subagents[0], &tree)
            .unwrap()
            .is_empty()
    );
    assert!(meta_problems(&tree.subagents[0]).is_empty());
}

#[test]
fn session_problems_name_a_foreign_line() {
    let tmp = sample();
    write(tmp.path(), S, "{\"sessionId\":\"other\"}\n");
    let tree = walk(tmp.path()).unwrap();

    let problems = harness_session_problems(&tree.harness_sessions[0]).unwrap();

    assert_eq!(problems.len(), 1);
    assert!(problems[0].ends_with("a line names sessionId other, not s1"));
}

#[test]
fn subagent_problems_name_each_fault() {
    let tmp = sample();
    write(tmp.path(), A, "{\"sessionId\":\"s1\",\"agentId\":\"y\"}\n");
    write(tmp.path(), META, "{}");
    let mut tree = walk(tmp.path()).unwrap();
    tree.harness_sessions.clear();

    let problems = subagent_problems(&tree.subagents[0], &tree).unwrap();

    assert_eq!(problems.len(), 2);
    assert!(problems[0].ends_with("a line names agentId y, not x"));
    assert!(problems[1].ends_with("harness session s1 has no transcript"));
    assert_eq!(meta_problems(&tree.subagents[0]).len(), 1);
    assert!(meta_problems(&tree.subagents[0])[0].ends_with("names no agentType"));
}

#[test]
fn meta_problems_name_a_missing_meta() {
    let tmp = sample();
    fs::remove_file(tmp.path().join(META)).unwrap();
    let tree = walk(tmp.path()).unwrap();

    let problems = meta_problems(&tree.subagents[0]);

    assert_eq!(problems.len(), 1);
    assert!(problems[0].starts_with("read "));
}

#[test]
fn problems_fail_on_an_unreadable_transcript() {
    let tmp = sample();
    let tree = walk(tmp.path()).unwrap();
    fs::remove_file(tmp.path().join(S)).unwrap();
    fs::remove_file(tmp.path().join(A)).unwrap();

    assert!(harness_session_problems(&tree.harness_sessions[0]).is_err());
    assert!(subagent_problems(&tree.subagents[0], &tree).is_err());
}

#[test]
fn transcript_dir_is_the_parent_directory_name() {
    let input = json!({"transcript_path": "/home/u/.claude/projects/-w-my-app/s.jsonl"});

    assert_eq!(transcript_dir(&input), "-w-my-app");
    assert_eq!(transcript_dir(&json!({})), "");
}

#[test]
fn a_transcript_naming_no_harness_session_is_a_problem() {
    let tmp = sample();
    write(tmp.path(), S, "{\"type\":\"last-prompt\"}\n");
    let tree = walk(tmp.path()).unwrap();

    let problems = harness_session_problems(&tree.harness_sessions[0]).unwrap();

    assert_eq!(problems.len(), 1);
    assert!(problems[0].ends_with("no line names its sessionId"));
}

#[test]
fn duplicates_name_an_id_in_two_directories() {
    let tmp = sample();
    write(tmp.path(), "-w-b/s1.jsonl", "{\"sessionId\":\"s1\"}\n");
    let tree = walk(tmp.path()).unwrap();

    assert_eq!(duplicates(&tree), ["harness session s1 has 2 transcripts"]);
    assert!(duplicates(&walk(sample().path()).unwrap()).is_empty());
}

#[test]
fn shared_directory_checks_mapping_presence_and_distinctness() {
    let tmp = sample();
    write(tmp.path(), "-w-a/s2.jsonl", "{\"sessionId\":\"s2\"}\n");
    let tree = walk(tmp.path()).unwrap();
    let input = |cwd: &str, id: &str, dir: &str| json!({"cwd": cwd, "session_id": id, "transcript_path": format!("/p/{dir}/{id}.jsonl")});
    let cwds = ["/w/a_".to_owned(), "/w/a.".to_owned()];

    let good = [input("/w/a_", "s1", "-w-a"), input("/w/a.", "s2", "-w-a")];
    assert!(shared_directory(&good, &cwds, "-w-a", &tree).is_empty());

    let bad = [input("/w/a_", "s1", "-w-b"), input("/w/a.", "s1", "-w-a")];
    assert_eq!(
        shared_directory(&bad, &cwds, "-w-a", &tree),
        [
            "/w/a_ maps to -w-b, not -w-a",
            "the harness sessions are not distinct"
        ]
    );

    let absent = [input("/w/a_", "s9", "-w-a")];
    assert_eq!(
        shared_directory(&absent, &cwds, "-w-a", &tree),
        [
            "the tree holds no transcript of s9 under -w-a",
            "no harness session recorded in /w/a.",
            "the harness sessions are not distinct"
        ]
    );
}
