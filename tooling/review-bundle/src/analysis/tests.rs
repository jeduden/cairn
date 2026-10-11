use super::*;
use std::path::PathBuf;

fn req(id: &str, text: &str) -> srs::Requirement {
    srs::Requirement {
        id: id.into(),
        priority: "P0".into(),
        verify: "T".into(),
        traces: vec!["I1".into()],
        text: text.into(),
        path: "docs/srs/05.md".into(),
        line: 3,
    }
}

fn sc(id: &str, pending: bool) -> scenario::Scenario {
    scenario::Scenario {
        path: PathBuf::from("features/record.feature"),
        line: 7,
        name: format!("{id} works"),
        id: id.into(),
        priority: "P0".into(),
        invariants: vec!["I1".into()],
        pending,
        steps: vec!["a step".into()],
    }
}

const ADR: &str = "docs/adr/ADR-2610111200-invariant.md";

fn record(status: &str, body: &str) -> String {
    format!(
        "---\nid: ADR-2610111200\ntitle: \"t\"\nstatus: {status}\nsummary: >-\n  s\n---\n# ADR-2610111200: t\n\n{body}\n"
    )
}

fn base() -> Snapshot {
    Snapshot {
        requirements: vec![
            req("REC-01", "Ingest."),
            req("REC-02", "Link."),
            req("PRV-10", "Certify."),
            req(
                "ENG-29",
                "A change to an invariant, or to a requirement marked for I2 review (PRV-10, PRV-11), MUST land with an ADR.",
            ),
        ],
        scenarios: vec![sc("REC-01", false), sc("REC-02", true), sc("PRV-10", true)],
        codeowners: "/docs/srs/ @owner\n/tooling/ @owner\n".into(),
        invariants: "I1".into(),
        decisions: BTreeMap::new(),
    }
}

fn header(path: &str) -> String {
    format!("diff --git a/{path} b/{path}\n")
}

#[test]
fn class_names_are_the_ones_the_workflow_reads() {
    assert_eq!(
        [Class::Light, Class::Standard, Class::Deep].map(Class::name),
        ["light", "standard", "deep"]
    );
}

#[test]
fn analyse_routes_by_the_paths_that_changed() {
    let cases = [
        (vec!["plan/x/plan.md", "README.md"], Class::Light),
        (
            vec!["plan/x/plan.md", "crates/cairn/src/lib.rs"],
            Class::Standard,
        ),
        (vec!["features/record.feature"], Class::Standard),
        (vec!["README.md", "docs/srs/05.md"], Class::Deep),
        (vec!["tooling/drift/src/cases.rs"], Class::Deep),
    ];
    for (paths, want) in cases {
        let diff: String = paths.iter().map(|p| header(p)).collect();
        let got = analyse(&base(), &base(), &diff);
        assert_eq!(got.class, want, "{paths:?}");
        assert_eq!(got.class == Class::Deep, !got.owned.is_empty(), "{paths:?}");
    }
}

#[test]
fn analyse_finds_nothing_in_an_unchanged_tree() {
    let got = analyse(&base(), &base(), &header("README.md"));

    assert!(got.requirements.is_empty());
    assert!(got.scenarios.is_empty());
    assert!(got.touched.is_empty());
    assert!(got.findings.is_empty());
    assert_eq!(got.paths, ["README.md"]);
}

#[test]
fn analyse_reads_added_removed_and_changed_rows() {
    let mut head = base();
    head.requirements[0].text = "Ingest every transcript.".into();
    head.requirements[1].line = 40;
    head.requirements.remove(2);
    head.requirements.push(req("REC-03", "Index."));

    let got = analyse(&base(), &head, "");

    assert_eq!(
        got.requirements,
        BTreeMap::from([
            ("PRV-10".to_owned(), Change::Removed),
            ("REC-01".to_owned(), Change::Changed),
            ("REC-03".to_owned(), Change::Added),
        ])
    );
}

#[test]
fn analyse_reads_scenario_changes_and_mentions_into_the_touched_ids() {
    let mut head = base();
    head.scenarios[1].steps.push("another".into());
    let diff = format!(
        "{}+see REC-01 and NFR-99\n",
        header("crates/cairn/src/lib.rs")
    );

    let got = analyse(&base(), &head, &diff);

    assert_eq!(
        got.scenarios,
        BTreeMap::from([("REC-02".to_owned(), Change::Changed)])
    );
    assert_eq!(got.touched, ["REC-01", "REC-02"]);
}

#[test]
fn analyse_refuses_a_live_scenario_removed_or_pended_again() {
    let mut head = base();
    head.scenarios[0].pending = true;
    let pended = analyse(&base(), &head, "");
    head.scenarios.remove(0);
    let removed = analyse(&base(), &head, "");

    assert_eq!(
        pended.findings,
        [Finding {
            path: "features/record.feature".into(),
            line: 7,
            severity: BLOCKING,
            body: "the REC-01 scenario is live on main and this change puts it back on @pending"
                .into(),
        }]
    );
    assert_eq!(removed.findings.len(), 1);
    assert!(
        removed.findings[0]
            .body
            .contains("REC-01 scenario is live on main and this change removes it"),
        "{:?}",
        removed.findings
    );
}

#[test]
fn analyse_lets_a_pending_scenario_go_live_or_go() {
    let mut head = base();
    head.scenarios[1].pending = false;
    head.scenarios.remove(2);

    assert!(analyse(&base(), &head, "").findings.is_empty());
}

#[test]
fn analyse_asks_eng_29_s_record_for_an_invariant_change() {
    let got = analyse(&base(), &base(), &header(INVARIANTS));

    assert_eq!(got.findings.len(), 1);
    assert_eq!(got.findings[0].path, INVARIANTS);
    assert!(
        got.findings[0]
            .body
            .contains("touches an invariant but changes no accepted ADR"),
        "{:?}",
        got.findings
    );
}

#[test]
fn analyse_asks_eng_29_s_record_for_a_row_marked_for_i2_review() {
    let mut head = base();
    head.requirements[2].text = "Certify, now differently.".into();
    let mut other = base();
    other.requirements[0].text = "Changed, but not marked.".into();

    let got = analyse(&base(), &head, &header("docs/srs/05.md"));

    assert!(
        got.findings[0].body.contains("touches PRV-10 but"),
        "{:?}",
        got.findings
    );
    assert!(
        analyse(&base(), &other, &header("docs/srs/05.md"))
            .findings
            .is_empty()
    );
}

#[test]
fn analyse_takes_an_accepted_record_naming_the_security_reviewer() {
    let diff = format!("{}{}", header(INVARIANTS), header(ADR));
    let with = |body: String| {
        let mut head = base();
        head.decisions.insert(ADR.into(), body);
        analyse(&base(), &head, &diff).findings.len()
    };

    assert_eq!(
        with(record("accepted", "Approved by the Security Reviewer.")),
        0
    );
    assert_eq!(
        with(record("proposed", "Approved by the security reviewer.")),
        1
    );
    assert_eq!(with(record("accepted", "Approved.")), 1);
    assert_eq!(with("no front matter".into()), 1);
}

#[test]
fn analyse_takes_only_a_record_the_change_touches() {
    let mut head = base();
    head.decisions.insert(
        ADR.into(),
        record("accepted", "the security reviewer approved"),
    );

    assert_eq!(
        analyse(&base(), &head, &header(INVARIANTS)).findings.len(),
        1
    );
}

#[test]
fn marked_for_i2_review_reads_eng_29_s_row() {
    assert_eq!(marked_for_i2_review(&base()), ["PRV-10", "PRV-11"]);
    let mut without = base();
    without.requirements.pop();
    assert!(marked_for_i2_review(&without).is_empty());
}
