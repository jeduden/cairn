//! The built `cairn` executable, as a person or a hook runs it.

#![allow(
    clippy::disallowed_types,
    reason = "the test starts the built executable; the shipped code starts no program"
)]
#![allow(
    clippy::disallowed_methods,
    reason = "the test waits for the executable it started; the test run's own limit bounds it"
)]
#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::process::{Command, Output};

fn cairn(args: &[&str]) -> Output {
    let home = testkit::TempDir::new().unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_cairn"));
    command.args(args);

    testkit::isolate(&mut command, home.path())
        .output()
        .unwrap()
}

#[test]
fn version_prints_the_build_s_version() {
    let out = cairn(&["version"]);

    assert!(out.status.success());
    let want = option_env!("CAIRN_VERSION").unwrap_or("dev");
    assert_eq!(String::from_utf8_lossy(&out.stdout), format!("{want}\n"));
}

#[test]
fn version_json_is_one_json_object() {
    let out = cairn(&["version", "--json"]);

    let got: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        got["version"],
        option_env!("CAIRN_VERSION").unwrap_or("dev")
    );
}

#[test]
fn a_wrong_command_line_exits_2_with_the_usage() {
    for args in [&[][..], &["ingest"], &["version", "--yaml"]] {
        let out = cairn(args);

        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert!(out.stdout.is_empty(), "{args:?}");
    }
}
