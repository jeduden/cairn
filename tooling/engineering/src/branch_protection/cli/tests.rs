use std::io;

use super::*;

const COMMITTED: &str = r#"{"enforcement":"active","rules":[{"type":"required_signatures"}]}"#;

fn run_with(args: &[&str], live: &str) -> (u8, String, String) {
    let read = |path: &str| match path {
        "committed.json" => Ok(COMMITTED.to_owned()),
        "live.json" => Ok(live.to_owned()),
        _ => Err(io::Error::from(io::ErrorKind::NotFound)),
    };
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run(
        args.iter().map(|a| (*a).to_owned()),
        &mut out,
        &mut err,
        &read,
    );

    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

const ARGS: [&str; 4] = ["--committed", "committed.json", "--live", "live.json"];

#[test]
fn an_equal_live_ruleset_passes() {
    let (code, out, err) = run_with(&ARGS, COMMITTED);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert!(out.contains("as strong as the committed one"));
}

#[test]
fn a_weaker_live_ruleset_fails_with_every_problem() {
    let (code, _, err) = run_with(&ARGS, r#"{"enforcement":"disabled","rules":[]}"#);

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("protection-check: enforcement is \"disabled\" live"));
    assert!(err.contains("protection-check: live has no required_signatures rule"));
}

#[test]
fn a_file_that_cannot_be_read_or_parsed_fails() {
    let (code, _, err) = run_with(&["--committed", "nope.json", "--live", "live.json"], "{}");
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("read nope.json"), "{err}");

    let (code, _, err) = run_with(&ARGS, "not json");
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("live.json: the ruleset: "), "{err}");
}

#[test]
fn a_wrong_command_line_is_a_usage_error() {
    for (args, want) in [
        (vec![], "--committed is required"),
        (vec!["--committed", "committed.json"], "--live is required"),
        (vec!["--live"], "--live needs a value"),
        (vec!["--live", ""], "--live needs a value"),
        (vec!["x"], "unexpected argument \"x\""),
    ] {
        let (code, _, err) = run_with(&args, "{}");
        assert_eq!(code, EXIT_USAGE, "{args:?}");
        assert!(err.contains(want) && err.contains("usage:"), "{err}");
    }
}
