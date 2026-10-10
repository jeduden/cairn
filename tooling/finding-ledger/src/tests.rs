use super::*;

const SAMPLE: &str = r#"{"themes": [
  {"id": "T-1", "title": "fixed one", "status": "fixed",
   "texts": [{"file": "docs/a.md", "quote": "the rule  stands"}]},
  {"id": "T-2", "title": "still open", "status": "open"}
]}"#;

fn text(file: &str, quote: &str) -> Text {
    Text {
        file: file.into(),
        quote: quote.into(),
    }
}

fn theme(id: &str, status: &str, texts: Vec<Text>) -> Theme {
    Theme {
        id: id.into(),
        title: id.into(),
        status: status.into(),
        texts,
        ..Theme::default()
    }
}

/// A repository of the given files, read as `read` closures do.
fn files(entries: &[(&str, &str)]) -> impl Fn(&str) -> io::Result<String> {
    let map: BTreeMap<String, String> = entries
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    move |path| {
        map.get(path)
            .cloned()
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
    }
}

#[test]
fn parse_reads_themes() {
    let l = parse(SAMPLE).unwrap();

    assert_eq!(l.themes.len(), 2);
    assert_eq!(l.themes[0].status, FIXED);
    assert_eq!(l.themes[0].texts, [text("docs/a.md", "the rule  stands")]);
    assert_eq!(l.themes[1].status, OPEN);
}

#[test]
fn parse_refuses_what_it_cannot_read() {
    let err = parse(r#"{"themes": ["#).unwrap_err();

    assert!(err.to_string().starts_with("ledger: parse: "), "{err}");
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn validate_accepts_every_well_formed_closure() {
    let x = || vec![text("docs/a.md", "q")];
    let l = Ledger {
        themes: vec![
            theme("T-1", OPEN, vec![]),
            theme("T-2", FIXED, x()),
            Theme {
                oq: "OQ-42".into(),
                milestone: "M7".into(),
                ..theme("T-3", DEFERRED, x())
            },
            Theme {
                risk: "R7".into(),
                ..theme("T-4", ACCEPTED, x())
            },
        ],
    };

    assert!(l.validate().is_empty());
}

#[test]
fn validate_reports_each_malformed_theme() {
    let x = || vec![text("docs/a.md", "q")];
    let cases = [
        (
            Theme {
                title: "nameless".into(),
                status: OPEN.into(),
                ..Theme::default()
            },
            r#"theme "nameless" has no id"#,
        ),
        (
            theme("T-1", "done", vec![]),
            r#"T-1: unknown status "done""#,
        ),
        (
            Theme {
                oq: "OQ-x".into(),
                milestone: "M7".into(),
                ..theme("T-2", DEFERRED, x())
            },
            "T-2: deferred without an OQ-n and an Mn milestone",
        ),
        (
            Theme {
                oq: "OQ-1".into(),
                ..theme("T-3", DEFERRED, x())
            },
            "T-3: deferred without an OQ-n and an Mn milestone",
        ),
        (
            Theme {
                risk: "risk".into(),
                ..theme("T-4", ACCEPTED, x())
            },
            "T-4: accepted without a residual risk Rn",
        ),
        (theme("T-5", FIXED, vec![]), "T-5: fixed but cites no text"),
        (
            theme("T-6", FIXED, vec![text("docs/a.md", "  ")]),
            "T-6: a cited text needs a file and a quote",
        ),
        (
            theme("T-7", FIXED, vec![text("", "q")]),
            "T-7: a cited text needs a file and a quote",
        ),
        (
            theme("T-8", FIXED, vec![text("/etc/passwd", "q")]),
            r#"T-8: cited file "/etc/passwd" is not a path inside the repository"#,
        ),
        (
            theme("T-9", FIXED, vec![text("docs/../../a.md", "q")]),
            r#"T-9: cited file "docs/../../a.md" is not a path inside the repository"#,
        ),
    ];
    for (t, want) in cases {
        assert_eq!(Ledger { themes: vec![t] }.validate(), [want]);
    }
}

#[test]
fn validate_refuses_a_duplicate_id() {
    let t = theme("T-1", FIXED, vec![text("docs/a.md", "q")]);

    assert_eq!(
        Ledger {
            themes: vec![t.clone(), t]
        }
        .validate(),
        ["theme id T-1 used twice"]
    );
}

#[test]
fn numbered_wants_digits_after_the_prefix() {
    assert!(numbered("OQ-42", "OQ-"));
    assert!(numbered("M7", "M"));
    assert!(!numbered("M", "M"));
    assert!(!numbered("M7a", "M"));
    assert!(!numbered("R7", "M"));
}

#[test]
fn inside_repository_refuses_a_root_or_a_parent() {
    assert!(inside_repository("docs/domain-model/places.md"));
    assert!(inside_repository("./docs/a.md"));
    assert!(!inside_repository("/etc/passwd"));
    assert!(!inside_repository("../a.md"));
    assert!(!inside_repository("docs/../../a.md"));
}

#[test]
fn normalize_collapses_whitespace() {
    assert_eq!(normalize("  a\n  b\t\tc \n"), "a b c");
}

#[test]
fn missing_ignores_wrapping_and_open_themes() {
    let read = files(&[("docs/a.md", "text before\nthe rule\n  stands, and more\n")]);
    let l = Ledger {
        themes: vec![
            theme(
                "T-1",
                FIXED,
                vec![
                    text("docs/a.md", "the rule stands"),
                    text("docs/a.md", "the rule is gone"),
                ],
            ),
            theme("T-2", OPEN, vec![text("docs/none.md", "never read")]),
        ],
    };

    assert_eq!(
        l.missing(read).unwrap(),
        ["T-1: docs/a.md: the rule is gone"]
    );
}

#[test]
fn missing_reports_a_file_it_cannot_read() {
    let l = Ledger {
        themes: vec![theme("T-1", FIXED, vec![text("docs/none.md", "q")])],
    };

    let err = l.missing(files(&[])).unwrap_err();

    assert!(
        err.to_string()
            .starts_with("ledger: T-1 cites docs/none.md: "),
        "{err}"
    );
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn load_reads_the_ledger_and_reports_a_missing_file() {
    let read = files(&[("ledger.json", SAMPLE)]);

    assert_eq!(load(&read, "ledger.json").unwrap().themes.len(), 2);

    let err = load(&read, "none.json").unwrap_err();
    assert!(
        err.to_string().starts_with("ledger: read none.json: "),
        "{err}"
    );
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn check_passes_a_carried_ledger_and_names_what_is_not() {
    let l = parse(SAMPLE).unwrap();
    assert!(check(&l, files(&[("docs/a.md", "the rule stands")])).is_ok());

    let err = check(&l, files(&[("docs/a.md", "another rule")])).unwrap_err();
    assert_eq!(
        err.to_string(),
        "closed findings no longer carried in the text:\nT-1: docs/a.md: the rule  stands"
    );
    assert!(std::error::Error::source(&err).is_none());

    assert!(matches!(check(&l, files(&[])), Err(Error::Cited { .. })));

    let bad = Ledger {
        themes: vec![theme("T-1", FIXED, vec![])],
    };
    let err = check(&bad, files(&[])).unwrap_err();
    assert_eq!(
        err.to_string(),
        "ledger: malformed: T-1: fixed but cites no text"
    );
    assert!(std::error::Error::source(&err).is_none());
}
