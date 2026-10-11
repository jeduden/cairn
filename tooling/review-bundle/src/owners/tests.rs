use super::*;

#[test]
fn patterns_reads_the_first_field_of_each_rule() {
    let got = patterns("# owners\n\n/docs/srs/   @a\n/Cargo.toml @a @b\n  *.lock @c\n");

    assert_eq!(got, ["/docs/srs/", "/Cargo.toml", "*.lock"]);
}

#[test]
fn owns_reads_an_anchored_directory_as_its_contents() {
    assert!(owns("/docs/srs/", "docs/srs/index.md"));
    assert!(owns("/docs/srs/", "docs/srs/a/b.md"));
    assert!(!owns("/docs/srs/", "docs/srs"));
    assert!(!owns("/docs/srs/", "docs/srsx/index.md"));
    assert!(!owns("/docs/srs/", "x/docs/srs/index.md"));
}

#[test]
fn owns_reads_an_anchored_file() {
    assert!(owns("/Cargo.toml", "Cargo.toml"));
    assert!(!owns("/Cargo.toml", "crates/cairn/Cargo.toml"));
}

#[test]
fn owns_matches_an_unanchored_pattern_at_any_depth() {
    assert!(owns("clippy.toml", "clippy.toml"));
    assert!(owns("clippy.toml", "crates/cairn/clippy.toml"));
    assert!(!owns("clippy.toml", "crates/cairn/clippy.toml.bak"));
}

#[test]
fn owns_globs_within_one_segment() {
    assert!(owns("/crates/*/clippy.toml", "crates/pin/clippy.toml"));
    assert!(!owns("/crates/*/clippy.toml", "crates/pin/src/clippy.toml"));
    assert!(owns(
        "/crates/*/tests/reach.rs",
        "crates/cairn/tests/reach.rs"
    ));
    assert!(owns("*.lock", "Cargo.lock"));
    assert!(!owns("*.lock", "Cargo.lockx"));
}

#[test]
fn owns_nothing_for_an_empty_pattern() {
    assert!(!owns("/", "README.md"));
}

#[test]
fn glob_matches_any_run_and_nothing_more() {
    assert!(glob("a*c", "abbc"));
    assert!(glob("a*", "a"));
    assert!(glob("*é", "café"));
    assert!(!glob("a*c", "abd"));
    assert!(!glob("b*", "a"));
}
