//! The requirement–scenario gate on the real repository.

/// Every SRS requirement has its one scenario under features/, tagged
/// with the row's priority and invariants, and no scenario names an id
/// the SRS lacks. Adding a requirement means adding its scenario,
/// tagged `@pending` until its steps exist, in the same change.
#[test]
fn specification_and_features_agree() {
    let root = testkit::repo_root();
    let reqs = srs::load(&root.join("docs/srs")).unwrap();
    let scenarios = scenario::scenarios(&root.join("features")).unwrap();

    let problems = scenario::check(&reqs, &scenarios);

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
