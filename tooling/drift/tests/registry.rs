//! The registry against the repository it injects into.

/// Every case's injection still finds its target in the real checkout,
/// and every case names what it guards and what its check must print.
#[test]
fn cases_apply_to_the_repository() {
    let root = testkit::repo_root();
    let cases = drift::cases();

    assert!(!cases.is_empty());
    let mut problems = Vec::new();
    for c in &cases {
        if let Err(err) = drift::validate(&root, &c.injection) {
            problems.push(format!("{}: {err}", c.name));
        }
        if c.guards.is_empty() || c.want.is_empty() {
            problems.push(format!("{}: names no guard or no message", c.name));
        }
    }

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
