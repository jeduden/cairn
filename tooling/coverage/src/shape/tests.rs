use super::*;
use crate::plan::Crate;
use std::path::PathBuf;

fn plan() -> Plan {
    Plan {
        crates: vec![Crate {
            name: "a".into(),
            dir: "/w/a".into(),
        }],
        ..Plan::default()
    }
}

fn source(path: &str, tests: usize) -> (PathBuf, String) {
    (PathBuf::from(path), "#[test]\nfn t() {}\n".repeat(tests))
}

#[test]
fn count_sorts_tests_by_where_they_live() {
    let sources = [
        source("/w/a/src/lib/tests.rs", 3),
        source("/w/a/src/tests.rs", 2),
        source("/w/a/tests/gates.rs", 2),
        source("/w/a/tests/e2e_cli.rs", 1),
        source("/w/a/tests/bdd/main.rs", 4),
        source("/w/a/benches/b.rs", 9),
        source("/w/a/tests", 9),
        source("/elsewhere/src/x.rs", 9),
    ];

    assert_eq!(
        count(&plan(), &sources, 6),
        Shape {
            tests: [5, 2, 11],
            scenarios: 6
        }
    );
}

#[test]
fn the_shape_is_a_pyramid_or_says_why_not() {
    let pyramid = Shape {
        tests: [10, 3, 3],
        scenarios: 2,
    };
    assert!(pyramid.inverted().is_none());
    assert_eq!(
        pyramid.line(),
        "Tests per test layer: unit 10, integration 3, end-to-end 3 (2 scenarios)"
    );

    for tests in [[2, 3, 1], [5, 1, 2]] {
        let msg = Shape {
            tests,
            scenarios: 0,
        }
        .inverted()
        .unwrap();
        assert!(
            msg.starts_with("coverage: the test pyramid is inverted: "),
            "{msg}"
        );
    }
}

#[test]
fn layer_of_reads_the_first_directories() {
    assert_eq!(layer_of(Path::new("src/x.rs")), Some(TestLayer::Unit));
    assert_eq!(layer_of(Path::new("tests/e2e")), Some(TestLayer::EndToEnd));
    assert_eq!(layer_of(Path::new("tests")), None);
    assert_eq!(layer_of(Path::new("")), None);
    assert_eq!(layer_of(Path::new("/abs")), None);
}
