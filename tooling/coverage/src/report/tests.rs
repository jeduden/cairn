use super::*;
use crate::lcov::parse;
use crate::plan::{Crate, Floors};
use std::collections::BTreeMap;

fn plan() -> Plan {
    Plan {
        crates: vec![
            Crate {
                name: "srs".into(),
                dir: "/w/srs".into(),
            },
            Crate {
                name: "gate".into(),
                dir: "/w/gate".into(),
            },
        ],
        floors: Floors {
            all: Some(100.0),
            unit: Some(90.0),
            ..Floors::default()
        },
        crate_floors: BTreeMap::from([(
            "gate".to_owned(),
            Floors {
                unit: Some(50.0),
                ..Floors::default()
            },
        )]),
        entry_points: vec!["src/main.rs".into()],
        ..Plan::default()
    }
}

fn layers() -> Vec<(TestLayer, Lines)> {
    vec![
        (
            TestLayer::Unit,
            parse("SF:/w/srs/src/lib.rs\nDA:1,1\nDA:2,1\nDA:3,0\nSF:/w/gate/src/main.rs\nDA:1,0\n")
                .unwrap(),
        ),
        (
            TestLayer::Integration,
            parse("SF:/w/srs/src/lib.rs\nDA:3,4\n").unwrap(),
        ),
        (
            TestLayer::EndToEnd,
            parse("SF:/w/gate/src/main.rs\nDA:1,1\nSF:/elsewhere/x.rs\nDA:1,1\n").unwrap(),
        ),
    ]
}

fn rows() -> Vec<Row> {
    let layers = layers();
    let all = crate::lcov::merge(layers.iter().map(|(_, l)| l));
    tally(&plan(), &layers, &all)
}

#[test]
fn tally_counts_each_crate_s_lines_per_layer() {
    assert_eq!(
        rows(),
        [
            Row {
                name: "gate".into(),
                lines: 1,
                own_lines: 0,
                by_layer: [0, 0, 1],
                all: 1
            },
            Row {
                name: "srs".into(),
                lines: 3,
                own_lines: 3,
                by_layer: [2, 1, 0],
                all: 3
            },
        ]
    );
}

#[test]
fn the_table_has_a_row_per_crate_and_one_for_the_workspace() {
    let table = table(&rows());

    assert!(
        table.starts_with("| Crate | Lines | Unit | Integration | End-to-end | All |\n"),
        "{table}"
    );
    assert!(
        table.contains("| srs | 3 | 66.7% | 33.3% | 0.0% | 100.0% |"),
        "{table}"
    );
    assert!(
        table.contains("| **workspace** | 4 | 66.7% | 33.3% | 25.0% | 100.0% |"),
        "{table}"
    );
}

#[test]
fn shortfalls_name_each_floor_a_crate_misses() {
    assert_eq!(
        shortfalls(&plan(), &rows()),
        ["coverage: srs is at 66.7% of its lines (2/3) in the unit test layer, want 90%",]
    );

    let bare = Row {
        name: "x".into(),
        lines: 4,
        own_lines: 4,
        by_layer: [4, 0, 0],
        all: 3,
    };
    assert_eq!(
        shortfalls(&plan(), &[bare]),
        ["coverage: x is at 75.0% of its lines (3/4) in every test layer together, want 100%"]
    );
}

#[test]
fn no_lines_count_as_fully_covered() {
    assert!((percent(0, 0) - 100.0).abs() < f64::EPSILON);
    assert!((percent(1, 4) - 25.0).abs() < f64::EPSILON);
    assert!((ratio(usize::MAX) - f64::from(u32::MAX)).abs() < f64::EPSILON);
}

#[test]
fn an_entry_point_counts_only_for_the_end_to_end_layer() {
    let gate = &rows()[0];

    assert_eq!(
        (
            gate.measured(TestLayer::Unit),
            gate.measured(TestLayer::EndToEnd)
        ),
        (0, 1)
    );
    assert!(table(&rows()).contains("| gate | 1 | 100.0% | 100.0% | 100.0% | 100.0% |"));
}
