//! The repository's record of accepted regressions parses, so the CI
//! job's gate never meets a malformed one (ENG-15).

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use benchmark::accepted;

#[test]
fn the_accepted_regressions_parse() {
    let path = testkit::repo_root().join("tooling/benchmark/accepted.txt");
    let text = std::fs::read_to_string(&path).unwrap();

    let parsed = accepted::parse(&text);

    assert!(
        parsed.is_ok(),
        "{}: {}",
        path.display(),
        parsed.unwrap_err()
    );
}
