//! The decision records under docs/adr, as the repository holds them.

/// Every file under docs/adr reads cleanly.
#[test]
fn decision_records_parse() {
    let adrs = adr::load(&testkit::repo_root().join("docs/adr")).unwrap();

    assert!(!adrs.is_empty());
}
