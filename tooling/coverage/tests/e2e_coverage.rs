//! The `coverage` executable as CI runs it, short of a full
//! measurement: its command line.

#![allow(
    clippy::panic,
    reason = "a test's helpers panic: a failure is the assertion"
)]

use std::process::Command;

fn coverage(args: &[&str]) -> std::process::Output {
    let out = Command::new(env!("CARGO_BIN_EXE_coverage"))
        .args(args)
        .env_remove("GITHUB_STEP_SUMMARY")
        .output();
    out.unwrap_or_else(|e| panic!("run coverage: {e}"))
}

#[test]
fn the_executable_explains_itself() {
    let out = coverage(&["--help"]);

    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).starts_with("usage: coverage"));
}

#[test]
fn the_executable_exits_2_on_a_wrong_command_line() {
    let out = coverage(&["--bogus"]);

    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("unexpected argument"));
}
