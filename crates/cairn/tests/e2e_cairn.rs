//! The built `cairn` executable, as a person or a hook runs it,
//! confined to the core's boundary: all network denied (ENG-12). Every
//! start goes through the testkit's confinement, so a socket the core
//! opens fails the test that started it, naming the core and the test.
//! This file names no `std::process::Command`: the crate's clippy.toml
//! refuses one, so the core cannot be started here unconfined.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]
#![allow(
    clippy::panic,
    reason = "a start the confinement refuses fails the test with its message"
)]

use std::path::Path;
use std::process::Output;

use testkit::boundary::{Component, Confinement};

/// Runs the core with `args`, confined and with a fresh home. A socket
/// it opens fails the calling test, named.
#[track_caller]
fn cairn(args: &[&str]) -> Output {
    let home = testkit::TempDir::new().unwrap();

    Confinement::of(Component::Core)
        .and_then(|c| c.run(Path::new(env!("CARGO_BIN_EXE_cairn")), args, home.path()))
        .unwrap_or_else(|e| panic!("{e}"))
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
