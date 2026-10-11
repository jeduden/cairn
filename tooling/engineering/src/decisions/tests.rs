use super::*;
use crate::dependencies::tests::dep_adr;
use std::path::PathBuf;

#[test]
fn identity_checks_pass_a_good_record() {
    let good = [dep_adr("ADR-01", adr::ACCEPTED, "x", vec![])];

    assert!(adrs_complete(&good).is_empty());
    assert!(adrs_named_for_id(&good).is_empty());
    assert!(adr_statuses_valid(&good).is_empty());
    assert!(adr_ids_unique(&good).is_empty());
    assert!(superseded_names_successor(&good).is_empty());
}

#[test]
fn identity_checks_name_every_bad_record() {
    let good = dep_adr("ADR-01", adr::ACCEPTED, "x", vec![]);
    let bad = Adr {
        path: PathBuf::from("docs/adr/other.md"),
        id: "7".into(),
        status: "draft".into(),
        ..Adr::default()
    };
    let gone = Adr {
        superseded_by: "ADR-99".into(),
        ..dep_adr("ADR-02", adr::SUPERSEDED, "x", vec![])
    };
    let adrs = [good.clone(), good, bad, gone];

    assert_eq!(
        adrs_complete(&adrs),
        [
            r#"docs/adr/other.md: id "7" is not ADR-<digits>"#,
            "docs/adr/other.md: no title",
            "docs/adr/other.md: no summary",
        ]
    );
    assert_eq!(
        adrs_named_for_id(&adrs),
        ["docs/adr/other.md is not named for its id 7"]
    );
    assert_eq!(
        adr_statuses_valid(&adrs),
        [r#"7: status "draft" is not proposed, accepted or superseded"#]
    );
    assert_eq!(
        adr_ids_unique(&adrs),
        ["docs/adr/ADR-01-x.md: id ADR-01 already used by docs/adr/ADR-01-x.md"]
    );
    assert_eq!(
        superseded_names_successor(&adrs),
        [r#"ADR-02 is superseded by "ADR-99", which is no other ADR"#]
    );
}

#[test]
fn an_id_wants_digits_after_its_prefix() {
    let adrs = [Adr {
        id: "ADR-".into(),
        ..dep_adr("ADR-1", adr::ACCEPTED, "x", vec![])
    }];

    assert_eq!(
        adrs_complete(&adrs),
        [r#"docs/adr/ADR-1-x.md: id "ADR-" is not ADR-<digits>"#]
    );
}

#[test]
fn supersession_names_another_record_and_marks_the_old_one() {
    let own = Adr {
        superseded_by: "ADR-01".into(),
        ..dep_adr("ADR-01", adr::SUPERSEDED, "x", vec![])
    };
    let half = Adr {
        superseded_by: "ADR-03".into(),
        ..dep_adr("ADR-02", adr::ACCEPTED, "x", vec![])
    };
    let replaced = Adr {
        superseded_by: "ADR-03".into(),
        ..dep_adr("ADR-04", adr::SUPERSEDED, "x", vec![])
    };
    let adrs = [
        own,
        half,
        dep_adr("ADR-03", adr::ACCEPTED, "x", vec![]),
        replaced,
    ];

    assert_eq!(
        superseded_names_successor(&adrs),
        [
            r#"ADR-01 is superseded by "ADR-01", which is no other ADR"#,
            r#"ADR-02 names successor ADR-03 but its status is "accepted", not superseded"#,
        ]
    );
}
