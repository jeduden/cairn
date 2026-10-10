//! The real host against this workspace: what the shape count reads.

use coverage::{Cargo, Host};

#[test]
fn the_real_host_reads_this_workspace_s_tests_and_scenarios() {
    let root = testkit::repo_root();
    let cargo = Cargo::from_env();

    let sources = cargo.sources(&root.join("tooling/coverage")).unwrap();
    assert!(
        sources
            .iter()
            .any(|(p, text)| p.ends_with("src/host/tests.rs") && text.contains("#[test]"))
    );
    assert!(cargo.bound_scenarios(&root.join("features")).unwrap() >= 6);
}
