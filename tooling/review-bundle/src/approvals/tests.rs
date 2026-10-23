use super::*;

const CODEOWNERS: &str =
    "/tooling/ @stake\n/crates/*/clippy.toml @stake\n/crates/pin/ @sec\n/crates/redaction/ @sec\n";

fn review(user: &str, state: &str, commit: &str) -> Review {
    Review {
        user: user.to_owned(),
        state: state.to_owned(),
        commit_id: commit.to_owned(),
    }
}

fn files(paths: &[&str]) -> Vec<String> {
    paths.iter().map(|p| (*p).to_owned()).collect()
}

#[test]
fn security_patterns_are_the_owned_crate_directories() {
    assert_eq!(
        security_patterns(CODEOWNERS),
        [
            ("/crates/pin/".to_owned(), vec!["@sec".to_owned()]),
            ("/crates/redaction/".to_owned(), vec!["@sec".to_owned()])
        ]
    );
}

#[test]
fn a_change_outside_the_security_crates_needs_nothing() {
    let got = check(CODEOWNERS, &files(&["tooling/x.rs"]), "ann", "h1", &[]);

    assert_eq!(got, Ok(Verdict::NotSecurity));
}

#[test]
fn two_approvals_with_the_security_reviewer_pass() {
    let reviews = [
        review("bob", "APPROVED", "h1"),
        review("sec", "APPROVED", "h1"),
    ];

    let got = check(
        CODEOWNERS,
        &files(&["crates/pin/src/lib.rs"]),
        "ann",
        "h1",
        &reviews,
    );

    assert_eq!(got, Ok(Verdict::Approved));
}

#[test]
fn one_approval_is_short() {
    let reviews = [review("sec", "APPROVED", "h1")];

    let got = check(
        CODEOWNERS,
        &files(&["crates/pin/src/lib.rs"]),
        "ann",
        "h1",
        &reviews,
    );

    assert!(got.is_err_and(|e| e.contains("2 approvals")));
}

#[test]
fn two_approvals_without_the_security_reviewer_are_short() {
    let reviews = [
        review("bob", "APPROVED", "h1"),
        review("cat", "APPROVED", "h1"),
    ];

    let got = check(
        CODEOWNERS,
        &files(&["crates/pin/src/lib.rs"]),
        "ann",
        "h1",
        &reviews,
    );

    assert!(got.is_err_and(|e| e.contains("@sec")));
}

#[test]
fn the_author_s_own_approval_does_not_count() {
    let reviews = [
        review("sec", "APPROVED", "h1"),
        review("ann", "APPROVED", "h1"),
    ];

    let got = check(CODEOWNERS, &files(&["crates/pin/x"]), "Ann", "h1", &reviews);

    assert!(got.is_err());
}

#[test]
fn an_approval_of_an_older_head_does_not_count() {
    let reviews = [
        review("bob", "APPROVED", "h0"),
        review("sec", "APPROVED", "h1"),
    ];

    assert!(check(CODEOWNERS, &files(&["crates/pin/x"]), "ann", "h1", &reviews).is_err());
}

#[test]
fn a_later_review_replaces_an_earlier_one_and_a_comment_changes_nothing() {
    let reviews = [
        review("bob", "APPROVED", "h1"),
        review("sec", "APPROVED", "h1"),
        review("sec", "CHANGES_REQUESTED", "h1"),
        review("bob", "COMMENTED", "h1"),
    ];

    assert!(check(CODEOWNERS, &files(&["crates/pin/x"]), "ann", "h1", &reviews).is_err());
}

#[test]
fn the_same_reviewer_twice_counts_once() {
    let reviews = [
        review("sec", "APPROVED", "h1"),
        review("SEC", "APPROVED", "h1"),
    ];

    assert!(check(CODEOWNERS, &files(&["crates/pin/x"]), "ann", "h1", &reviews).is_err());
}

#[test]
fn parse_reviews_reads_the_forge_s_shape() {
    let json = r#"[{"user":{"login":"bob"},"state":"APPROVED","commit_id":"h1"}]"#;

    assert_eq!(
        parse_reviews(json),
        Ok(vec![review("bob", "APPROVED", "h1")])
    );
    assert!(parse_reviews("{}").is_err());
    assert!(parse_reviews(r#"[{"state":"APPROVED"}]"#).is_err());
}

#[test]
fn parse_files_reads_the_forge_s_shape() {
    let json = r#"[{"filename":"a/b"},{"filename":"c","previous_filename":"d"}]"#;

    assert_eq!(parse_files(json), Ok(files(&["a/b", "c", "d"])));
    assert!(parse_files("[1]").is_err());
}
