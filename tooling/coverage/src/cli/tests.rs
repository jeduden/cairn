use super::*;
use crate::tests::FakeHost;
use std::cell::RefCell;

const METADATA: &str = r#"{
  "workspace_root": "/w",
  "packages": [{"name": "srs", "manifest_path": "/w/srs/Cargo.toml", "targets": []}],
  "metadata": {"coverage": {"floors": {"all": 100}, "pyramid": true}}
}"#;

fn host(report: &str) -> FakeHost {
    FakeHost {
        metadata: METADATA.into(),
        reports: RefCell::new([report.to_owned()].into()),
        sources: vec![
            ("/w/srs/src/tests.rs".into(), "#[test]\n".repeat(3)),
            ("/w/srs/tests/gates.rs".into(), "#[test]\n".repeat(2)),
        ],
        scenarios: 1,
        ..FakeHost::default()
    }
}

fn invoke(args: &[&str], host: &FakeHost, summary: Option<&Path>) -> (u8, String, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run(
        args.iter().map(|a| (*a).to_owned()),
        &mut out,
        &mut err,
        host,
        summary,
        false,
    );
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

#[test]
fn run_prints_the_table_and_writes_each_layer() {
    let host = host("SF:/w/srs/src/lib.rs\nDA:1,1\n");

    let (code, out, err) = invoke(&["--out", "cov"], &host, Some(Path::new("summary.md")));

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(out.contains("| srs | 1 | 100.0% |"), "{out}");
    let files = host.files.borrow();
    assert!(files.contains_key(Path::new("cov/unit.lcov")));
    assert!(
        files[Path::new("summary.md")].starts_with("## Line coverage per test layer\n\n| Crate |")
    );
}

#[test]
fn run_fails_on_a_crate_below_its_floor() {
    let host = host("SF:/w/srs/src/lib.rs\nDA:1,1\nDA:2,0\n");

    let (code, _, err) = invoke(&["--out=cov"], &host, None);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.contains(
            "coverage: srs is at 50.0% of its lines (1/2) in every test layer together, want 100%"
        ),
        "{err}"
    );
    assert!(host.files.borrow().contains_key(Path::new("cov/all.lcov")));
}

#[test]
fn run_fails_when_the_measurement_does() {
    for needle in ["metadata", "--lib", "append"] {
        let host = FakeHost {
            fail_on: Some(needle),
            ..host("")
        };
        let (code, _, err) = invoke(&[], &host, Some(Path::new("summary.md")));
        assert_eq!(code, EXIT_FAILURE, "{needle}");
        assert!(err.starts_with("coverage: "), "{needle}: {err}");
    }

    let host = FakeHost {
        metadata: "not json".into(),
        ..host("")
    };
    assert!(invoke(&[], &host, None).2.contains("read cargo metadata"));
}

#[test]
fn run_explains_its_command_line() {
    let host = host("");

    let (code, out, _) = invoke(&["--help"], &host, None);
    assert_eq!(code, EXIT_OK);
    assert!(out.starts_with("usage: coverage [--out DIR]"));

    for (args, want) in [
        (&["--bogus"][..], r#"unexpected argument "--bogus""#),
        (&["--out"], "--out needs a directory"),
    ] {
        let (code, _, err) = invoke(args, &host, None);
        assert_eq!(code, EXIT_USAGE);
        assert!(err.contains(want), "{err}");
    }
    assert!(
        host.calls.borrow().is_empty(),
        "a wrong command line runs nothing"
    );
}

#[test]
fn run_measures_into_target_coverage_by_default() {
    let host = host("");

    invoke(&[], &host, None);

    assert!(
        host.files
            .borrow()
            .contains_key(Path::new("target/coverage/unit.lcov"))
    );
}

#[test]
fn run_reports_the_pyramid_s_shape() {
    let host = host("SF:/w/srs/src/lib.rs\nDA:1,1\n");

    let (code, out, err) = invoke(&[], &host, None);

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(
        out.contains("Tests per test layer: unit 3, integration 2, end-to-end 1 (1 scenarios)"),
        "{out}"
    );
    assert!(
        host.calls
            .borrow()
            .contains(&"scenarios /w/features".to_owned())
    );
}

#[test]
fn run_fails_an_inverted_pyramid() {
    let host = FakeHost {
        scenarios: 5,
        ..host("SF:/w/srs/src/lib.rs\nDA:1,1\n")
    };

    let (code, _, err) = invoke(&[], &host, None);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.contains("coverage: the test pyramid is inverted: unit 3, integration 2, end-to-end 5"),
        "{err}"
    );
}

#[test]
fn run_fails_when_the_tests_cannot_be_counted() {
    for needle in ["sources", "scenarios"] {
        let host = FakeHost {
            fail_on: Some(needle),
            ..host("SF:/w/srs/src/lib.rs\nDA:1,1\n")
        };
        let (code, _, err) = invoke(&[], &host, None);
        assert_eq!(code, EXIT_FAILURE, "{needle}");
        assert!(err.contains(&format!("{needle} /w")), "{needle}: {err}");
    }
}

#[test]
fn run_refuses_to_measure_inside_a_measurement() {
    let host = host("");
    let (mut out, mut err) = (Vec::new(), Vec::new());

    let code = run(Vec::<String>::new(), &mut out, &mut err, &host, None, true);

    assert_eq!(code, EXIT_USAGE);
    assert!(
        String::from_utf8(err)
            .unwrap()
            .contains("refusing to recurse")
    );
    assert!(host.calls.borrow().is_empty());
}
