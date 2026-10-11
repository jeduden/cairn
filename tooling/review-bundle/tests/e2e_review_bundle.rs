//! The `review-bundle` executable as the review workflow runs it: two
//! trees and a diff in, the bundle and findings written, the route on
//! stdout and an exit code out.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::fs;
use std::process::Command;

use testkit::TempDir;

#[test]
fn the_executable_bundles_this_repository_against_itself() {
    let out = TempDir::new().unwrap();
    let diff = out.path().join("pr.diff");
    fs::write(&diff, "diff --git a/docs/srs/10-engineering-quality.md b/docs/srs/10-engineering-quality.md\n+ENG-28\n").unwrap();
    let root = testkit::repo_root();

    let run = Command::new(env!("CARGO_BIN_EXE_review-bundle"))
        .arg("--base")
        .arg(&root)
        .arg("--head")
        .arg(&root)
        .arg("--diff")
        .arg(&diff)
        .arg("--out")
        .arg(out.path())
        .output()
        .unwrap();

    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8(run.stdout).unwrap(),
        "class=deep\ntouched=1\nfindings=0\n"
    );
    let bundle = fs::read_to_string(out.path().join("bundle.md")).unwrap();
    assert!(bundle.contains("### ENG-28"), "{bundle}");
    assert_eq!(
        fs::read_to_string(out.path().join("checked.json")).unwrap(),
        "[]\n"
    );
}

#[test]
fn the_executable_exits_2_on_a_wrong_command_line() {
    let run = Command::new(env!("CARGO_BIN_EXE_review-bundle"))
        .arg("--nope")
        .output()
        .unwrap();

    assert_eq!(run.status.code(), Some(2));
}
