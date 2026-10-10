//! The `coverage` executable as CI runs it, short of a full
//! measurement: its command line.

#![allow(
    clippy::panic,
    reason = "a test's helpers panic: a failure is the assertion"
)]

use std::process::Command;

fn coverage(args: &[&str], cargo: &str) -> std::process::Output {
    let home = testkit::TempDir::new().unwrap_or_else(|e| panic!("temp dir: {e}"));
    let mut command = Command::new(env!("CARGO_BIN_EXE_coverage"));
    command
        .args(args)
        .env("CARGO", cargo)
        .env_remove("GITHUB_STEP_SUMMARY")
        .env_remove(coverage::MEASURING);
    testkit::isolate(&mut command, home.path())
        .output()
        .unwrap_or_else(|e| panic!("run coverage: {e}"))
}

#[test]
fn the_executable_explains_itself() {
    let out = coverage(&["--help"], "cargo");

    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).starts_with("usage: coverage"));
}

#[test]
fn the_executable_exits_2_on_a_wrong_command_line() {
    let out = coverage(&["--bogus"], "cargo");

    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("unexpected argument"));
}

#[test]
fn the_executable_exits_1_when_cargo_cannot_run() {
    let out = coverage(&[], "/nonexistent/cargo");

    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).starts_with("coverage: run cargo metadata"));
}

#[test]
fn the_executable_refuses_to_measure_inside_a_measurement() {
    let home = testkit::TempDir::new().unwrap_or_else(|e| panic!("temp dir: {e}"));
    let mut command = Command::new(env!("CARGO_BIN_EXE_coverage"));
    command.env(coverage::MEASURING, "1");

    let out = testkit::isolate(&mut command, home.path())
        .output()
        .unwrap_or_else(|e| panic!("run coverage: {e}"));

    assert_eq!(out.status.code(), Some(2));
}
