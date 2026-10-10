//! The repository the tests inspect.

#[test]
fn repo_root_holds_the_workspace_manifest() {
    assert!(testkit::repo_root().join("Cargo.toml").is_file());
}
