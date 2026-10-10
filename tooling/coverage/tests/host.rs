//! The real host: cargo and the file system.

use coverage::{Cargo, Host};
use testkit::TempDir;

fn args(a: &[&str]) -> Vec<String> {
    a.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn cargo_runs_and_returns_its_output() {
    let cargo = Cargo::from_env();

    assert!(
        cargo
            .cargo_output(&args(&["--version"]))
            .unwrap()
            .starts_with("cargo ")
    );
    assert!(cargo.cargo(&args(&["--version"])).is_ok());
    assert!(
        cargo
            .cargo(&args(&["no-such-command"]))
            .unwrap_err()
            .starts_with("cargo no-such-command: ")
    );
    assert!(
        cargo
            .cargo_output(&args(&["no-such-command"]))
            .unwrap_err()
            .starts_with("cargo no-such-command: ")
    );
}

#[test]
fn files_are_written_and_appended() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("cov/unit.lcov");
    let cargo = Cargo::from_env();

    cargo.write(&path, "a").unwrap();
    cargo.append(&path, "b").unwrap();

    assert_eq!(std::fs::read_to_string(&path).unwrap(), "ab");
    assert!(
        cargo
            .write(&dir.path().join("cov/unit.lcov/x"), "a")
            .unwrap_err()
            .starts_with("write ")
    );
    assert!(
        cargo
            .append(&dir.path().join("missing/x"), "a")
            .unwrap_err()
            .starts_with("append to ")
    );
}
