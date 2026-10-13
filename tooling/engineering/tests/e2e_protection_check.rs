//! The `protection-check` executable as the nightly workflow runs it: a
//! process with two ruleset files in, a verdict on the terminal and an
//! exit code out.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::fs;
use std::process::{Command, Output};

use testkit::TempDir;

const COMMITTED: &str = r#"{"enforcement":"active","rules":[{"type":"required_signatures"}]}"#;

fn protection_check(live: &str) -> Output {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("committed.json"), COMMITTED).unwrap();
    fs::write(dir.path().join("live.json"), live).unwrap();

    Command::new(env!("CARGO_BIN_EXE_protection-check"))
        .current_dir(dir.path())
        .args(["--committed", "committed.json", "--live", "live.json"])
        .output()
        .unwrap()
}

#[test]
fn the_executable_passes_a_live_ruleset_as_strong_as_the_committed_one() {
    let out = protection_check(COMMITTED);

    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("as strong as the committed one"));
}

#[test]
fn the_executable_fails_a_weaker_live_ruleset() {
    let out = protection_check(r#"{"enforcement":"disabled","rules":[]}"#);

    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("live has no required_signatures rule"));
}

#[test]
fn the_executable_refuses_a_missing_flag() {
    let out = Command::new(env!("CARGO_BIN_EXE_protection-check"))
        .output()
        .unwrap();

    assert_eq!(out.status.code(), Some(2));
}
