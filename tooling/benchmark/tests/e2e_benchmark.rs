//! The `benchmark` executable as the CI job runs it: `run` samples the
//! suite against a core executable, and `gate` decides from two samples
//! files, failing naming the benchmark that regressed.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::env;
use std::fs;
use std::process::{Command, Output};

use testkit::TempDir;

fn benchmark(dir: &TempDir, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_benchmark"))
        .current_dir(dir.path())
        .args(args)
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap()
}

/// `n` samples of the core's start-up near `ns`.
fn samples(ns: u64, n: u64) -> String {
    (0..n).fold(String::from("# benchmark nanoseconds\n"), |out, i| {
        format!("{out}core_start_up {}\n", ns + i % 7)
    })
}

/// This test executable stands in for the core: `run` starts it with
/// `version`, which it takes as a test filter that no test here
/// matches, so it runs nothing and exits 0.
#[test]
fn run_samples_the_suite_against_the_core_it_is_given() {
    let dir = TempDir::new().unwrap();
    let core = env::current_exe().unwrap();

    let out = benchmark(
        &dir,
        &["run", "--core", core.to_str().unwrap(), "--samples", "3"],
    );

    assert!(out.status.success(), "{}", text(&out.stderr));
    let samples = text(&out.stdout);
    assert!(
        samples.starts_with("# benchmark nanoseconds\n"),
        "{samples}"
    );
    assert_eq!(
        samples
            .lines()
            .filter(|l| l.starts_with("core_start_up "))
            .count(),
        3
    );
}

#[test]
fn run_fails_naming_the_benchmark_when_the_core_cannot_start() {
    let dir = TempDir::new().unwrap();

    let out = benchmark(&dir, &["run", "--core", "/no/such/cairn"]);

    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out.stderr).starts_with("benchmark: core_start_up: start /no/such/cairn: "),
        "{}",
        text(&out.stderr)
    );
}

#[test]
fn gate_fails_naming_the_benchmark_that_regressed() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("base.samples"), samples(800_000, 30)).unwrap();
    fs::write(dir.path().join("head.samples"), samples(960_000, 30)).unwrap();

    let out = benchmark(
        &dir,
        &["gate", "--base", "base.samples", "--head", "head.samples"],
    );

    assert_eq!(out.status.code(), Some(1));
    assert!(text(&out.stdout).contains("| core_start_up | 800.0 µs | 960.0 µs | +20.0% |"));
    assert!(
        text(&out.stderr).starts_with("benchmark: core_start_up regressed by 20.0%"),
        "{}",
        text(&out.stderr)
    );
}

#[test]
fn gate_holds_a_head_within_the_threshold() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("base.samples"), samples(800_000, 30)).unwrap();
    fs::write(dir.path().join("head.samples"), samples(840_000, 30)).unwrap();

    let out = benchmark(
        &dir,
        &["gate", "--base", "base.samples", "--head", "head.samples"],
    );

    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(text(&out.stdout).ends_with("| held |\n"));
}

#[test]
fn a_wrong_command_line_exits_2_with_the_usage() {
    let dir = TempDir::new().unwrap();

    let out = benchmark(&dir, &["gate"]);

    assert_eq!(out.status.code(), Some(2));
    assert!(text(&out.stderr).contains("usage: benchmark run"));
}
