//! The transcript contract (ENG-17, REC-26) against the real recordings
//! under `testdata/claude-code`: the recorded shapes the core compiles
//! in are exactly what the recordings give, every recorded transcript
//! replays through the core's parser with no unparsed line or format
//! drift, and no fixture carries a residual identifier.
//!
//! `CAIRN_REGENERATE_SHAPES=1 cargo test -p harness-contract --test
//! contract` writes the shapes afresh after a new recording.

use std::path::PathBuf;

use harness_contract::{recording, replay, residual, shapes};
use redaction::Redactor;
use transcript::shape::Shapes;

fn recordings() -> PathBuf {
    testkit::repo_root().join("testdata/claude-code")
}

#[test]
fn recorded_shapes_regenerate_byte_for_byte() {
    let write = std::env::var_os("CAIRN_REGENERATE_SHAPES").is_some();
    let problems = shapes::regenerate(
        &recordings(),
        &testkit::repo_root().join(shapes::SHAPES_DIR),
        write,
    )
    .unwrap();

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn every_recorded_transcript_replays_without_an_unparsed_line_or_format_drift() {
    let supported = Shapes::supported().unwrap();
    let redactor = Redactor::new(&[]).unwrap().0;
    let mut problems = Vec::new();
    let mut replayed = 0;
    for version in recording::versions(&recordings()).unwrap() {
        for path in shapes::transcripts(&version).unwrap() {
            problems.extend(replay::replay_file(&path, &supported, &redactor).unwrap());
            replayed += 1;
        }
    }

    assert!(replayed > 0, "no recorded transcript");
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn recorded_fixtures_hold_no_residual_identifier() {
    let problems = residual::check(&recordings()).unwrap();

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
