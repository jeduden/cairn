use super::*;

const APPENDIX_B: &str = "# B

| Other | Table |
|---|---|
| x | y |

| Invariant | Requirements |
|---|---|
| **I1** — Nothing is lost | REC-01, SEC-08 |
| **I10** — Rebuildable | REC-03 |

| Priority | Functional | Engineering |
|---|---|---|
| P0 | 3 | 1 |
| P1 | 0 | 0 |
| — | Non-functional: 14 | |
";

fn map(entries: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
    entries
        .iter()
        .map(|(k, v)| ((*k).to_owned(), v.iter().map(|s| (*s).to_owned()).collect()))
        .collect()
}

fn counts(functional: &[(&str, usize)], engineering: &[(&str, usize)]) -> Counts {
    let m = |e: &[(&str, usize)]| e.iter().map(|(k, v)| ((*k).to_owned(), *v)).collect();
    Counts {
        functional: m(functional),
        engineering: m(engineering),
    }
}

fn req(id: &str, priority: &str, traces: &[&str]) -> Requirement {
    Requirement {
        id: id.into(),
        priority: priority.into(),
        traces: traces.iter().map(|t| (*t).to_owned()).collect(),
        ..Requirement::default()
    }
}

#[test]
fn invariant_coverage_reads_the_table() {
    let got = invariant_coverage(APPENDIX_B).unwrap();

    assert_eq!(
        got,
        map(&[("I1", &["REC-01", "SEC-08"]), ("I10", &["REC-03"])])
    );
}

#[test]
fn invariant_coverage_rejects_malformed_rows() {
    let err = |body: &str| invariant_coverage(body).unwrap_err().to_string();
    let head = "| Invariant | Requirements |\n|---|---|\n";

    assert_eq!(
        err(&format!("{head}| I1 | REC-01 |\n")),
        "srs: line 3: malformed invariant row"
    );
    assert_eq!(
        err(&format!("{head}| **I11** | REC-01 |\n")),
        "srs: line 3: malformed invariant row"
    );
    assert_eq!(
        err(&format!("{head}| **I1 | REC-01 |\n")),
        "srs: line 3: malformed invariant row"
    );
    assert_eq!(
        err(&format!("{head}| **I1** |\n")),
        "srs: line 3: malformed invariant row"
    );
    assert_eq!(
        err(&format!("{head}| **I1** | REC-1 |\n")),
        r#"srs: line 3: malformed requirement id "REC-1""#
    );
    assert_eq!(err("no table"), "srs: no invariant coverage table");
}

#[test]
fn stated_counts_reads_priority_rows() {
    assert_eq!(
        stated_counts(APPENDIX_B).unwrap(),
        counts(&[("P0", 3)], &[("P0", 1)])
    );
}

#[test]
fn stated_counts_rejects_malformed_rows() {
    let err = |body: &str| stated_counts(body).unwrap_err().to_string();
    let head = "| Priority | F | E |\n|---|---|---|\n";

    assert!(
        err(&format!("{head}| P0 | x | 1 |\n"))
            .starts_with("srs: line 3: malformed count row: \"x\"")
    );
    assert!(
        err(&format!("{head}| P0 | 1 | y |\n"))
            .starts_with("srs: line 3: malformed count row: \"y\"")
    );
    assert_eq!(
        err(&format!("{head}| P0 | 1 |\n")),
        "srs: line 3: malformed count row"
    );
    assert_eq!(err("none"), "srs: no requirement count table");
}

#[test]
fn traced_coverage_and_counts_skip_out_of_scope_families() {
    let reqs = [
        req("REC-01", "P0", &["I1"]),
        req("SEC-08", "P0", &["I1"]),
        req("REC-03", "P0", &["I10"]),
        req("ENG-01", "P0", &["I1"]),
        req("ASM-01", "", &["I2"]),
        req("NFR-01", "", &[]),
    ];

    assert_eq!(
        traced_coverage(&reqs),
        map(&[("I1", &["REC-01", "SEC-08"]), ("I10", &["REC-03"])])
    );
    assert_eq!(priority_counts(&reqs), counts(&[("P0", 3)], &[("P0", 1)]));
}
