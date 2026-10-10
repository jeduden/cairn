use super::*;
use testkit::TempDir;

const SAMPLE: &str = "---
id: ADR-2609292234
title: \"The test stack\"
status: 'accepted'
summary: >-
  Scenarios run through godog,
  with testify's assertions.
superseded-by: ADR-2609300000
---
# ADR-2609292234: The test stack

## Context

Why.

## Decision

| Module                      | Purpose | License | Maintenance |
| --------------------------- | ------- | ------- | ----------- |
| `example.com/one`           | tests   | MIT     | active      |
| example.com/two             | tests   | ISC     |

## Alternatives

None worth it.
";

fn a_md() -> &'static Path {
    Path::new("a.md")
}

fn module(name: &str, purpose: &str, license: &str, maintenance: &str) -> Module {
    Module {
        name: name.into(),
        purpose: purpose.into(),
        license: license.into(),
        maintenance: maintenance.into(),
    }
}

#[test]
fn parse_reads_front_matter_sections_and_modules() {
    let a = parse(Path::new("docs/adr/ADR-2609292234-test-stack.md"), SAMPLE).unwrap();

    assert_eq!(a.id, "ADR-2609292234");
    assert_eq!(a.title, "The test stack");
    assert_eq!(a.status, ACCEPTED);
    assert_eq!(
        a.summary,
        "Scenarios run through godog, with testify's assertions."
    );
    assert_eq!(a.superseded_by, "ADR-2609300000");
    assert_eq!(a.sections["Context"], "Why.");
    assert_eq!(a.sections["Alternatives"], "None worth it.");
    assert_eq!(
        a.modules,
        [
            module("example.com/one", "tests", "MIT", "active"),
            module("example.com/two", "tests", "ISC", "")
        ]
    );
}

#[test]
fn parse_without_module_table_has_no_modules() {
    let a = parse(
        a_md(),
        "---\nid: ADR-01\n---\n# ADR-01\n\n## Decision\n\n| A | B |\n| - | - |\n| 1 | 2 |\n",
    )
    .unwrap();

    assert!(a.modules.is_empty());
    assert_eq!(a.sections.keys().collect::<Vec<_>>(), ["Decision"]);
}

#[test]
fn parse_without_decision_has_no_modules() {
    let a = parse(a_md(), "---\nid: ADR-01\n---\n# ADR-01\n").unwrap();

    assert!(a.modules.is_empty());
    assert!(a.sections.is_empty());
}

#[test]
fn parse_refuses_what_it_cannot_read() {
    let cases = [
        ("# ADR-01\n", "no front matter"),
        ("---\nid: ADR-01\n", "no front matter"),
        ("id: ADR-01\n---\n", "no front matter"),
        (
            "---\nmodules:\n  - example.com/one\n---\n",
            r#"line 3: unsupported front matter "  - example.com/one""#,
        ),
        (
            "---\nid ADR-01\n---\n",
            r#"line 2: malformed front matter "id ADR-01""#,
        ),
        ("---\n: x\n---\n", r#"line 2: malformed front matter ": x""#),
    ];
    for (body, want) in cases {
        assert_eq!(
            parse(a_md(), body).unwrap_err().to_string(),
            format!("adr: a.md: {want}"),
            "{body}"
        );
    }
}

#[test]
fn parse_skips_blank_lines_and_keeps_quotes_it_cannot_pair() {
    let a = parse(a_md(), "---\nid: ADR-01\n\ntitle: \"half\n---\n").unwrap();

    assert_eq!(a.id, "ADR-01");
    assert_eq!(a.title, "\"half");
}

#[test]
fn parse_joins_a_literal_block_scalar() {
    let a = parse(
        a_md(),
        "---\nsummary: |\n  one\n\n  two\nstatus: proposed\n---\n",
    )
    .unwrap();

    assert_eq!(a.summary, "one two");
    assert_eq!(a.status, PROPOSED);
}

#[test]
fn parse_reads_modules_only_from_the_decision() {
    let body = "---\nid: ADR-01\n---\n# ADR-01\n\n## Decision\n\n| Module | Purpose |\n| - | - |\n| `a` | p |\n\n\
                    ## Alternatives\n\n| Module | Why not |\n| - | - |\n| `b` | slow |\n";

    assert_eq!(
        parse(a_md(), body).unwrap().modules,
        [module("a", "p", "", "")]
    );
}

#[test]
fn parse_keeps_fenced_headings_inside_their_section() {
    let body =
        "---\nid: ADR-01\n---\n## Alternatives\n\nOne.\n\n```md\n## Not a section\n```\n\nTwo.\n";

    let a = parse(a_md(), body).unwrap();

    assert_eq!(
        a.sections["Alternatives"],
        "One.\n\n```md\n## Not a section\n```\n\nTwo."
    );
    assert!(!a.sections.contains_key("Not a section"));
}

#[test]
fn parse_reads_crlf_line_endings() {
    let a = parse(
        a_md(),
        "---\r\nid: ADR-01\r\nstatus: accepted\r\n---\r\n## Context\r\n\r\nWhy.\r\n",
    )
    .unwrap();

    assert_eq!(a.id, "ADR-01");
    assert_eq!(a.status, ACCEPTED);
    assert_eq!(a.sections["Context"], "Why.");
}

#[test]
fn unquote_strips_one_matching_pair() {
    assert_eq!(unquote("\"a\""), "a");
    assert_eq!(unquote("'a'"), "a");
    assert_eq!(unquote("\"a'"), "\"a'");
    assert_eq!(unquote("\""), "\"");
    assert_eq!(unquote("a"), "a");
}

#[test]
fn load_reads_every_file_in_name_order() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("ADR-02-b.md"), "---\nid: ADR-02\n---\n").unwrap();
    fs::write(dir.path().join("ADR-01-a.md"), "---\nid: ADR-01\n---\n").unwrap();
    fs::write(dir.path().join("notes.txt"), "not a record").unwrap();

    let adrs = load(dir.path()).unwrap();

    assert_eq!(adrs.len(), 2);
    assert_eq!(adrs[0].id, "ADR-01");
    assert_eq!(adrs[1].path, dir.path().join("ADR-02-b.md"));
}

#[test]
fn load_reports_parse_read_and_list_errors() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("bad.md"), "# no front matter\n").unwrap();
    let err = load(dir.path()).unwrap_err();
    assert!(
        err.to_string().ends_with("bad.md: no front matter"),
        "{err}"
    );
    assert!(std::error::Error::source(&err).is_none());

    let unreadable = TempDir::new().unwrap();
    fs::create_dir(unreadable.path().join("dir.md")).unwrap();
    let err = load(unreadable.path()).unwrap_err();
    assert!(err.to_string().starts_with("adr: read "), "{err}");
    assert!(std::error::Error::source(&err).is_some());

    let err = load(&unreadable.path().join("missing")).unwrap_err();
    assert!(err.to_string().starts_with("adr: list "), "{err}");
    assert!(std::error::Error::source(&err).is_some());
}
