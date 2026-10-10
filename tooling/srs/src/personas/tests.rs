use super::*;

const APPENDIX_C: &str = "# C

| Other | Table |
|---|---|
| x | y |

| Requirements | Personas |
|---|---|
| REC-01, REC-02 | U9, U4 |
| SEC-08 | U8 |
";

fn strings(s: &[&str]) -> Vec<String> {
    s.iter().map(|x| (*x).to_owned()).collect()
}

#[test]
fn persona_coverage_reads_the_table() {
    let got = persona_coverage(APPENDIX_C).unwrap();

    let want: BTreeMap<String, Vec<String>> = [
        ("REC-01".to_owned(), strings(&["U9", "U4"])),
        ("REC-02".to_owned(), strings(&["U9", "U4"])),
        ("SEC-08".to_owned(), strings(&["U8"])),
    ]
    .into();
    assert_eq!(got, want);
}

#[test]
fn persona_coverage_rejects_bad_rows() {
    let head = "| Requirements | Personas |\n|---|---|\n";
    let cases = [
        (
            "| REC-01 | U10 |\n",
            r#"srs: line 3: malformed persona "U10""#,
        ),
        ("| REC-01 | |\n", r#"srs: line 3: malformed persona """#),
        (
            "| REC-01 | U1 |\n| REC-01 | U2 |\n",
            "srs: line 4: REC-01 listed twice",
        ),
        (
            "| rec-1 | U1 |\n",
            r#"srs: line 3: malformed requirement id "rec-1""#,
        ),
        (
            "| REC-01 |\n",
            "srs: line 3: malformed persona coverage row",
        ),
    ];
    for (rows, want) in cases {
        assert_eq!(
            persona_coverage(&format!("{head}{rows}"))
                .unwrap_err()
                .to_string(),
            want
        );
    }
    assert_eq!(
        persona_coverage("# no table\n").unwrap_err().to_string(),
        "srs: no persona coverage table"
    );
}

#[test]
fn split_personas_reads_the_cell() {
    assert_eq!(split_personas(" U9 , U4,U1 ").unwrap(), ["U9", "U4", "U1"]);
    assert_eq!(
        split_personas("U1, X2").unwrap_err(),
        r#"malformed persona "X2""#
    );
    assert_eq!(
        split_personas("U1, U4, U1").unwrap_err(),
        "persona U1 listed twice"
    );
}

#[test]
fn persona_agents_reads_section_2_5() {
    let body = "| # | Persona | Agent | Who |\n|---|---|---|---|\n\
                    | U1 | Platform operator | `persona-platform-engineer` | x |\n\
                    | U9 | Agent | `persona-agent` | y |\n";

    let got = persona_agents(body).unwrap();

    let want: BTreeMap<String, String> = [
        ("U1".into(), "persona-platform-engineer".into()),
        ("U9".into(), "persona-agent".into()),
    ]
    .into();
    assert_eq!(got, want);
}

#[test]
fn persona_agents_rejects_bad_rows() {
    let head = "| # | Persona | Agent |\n|---|---|---|\n";
    let err = |rows: &str| {
        persona_agents(&format!("{head}{rows}"))
            .unwrap_err()
            .to_string()
    };

    assert_eq!(
        err("| X1 | p | `a` |\n"),
        "srs: line 3: malformed persona row"
    );
    assert_eq!(
        err("| U1 | p | a |\n"),
        "srs: line 3: malformed persona row"
    );
    assert_eq!(
        err("| U1 | p | `persona-a` |\n| U1 | p | `persona-a` |\n"),
        "srs: line 4: U1 listed twice"
    );
    assert_eq!(
        err("| U1 | p | `persona-a` |\n| U2 | q | `persona-a` |\n"),
        "srs: line 4: persona-a named twice"
    );
    assert_eq!(
        persona_agents("# none\n").unwrap_err().to_string(),
        "srs: no persona table"
    );
}

#[test]
fn persona_row_reads_one_row() {
    let row = |cells: &[&str], line| Row {
        cells: strings(cells),
        line,
    };

    assert_eq!(
        persona_row(&row(&["U2", "p", "`persona-b`"], 3)).unwrap(),
        ("U2".into(), "persona-b".into())
    );
    for (cells, line) in [
        (&["U2", "p"][..], 4),
        (&["U2", "p", "persona-b"], 5),
        (&["U2", "p", "`persona-`"], 6),
        (&["U2", "p", "`persona-B`"], 7),
        (&["U2", "p", "`agent-b`"], 8),
        (&["U2", "p", "`persona-b"], 9),
    ] {
        assert_eq!(
            persona_row(&row(cells, line)).unwrap_err().to_string(),
            format!("srs: line {line}: malformed persona row")
        );
    }
}
