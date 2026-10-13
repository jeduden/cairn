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
fn run_fails_naming_the_benchmark_the_core_cannot_start_and_exits_2_on_bad_usage() {
    let dir = TempDir::new().unwrap();

    let out = benchmark(&dir, &["run", "--core", "/no/such/cairn"]);

    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out.stderr).starts_with("benchmark: core_start_up: start /no/such/cairn: "),
        "{}",
        text(&out.stderr)
    );

    let out = benchmark(&dir, &["gate"]);

    assert_eq!(out.status.code(), Some(2));
    assert!(text(&out.stderr).contains("usage: benchmark run"));
}

/// Gates `head_ns` against a base of 800 µs, thirty samples a side.
fn gate(dir: &TempDir, head_ns: u64) -> Output {
    fs::write(dir.path().join("base.samples"), samples(800_000, 30)).unwrap();
    fs::write(dir.path().join("head.samples"), samples(head_ns, 30)).unwrap();
    benchmark(
        dir,
        &["gate", "--base", "base.samples", "--head", "head.samples"],
    )
}

#[test]
fn gate_holds_within_the_threshold_and_fails_naming_the_benchmark_beyond_it() {
    let dir = TempDir::new().unwrap();

    let held = gate(&dir, 840_000);

    assert!(held.status.success(), "{}", text(&held.stderr));
    assert!(text(&held.stdout).ends_with("| held |\n"));

    let regressed = gate(&dir, 960_000);

    assert_eq!(regressed.status.code(), Some(1));
    assert!(text(&regressed.stdout).contains("| core_start_up | 800.0 µs | 960.0 µs | +20.0% |"));
    assert!(
        text(&regressed.stderr).starts_with("benchmark: core_start_up regressed by 20.0%"),
        "{}",
        text(&regressed.stderr)
    );
}
