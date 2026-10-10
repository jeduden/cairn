use super::*;
use crate::{APPROVE, Review};
use std::collections::BTreeMap;

const HEAD: &str = "206b0a76a81abb510e53d87ef187e83b5aacbcbb";

fn inputs() -> BTreeMap<String, String> {
    [
        (
            "outcome.json",
            r#"{"verdict":"approve","summary":"Fine.","findings":[]}"#,
        ),
        (
            "check-runs.jsonl",
            r#"{"name":"CI","status":"completed","conclusion":"success"}"#,
        ),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v.to_owned()))
    .collect()
}

fn args(current: &str) -> Vec<String> {
    [
        "--outcome",
        "outcome.json",
        "--check-runs",
        "check-runs.jsonl",
        "--reviewed",
        HEAD,
        "--head",
        current,
    ]
    .map(str::to_owned)
    .to_vec()
}

/// Runs the command over `files`, returning its code, stdout and
/// stderr.
fn invoke(args: Vec<String>, files: &BTreeMap<String, String>) -> (u8, String, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let read = |path: &str| {
        files
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
    };
    let code = run(args, &mut out, &mut err, &read);
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

#[test]
fn run_prints_the_review_to_post() {
    let (code, out, err) = invoke(args(HEAD), &inputs());

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(out.ends_with('\n'));
    let review: Review = serde_json::from_str(&out).unwrap();
    assert_eq!(review.event, APPROVE);
    assert_eq!(review.commit_id, HEAD);
}

#[test]
fn run_takes_inline_flag_values() {
    let mut args = args(HEAD);
    args.splice(0..2, ["--outcome=outcome.json".to_owned()]);

    assert_eq!(invoke(args, &inputs()).0, EXIT_OK);
}

#[test]
fn run_prints_nothing_for_a_head_that_moved() {
    let (code, out, err) = invoke(args("490efa734617788d1dc1f831b26d0e23888e90fa"), &inputs());

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(out.is_empty());
    assert!(err.contains("moved past"), "{err}");
}

#[test]
fn run_fails_closed() {
    let with = |name: &str, body: &str| {
        let mut m = inputs();
        m.insert(name.to_owned(), body.to_owned());
        m
    };
    let without = |name: &str| {
        let mut m = inputs();
        m.remove(name);
        m
    };
    let mut stray = args(HEAD);
    stray.push("extra".into());
    let cases = [
        (
            "an unknown flag",
            vec!["--approve".to_owned()],
            inputs(),
            r#"unknown flag "--approve""#,
            EXIT_USAGE,
        ),
        (
            "a flag with no value",
            vec!["--head".to_owned()],
            inputs(),
            "--head needs a value",
            EXIT_USAGE,
        ),
        (
            "a missing flag",
            args(HEAD)[..6].to_vec(),
            inputs(),
            "--head is required",
            EXIT_USAGE,
        ),
        (
            "an empty reviewed head",
            {
                let mut a = args(HEAD);
                a[5] = String::new();
                a
            },
            inputs(),
            "--reviewed needs a value",
            EXIT_USAGE,
        ),
        (
            "an empty inline head",
            vec![
                "--outcome=outcome.json".to_owned(),
                "--check-runs=check-runs.jsonl".to_owned(),
                format!("--reviewed={HEAD}"),
                "--head=".to_owned(),
            ],
            inputs(),
            "--head needs a value",
            EXIT_USAGE,
        ),
        (
            "a stray argument",
            stray,
            inputs(),
            "unexpected argument",
            EXIT_USAGE,
        ),
        (
            "no outcome file",
            args(HEAD),
            BTreeMap::new(),
            "read outcome.json",
            EXIT_FAILURE,
        ),
        (
            "a malformed outcome",
            args(HEAD),
            with("outcome.json", "{}"),
            "malformed review outcome",
            EXIT_FAILURE,
        ),
        (
            "no check runs file",
            args(HEAD),
            without("check-runs.jsonl"),
            "read check-runs.jsonl",
            EXIT_FAILURE,
        ),
        (
            "a malformed check run",
            args(HEAD),
            with("check-runs.jsonl", "nope"),
            "check run line 1",
            EXIT_FAILURE,
        ),
        (
            "CI not passed at head",
            args(HEAD),
            with("check-runs.jsonl", ""),
            "CI has not passed",
            EXIT_FAILURE,
        ),
    ];
    for (name, args, files, want, code) in cases {
        let (got, out, err) = invoke(args, &files);
        assert_eq!(got, code, "{name}: {err}");
        assert!(out.is_empty(), "{name}");
        assert!(err.contains(want), "{name}: {err}");
    }
}

/// A stdout that refuses every write, as a closed pipe does.
struct Closed;

impl Write for Closed {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn run_reports_a_failed_write() {
    let files = inputs();
    let read = |path: &str| {
        files
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
    };
    let mut err = Vec::new();

    let code = run(args(HEAD), &mut Closed, &mut err, &read);

    assert_eq!(code, EXIT_FAILURE);
    assert!(String::from_utf8(err).unwrap().contains("closed"));
    assert!(Closed.flush().is_ok());
}
