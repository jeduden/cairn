use super::*;

const REVIEWED: &str = "206b0a76a81abb510e53d87ef187e83b5aacbcbb";
const MOVED: &str = "490efa734617788d1dc1f831b26d0e23888e90fa";

fn approve() -> ReviewOutcome {
    ReviewOutcome {
        verdict: APPROVE_VERDICT.into(),
        summary: "Looks right.".into(),
        findings: vec![],
    }
}

fn run(name: &str, status: &str, conclusion: &str) -> CheckRun {
    CheckRun {
        name: name.into(),
        status: status.into(),
        conclusion: conclusion.into(),
        ..CheckRun::default()
    }
}

fn at(r: CheckRun, started_at: &str, id: i64) -> CheckRun {
    CheckRun {
        started_at: started_at.into(),
        id,
        ..r
    }
}

fn green() -> Vec<CheckRun> {
    vec![
        run("CI", "completed", "success"),
        run("test", "completed", "success"),
    ]
}

fn input(outcome: ReviewOutcome, check_runs: Vec<CheckRun>) -> Input {
    Input {
        outcome,
        reviewed: REVIEWED.into(),
        head: REVIEWED.into(),
        check_runs,
    }
}

fn finding(path: &str, line: i64, severity: &str, body: &str) -> Finding {
    Finding {
        path: path.into(),
        line,
        severity: severity.into(),
        body: body.into(),
    }
}

#[test]
fn parse_outcome_reads_the_agents_output() {
    let got = parse_outcome(
            r#"{"verdict":"request_changes","summary":"ENG-28 has no drift case.",
            "findings":[{"path":"tooling/drift/src/cases.rs","line":12,"severity":"blocking","body":"add one"}]}"#,
        )
        .unwrap();

    assert_eq!(
        got,
        ReviewOutcome {
            verdict: REQUEST_CHANGES_VERDICT.into(),
            summary: "ENG-28 has no drift case.".into(),
            findings: vec![finding(
                "tooling/drift/src/cases.rs",
                12,
                BLOCKING,
                "add one"
            )],
        }
    );
    assert!(
        parse_outcome(r#"{"verdict":"approve","summary":"s","findings":null}"#)
            .unwrap()
            .findings
            .is_empty()
    );
}

#[test]
fn parse_outcome_refuses_what_the_schema_does_not() {
    let cases = [
        ("not JSON", "approve", "expected value"),
        (
            "an unknown field",
            r#"{"verdict":"approve","summary":"s","findings":[],"approve":true}"#,
            "unknown field",
        ),
        (
            "an unknown finding field",
            r#"{"verdict":"approve","summary":"s","findings":[{"x":1}]}"#,
            "unknown field",
        ),
        (
            "trailing data",
            r#"{"verdict":"approve","summary":"s","findings":[]} {}"#,
            "trailing characters",
        ),
        (
            "an unknown answer",
            r#"{"verdict":"lgtm","summary":"s","findings":[]}"#,
            r#"verdict "lgtm" is neither"#,
        ),
        (
            "no summary",
            r#"{"verdict":"approve","summary":" ","findings":[]}"#,
            "no summary",
        ),
        (
            "an unknown severity",
            r#"{"verdict":"approve","summary":"s","findings":[{"path":"a","line":1,"severity":"minor","body":"b"}]}"#,
            r#"finding 0: severity "minor""#,
        ),
        (
            "a finding with no path",
            r#"{"verdict":"approve","summary":"s","findings":[{"path":"","line":1,"severity":"nit","body":"b"}]}"#,
            "finding 0 has no path or no body",
        ),
        (
            "a finding with no body",
            r#"{"verdict":"approve","summary":"s","findings":[{"path":"a","line":1,"severity":"nit","body":" "}]}"#,
            "finding 0 has no path or no body",
        ),
        (
            "a negative line",
            r#"{"verdict":"approve","summary":"s","findings":[{"path":"a","line":-1,"severity":"nit","body":"b"}]}"#,
            "finding 0: line -1",
        ),
    ];
    for (name, body, want) in cases {
        let err = parse_outcome(body).unwrap_err();
        assert!(matches!(err, Error::Outcome(_)), "{name}");
        assert!(
            err.to_string()
                .starts_with("review: malformed review outcome: "),
            "{name}: {err}"
        );
        assert!(err.to_string().contains(want), "{name}: {err}");
        assert!(std::error::Error::source(&err).is_none());
    }
}

#[test]
fn parse_check_runs_reads_one_per_line() {
    let got = parse_check_runs(
        "{\"name\":\"CI\",\"status\":\"completed\",\"conclusion\":\"success\",\"extra\":1}\n\n\
             {\"name\":\"review\",\"status\":\"in_progress\",\"conclusion\":null}\n",
    )
    .unwrap();

    assert_eq!(
        got,
        [
            run("CI", "completed", "success"),
            run("review", "in_progress", "")
        ]
    );
}

#[test]
fn parse_check_runs_refuses_a_line_that_is_no_check_run() {
    let err = parse_check_runs("{\"name\":\"CI\"}\nnot json\n").unwrap_err();

    assert!(
        err.to_string().starts_with("review: check run line 2: "),
        "{err}"
    );
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn decide_skips_a_head_that_moved_past_the_review() {
    let got = decide(&Input {
        head: MOVED.into(),
        ..input(approve(), green())
    })
    .unwrap();

    assert!(got.is_none());
}

#[test]
fn decide_approves_only_an_approving_outcome_on_a_green_head() {
    let r = decide(&input(approve(), green())).unwrap().unwrap();

    assert_eq!(r.event, APPROVE);
    assert_eq!(r.commit_id, REVIEWED);
    assert!(r.body.contains("Looks right."), "{}", r.body);
    assert!(
        r.body
            .starts_with("**Approved** by the review agent on `206b0a76a81a`."),
        "{}",
        r.body
    );
}

#[test]
fn decide_refuses_a_head_whose_ci_did_not_pass() {
    let cases = [
        ("no CI check", vec![run("lint", "completed", "success")]),
        ("CI failed", vec![run("CI", "completed", "failure")]),
        ("CI running", vec![run("CI", "in_progress", "")]),
    ];
    for (name, runs) in cases {
        let err = decide(&input(approve(), runs)).unwrap_err();
        assert_eq!(
            err.to_string(),
            format!("review: CI has not passed on the reviewed head: {REVIEWED}"),
            "{name}"
        );
        assert!(std::error::Error::source(&err).is_none());
    }
}

#[test]
fn decide_requests_changes_otherwise() {
    let mut failed = green();
    failed.push(run("CodeQL", "completed", "failure"));
    let blocking = ReviewOutcome {
        findings: vec![finding("a.rs", 3, BLOCKING, "breaks I4")],
        ..approve()
    };
    let changes = ReviewOutcome {
        verdict: REQUEST_CHANGES_VERDICT.into(),
        summary: "No scenario.".into(),
        findings: vec![],
    };
    let cases = [
        (
            "the agent requests changes",
            input(changes, green()),
            "No scenario.",
        ),
        (
            "another check failed",
            input(approve(), failed),
            "- check `CodeQL`: failure",
        ),
        (
            "an approval with a blocking finding",
            input(blocking, green()),
            "`a.rs:3`: breaks I4",
        ),
    ];
    for (name, input, want) in cases {
        let r = decide(&input).unwrap().unwrap();
        assert_eq!(r.event, REQUEST_CHANGES, "{name}");
        assert!(
            r.body.starts_with("**Changes requested**"),
            "{name}: {}",
            r.body
        );
        assert!(r.body.contains(want), "{name}: {}", r.body);
    }
}

#[test]
fn decide_ignores_checks_still_running_and_passing_ones() {
    let mut runs = green();
    runs.extend([
        run("review", "in_progress", ""),
        run("codecov/patch", "completed", "neutral"),
        run("release", "completed", "skipped"),
    ]);

    assert_eq!(
        decide(&input(approve(), runs)).unwrap().unwrap().event,
        APPROVE
    );
}

#[test]
fn decide_judges_each_check_by_its_latest_run() {
    // A second CI run on one commit cancels the first; the first
    // run's cancelled jobs and its red gate job must not count.
    let runs = vec![
        at(run("CI", "completed", "failure"), "2026-09-30T21:38:10Z", 0),
        at(
            run("test", "completed", "cancelled"),
            "2026-09-30T21:38:09Z",
            0,
        ),
        at(run("CI", "completed", "success"), "2026-09-30T21:40:01Z", 0),
        at(
            run("test", "completed", "success"),
            "2026-09-30T21:39:10Z",
            0,
        ),
    ];

    let r = decide(&input(approve(), runs)).unwrap().unwrap();

    assert_eq!(r.event, APPROVE, "{}", r.body);
}

#[test]
fn decide_still_refuses_when_the_latest_ci_run_failed() {
    let runs = vec![
        at(run("CI", "completed", "success"), "2026-09-30T21:38:10Z", 0),
        at(run("CI", "completed", "failure"), "2026-09-30T21:40:01Z", 0),
    ];

    assert!(matches!(decide(&input(approve(), runs)), Err(Error::Ci(_))));
}

#[test]
fn latest_keeps_the_newest_run_of_each_check_in_first_seen_order() {
    let runs = [
        at(run("b", "", "failure"), "2026-09-30T21:00:00Z", 0),
        at(run("a", "", "success"), "2026-09-30T21:00:00Z", 0),
        at(run("b", "", "success"), "2026-09-30T21:05:00Z", 0),
        at(run("b", "", "cancelled"), "2026-09-30T21:01:00Z", 0),
    ];

    assert_eq!(latest(&runs), [&runs[2], &runs[1]]);
}

#[test]
fn latest_breaks_a_same_second_tie_by_the_higher_run_id() {
    // started_at has one-second resolution, so two runs can tie;
    // GitHub numbers check runs in creation order.
    let t = "2026-09-30T21:00:00Z";
    let runs = [
        at(run("CI", "", "success"), t, 7),
        at(run("CI", "", "failure"), t, 9),
        at(run("CI", "", "cancelled"), t, 8),
    ];

    assert_eq!(latest(&runs), [&runs[1]]);
}

#[test]
fn newer_orders_by_start_then_by_id() {
    let early = at(CheckRun::default(), "2026-09-30T21:00:00Z", 9);
    let late = at(CheckRun::default(), "2026-09-30T21:00:01Z", 1);

    assert!(newer(&late, &early), "a later start wins over a higher id");
    assert!(!newer(&early, &late));
    assert!(newer(
        &at(CheckRun::default(), "2026-09-30T21:00:00Z", 10),
        &early
    ));
    assert!(!newer(&early, &early));
}

#[test]
fn render_lists_blocking_findings_before_nits() {
    let outcome = ReviewOutcome {
        verdict: REQUEST_CHANGES_VERDICT.into(),
        summary: "Two things.".into(),
        findings: vec![
            finding("b.rs", 0, NIT, "rename"),
            finding("a.rs", 7, BLOCKING, "unwrapped error"),
        ],
    };

    let body = render(&outcome, &[], REQUEST_CHANGES, REVIEWED);

    let (blocking, nit) = (
        body.find("unwrapped error").unwrap(),
        body.find("rename").unwrap(),
    );
    assert!(blocking < nit, "{body}");
    assert!(body.contains("- nit `b.rs`: rename"), "{body}");
    assert!(body.contains(&REVIEWED[..12]), "{body}");
    assert!(render(&outcome, &[], REQUEST_CHANGES, "abc").contains("on `abc`."));
}

#[test]
fn location_names_the_line_only_when_there_is_one() {
    assert_eq!(location(&finding("a.rs", 0, NIT, "b")), "a.rs");
    assert_eq!(location(&finding("a.rs", 4, NIT, "b")), "a.rs:4");
}

#[test]
fn failed_counts_only_completed_failures() {
    assert!(!failed(&run("a", "completed", "success")));
    assert!(failed(&run("b", "completed", "cancelled")));
    assert!(!failed(&run("c", "queued", "")));
}

#[test]
fn passed_ci_wants_the_required_check_completed_green() {
    assert!(passed_ci(&run("CI", "completed", "success")));
    assert!(!passed_ci(&run("ci", "completed", "success")));
}

#[test]
fn is_blocking_is_the_blocking_severity() {
    assert!(is_blocking(&finding("a", 0, BLOCKING, "b")));
    assert!(!is_blocking(&finding("a", 0, NIT, "b")));
}

#[test]
fn defuse_mentions_wraps_bare_at_words_in_code() {
    let cases = [
        (
            "@LANE-16 is retagged from @P2 @I2",
            "`@LANE-16` is retagged from `@P2` `@I2`",
        ),
        ("(I4 and @I10 added)", "(I4 and `@I10` added)"),
        ("tags `@P1 @I2` stay", "tags `@P1 @I2` stay"),
        ("mail a@b.com", "mail a@b.com"),
        ("a lone @ sign", "a lone @ sign"),
        ("ends with @", "ends with @"),
        ("path `x`: @I4.", "path `x`: `@I4`."),
        ("@I4", "`@I4`"),
    ];
    for (input, want) in cases {
        assert_eq!(defuse_mentions(input), want, "{input}");
    }
}

#[test]
fn the_body_mentions_no_one() {
    let outcome = ReviewOutcome {
        verdict: REQUEST_CHANGES_VERDICT.into(),
        summary: "@I2 widens.".into(),
        findings: vec![finding(
            "features/lane.feature",
            0,
            BLOCKING,
            "@LANE-16 is retagged @P1 @I4",
        )],
    };

    let body = render(&outcome, &[], REQUEST_CHANGES, REVIEWED);

    assert!(body.contains("`@I2` widens."), "{body}");
    assert!(
        body.contains("`@LANE-16` is retagged `@P1` `@I4`"),
        "{body}"
    );
}
