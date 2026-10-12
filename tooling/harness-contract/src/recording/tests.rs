use super::*;
use serde_json::json;
use testkit::TempDir;

fn write(dir: &Path, rel: &str, body: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

#[test]
fn versions_skip_the_record_kit_and_files() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "2.1.296/hooks/x/01-A.json", "{}");
    write(tmp.path(), "2.0.1/cleanup.json", "{}");
    write(tmp.path(), "record/cc", "");
    write(tmp.path(), "notes.txt", "");

    let names: Vec<_> = versions(tmp.path())
        .unwrap()
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();

    assert_eq!(names, ["2.0.1", "2.1.296"]);
}

#[test]
fn versions_fail_without_a_version() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "record/cc", "");

    assert!(
        versions(tmp.path())
            .unwrap_err()
            .contains("no recorded version")
    );
}

#[test]
fn entries_fail_on_a_missing_directory() {
    let tmp = TempDir::new().unwrap();

    assert!(
        entries(&tmp.path().join("gone"))
            .unwrap_err()
            .starts_with("list ")
    );
}

#[test]
fn files_recurse_and_filter_by_suffix() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "a.jsonl", "");
    write(tmp.path(), "s/subagents/agent-1.jsonl", "");
    write(tmp.path(), "s/subagents/agent-1.meta.json", "");

    let found = files(tmp.path(), ".jsonl").unwrap();

    assert_eq!(
        found,
        [
            tmp.path().join("a.jsonl"),
            tmp.path().join("s/subagents/agent-1.jsonl")
        ]
    );
}

#[test]
fn files_fail_on_a_missing_directory() {
    let tmp = TempDir::new().unwrap();

    assert!(files(&tmp.path().join("gone"), ".json").is_err());
}

#[test]
fn json_reads_a_document_and_names_a_bad_one() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "good.json", r#"{"a": 1}"#);
    write(tmp.path(), "bad.json", "{");

    assert_eq!(
        json(&tmp.path().join("good.json")).unwrap(),
        json!({"a": 1})
    );
    assert!(
        json(&tmp.path().join("bad.json"))
            .unwrap_err()
            .contains("bad.json")
    );
    assert!(
        json(&tmp.path().join("gone.json"))
            .unwrap_err()
            .starts_with("read ")
    );
}

#[test]
fn parse_lines_skips_blank_lines_and_names_a_bad_one() {
    assert_eq!(
        parse_lines("{\"a\":1}\n\n{\"b\":2}\n").unwrap(),
        [json!({"a": 1}), json!({"b": 2})]
    );
    assert_eq!(
        parse_lines("{}\nnope").unwrap_err().split(':').next(),
        Some("line 2")
    );
}

#[test]
fn lines_read_a_file_and_name_it_on_failure() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "t.jsonl", "{\"type\":\"user\"}\n");
    write(tmp.path(), "bad.jsonl", "nope\n");

    assert_eq!(
        lines(&tmp.path().join("t.jsonl")).unwrap(),
        [json!({"type": "user"})]
    );
    assert!(
        lines(&tmp.path().join("bad.jsonl"))
            .unwrap_err()
            .contains("bad.jsonl")
    );
    assert!(
        lines(&tmp.path().join("gone.jsonl"))
            .unwrap_err()
            .starts_with("read ")
    );
}

#[test]
fn hook_inputs_read_a_flow_in_firing_order() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "hooks/startup/02-Stop.json",
        r#"{"hook_event_name":"Stop"}"#,
    );
    write(
        tmp.path(),
        "hooks/startup/01-SessionStart.json",
        r#"{"hook_event_name":"SessionStart"}"#,
    );

    let inputs = hook_inputs(tmp.path(), "startup").unwrap();

    assert_eq!(
        inputs,
        [
            json!({"hook_event_name": "SessionStart"}),
            json!({"hook_event_name": "Stop"})
        ]
    );
}

#[test]
fn hook_inputs_fail_on_an_empty_or_unreadable_flow() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "hooks/empty/notes.txt", "");
    write(tmp.path(), "hooks/bad/01-A.json", "{");

    assert!(
        hook_inputs(tmp.path(), "empty")
            .unwrap_err()
            .contains("no hook input")
    );
    assert!(hook_inputs(tmp.path(), "bad").is_err());
    assert!(hook_inputs(tmp.path(), "gone").is_err());
}

#[test]
fn hook_inputs_refuse_an_unnamed_flow() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "hooks/startup/01-A.json", "{}");

    assert_eq!(
        hook_inputs(tmp.path(), "").unwrap_err(),
        "no hook flow named"
    );
}

#[test]
fn named_reads_a_string_and_refuses_an_empty_or_missing_one() {
    let m = json!({"a": {"b": "x"}, "e": "", "n": 1});

    assert_eq!(named(&m, "a.b").unwrap(), "x");
    assert!(named(&m, "e").is_err());
    assert!(named(&m, "n").is_err());
    assert_eq!(named(&m, "gone").unwrap_err(), "the manifest names no gone");
}

#[test]
fn pairs_read_an_object_of_strings() {
    let m = json!({"k": {"a": "1", "b": "2"}, "empty": {}, "bad": {"a": 1}});

    assert_eq!(
        pairs(&m, "k").unwrap(),
        [
            ("a".to_owned(), "1".to_owned()),
            ("b".to_owned(), "2".to_owned())
        ]
    );
    assert!(pairs(&m, "empty").is_err());
    assert!(pairs(&m, "gone").is_err());
    assert_eq!(
        pairs(&m, "bad").unwrap_err(),
        "the manifest's bad.a is not a string"
    );
}
