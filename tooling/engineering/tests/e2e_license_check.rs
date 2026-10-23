//! The `license-check` executable as CI runs it: a checkout in, a
//! verdict on the terminal and an exit code out.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::fs;
use std::process::{Command, Output};

use testkit::TempDir;

/// A workspace with one product binary and no dependencies.
fn license_check(deny: &str) -> Output {
    let dir = TempDir::new().unwrap();
    let files = [
        (
            "Cargo.toml",
            "[workspace]\nmembers = [\"crates/app\"]\nresolver = \"3\"\n",
        ),
        (
            "Cargo.lock",
            "version = 4\n\n[[package]]\nname = \"app\"\nversion = \"0.0.0\"\n",
        ),
        (
            "crates/app/Cargo.toml",
            "[package]\nname = \"app\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
        ),
        ("crates/app/src/main.rs", "fn main() {}\n"),
        (
            ".github/workflows/build.yml",
            "env:\n  LINUX_TARGETS: >-\n    x86_64-unknown-linux-musl:linux-amd64\n",
        ),
        ("deny.toml", deny),
    ];
    for (name, body) in files {
        let path = dir.path().join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    Command::new(env!("CARGO_BIN_EXE_license-check"))
        .args(["--root"])
        .arg(dir.path())
        .output()
        .unwrap()
}

#[test]
fn the_executable_passes_a_checkout_that_links_no_unlicensed_crate() {
    let out = license_check("[licenses]\nallow = [\"MIT\"]\n");

    assert!(out.status.success(), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stdout).contains("has a license text"));
}

#[test]
fn the_executable_fails_a_license_deny_toml_grants_without_a_text() {
    let out = license_check("[licenses]\nallow = [\"WTFPL\"]\n");

    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("license-check: deny.toml grants WTFPL"));
}

#[test]
fn the_executable_refuses_an_unknown_flag() {
    let out = Command::new(env!("CARGO_BIN_EXE_license-check"))
        .arg("--bogus")
        .output()
        .unwrap();

    assert_eq!(out.status.code(), Some(2));
}
