use super::*;

fn req(id: &str, priority: &str, traces: &[&str]) -> srs::Requirement {
    srs::Requirement {
        id: id.into(),
        priority: priority.into(),
        traces: traces.iter().map(|t| (*t).to_owned()).collect(),
        ..srs::Requirement::default()
    }
}

fn sc(id: &str, priority: &str, invariants: &[&str], path: &str, line: usize) -> Scenario {
    Scenario {
        id: id.into(),
        priority: priority.into(),
        invariants: invariants.iter().map(|t| (*t).to_owned()).collect(),
        path: path.into(),
        line,
        ..Scenario::default()
    }
}

#[test]
fn check_is_empty_when_everything_agrees() {
    let reqs = [req("REC-03", "P0", &["I10", "I1"]), req("NFR-01", "", &[])];
    let scenarios = [
        sc("REC-03", "P0", &["I1", "I10"], "", 0),
        sc("NFR-01", "", &[], "", 0),
    ];

    assert!(check(&reqs, &scenarios).is_empty());
}

#[test]
fn check_reports_every_disagreement_sorted() {
    let reqs = [
        req("REC-01", "P0", &["I1"]),
        req("REC-02", "P0", &["I1"]),
        req("REC-03", "P1", &[]),
    ];
    let scenarios = [
        sc("REC-01", "P1", &["I1"], "a", 1),
        sc("REC-01", "", &[], "b", 2),
        sc("REC-03", "P1", &["I2"], "", 0),
        sc("SEC-99", "", &[], "c", 3),
    ];

    assert_eq!(
        check(&reqs, &scenarios),
        [
            r#"REC-01: scenario priority "P1", requirement says "P0""#,
            "REC-01: tagged on a:1 and b:2",
            "REC-02: no scenario under features/",
            "REC-03: scenario invariants [I2], requirement traces []",
            "SEC-99: tagged on c:3 but no SRS requirement has that id",
        ]
    );
}
