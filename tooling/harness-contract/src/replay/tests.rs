use std::fs;

use serde_json::json;
use testkit::TempDir;
use transcript::shape::RecordedShapes;

use super::*;

fn shapes() -> Shapes {
    let mode = json!({"type": "mode", "mode": "normal"});
    Shapes::of(vec![RecordedShapes::generate(
        "1.0.0",
        &[mode.as_object().cloned().unwrap()],
    )])
}

fn redactor() -> Redactor {
    Redactor::new(&[]).unwrap().0
}

#[test]
fn a_transcript_that_fits_its_shapes_replays_clean() {
    let body = "{\"type\":\"mode\",\"mode\":\"plan\"}\n\n{\"type\":\"mode\",\"mode\":\"normal\"}\n";

    assert_eq!(replay(body, &shapes(), &redactor()), Vec::<String>::new());
}

#[test]
fn replaying_names_each_unparsed_and_drifting_line() {
    let body = "{x\n{\"type\":\"future_kind\"}\n{\"type\":\"mode\"}\n{\"type\":\"mode\",\"mode\":\"n\",\"extra\":1}\n";

    assert_eq!(
        replay(body, &shapes(), &redactor()),
        [
            "line 1: unparsed",
            "line 2: format drift claude-code:unversioned:future_kind:unrecorded-type",
            "line 3: format drift claude-code:unversioned:mode:missing:mode",
            "line 4: format drift claude-code:unversioned:mode:beyond:extra",
        ]
    );
}

#[test]
fn replaying_a_file_leads_each_problem_with_its_path() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.jsonl");
    fs::write(&path, "{x\n").unwrap();

    let problems = replay_file(&path, &shapes(), &redactor()).unwrap();
    assert_eq!(problems, [format!("{}: line 1: unparsed", path.display())]);
    assert!(replay_file(&dir.path().join("none"), &shapes(), &redactor()).is_err());
}
