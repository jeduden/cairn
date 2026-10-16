use serde_json::json;

use super::account::HarnessAccountIdentifiers;
use super::{PatternRefused, Redacted, Redactor, extra_pattern, merged, unique};

const AWS: &str = "AKIAIOSFODNN7EXAMPLE";

fn redactor(extra: &[&str]) -> Redactor {
    let extra: Vec<String> = extra.iter().map(|p| (*p).to_owned()).collect();
    let (r, refused) = Redactor::new(&extra).unwrap();
    assert!(refused.is_empty());

    r
}

#[test]
fn extra_pattern_names_the_principals_nth_pattern() {
    assert_eq!(extra_pattern(1), "extra_pattern_1");
}

#[test]
fn redact_text_replaces_each_secret_with_its_rules_marker() {
    let r = redactor(&["ACME-[0-9]{8}"]);
    let out = r.redact_text(&format!("a {AWS} b ACME-12345678 c"));
    assert_eq!(
        out,
        Redacted {
            text: "a [REDACTED:aws_access_key] b [REDACTED:extra_pattern_1] c".to_owned(),
            rules: vec!["aws_access_key".to_owned(), "extra_pattern_1".to_owned()],
        }
    );
    let clean = r.redact_text("nothing here");
    assert_eq!(clean.text, "nothing here");
    assert!(clean.rules.is_empty());
}

#[test]
fn overlapping_matches_merge_into_the_first_rules_marker() {
    let r = redactor(&["EXAMPLE tail"]);
    let out = r.redact_text(&format!("{AWS} tail!"));
    assert_eq!(out.text, "[REDACTED:aws_access_key]!");
    assert_eq!(out.rules, ["aws_access_key"]);
}

#[test]
fn merged_joins_overlaps_but_not_neighbours() {
    assert_eq!(
        merged(vec![(0, 4, 1), (2, 6, 0), (6, 8, 2), (7, 7, 3)]),
        [(0, 6, 1), (6, 8, 2)]
    );
    assert!(merged(Vec::new()).is_empty());
}

#[test]
fn a_refused_pattern_is_named_and_the_rest_still_apply() {
    let extra = vec!["(".to_owned(), "ACME".to_owned()];
    let (r, refused) = Redactor::new(&extra).unwrap();
    assert_eq!(refused.len(), 1);
    let PatternRefused { rule, .. } = &refused[0];
    assert_eq!(rule, "extra_pattern_1");
    assert_eq!(
        refused[0].to_string(),
        "redaction pattern extra_pattern_1 refused"
    );
    assert!(std::error::Error::source(&refused[0]).is_some());
    assert_eq!(r.redact_text("ACME").text, "[REDACTED:extra_pattern_2]");
}

#[test]
fn with_identifiers_redacts_the_harness_account_ahead_of_extra_patterns() {
    let mut ids = HarnessAccountIdentifiers::default();
    ids.learn(r#"{"attachment":{"type":"session_context","context":{"userEmail":"is user@example.com."}}}"#);
    let r = redactor(&["user@example"]).with_identifiers(&ids).unwrap();
    let out = r.redact_text("from user@example.com");
    assert_eq!(out.text, "from [REDACTED:account_email]");
    let none = redactor(&[])
        .with_identifiers(&HarnessAccountIdentifiers::default())
        .unwrap();
    assert_eq!(
        none.redact_text("user@example.com").text,
        "user@example.com"
    );
}

#[test]
fn redact_value_walks_strings_and_keys_and_keeps_the_rest() {
    let r = redactor(&[]);
    let value = json!({
        "message": {"content": [{"text": format!("out {AWS}")}]},
        AWS: 1,
        "n": 2.5,
        "ok": true,
        "nothing": null,
    });
    let (out, rules) = r.redact_value(&value);
    assert_eq!(
        out,
        json!({
            "message": {"content": [{"text": "out [REDACTED:aws_access_key]"}]},
            "[REDACTED:aws_access_key]": 1,
            "n": 2.5,
            "ok": true,
            "nothing": null,
        })
    );
    assert_eq!(rules.len(), 2);
}

#[test]
fn two_keys_that_redact_alike_both_keep_their_values() {
    let r = redactor(&[]);
    let value = json!({"AKIAIOSFODNN7EXAMPLA": 1, "AKIAIOSFODNN7EXAMPLB": 2});
    let (out, _) = r.redact_value(&value);
    assert_eq!(
        out,
        json!({"[REDACTED:aws_access_key]": 1, "[REDACTED:aws_access_key] (2)": 2})
    );
}

#[test]
fn unique_counts_past_every_key_taken() {
    let map = json!({"k": 0, "k (2)": 0});
    let serde_json::Value::Object(map) = map else {
        panic!("an object");
    };
    assert_eq!(unique("j".to_owned(), &map), "j");
    assert_eq!(unique("k".to_owned(), &map), "k (3)");
}

#[test]
fn redact_line_keeps_a_clean_line_byte_for_byte() {
    let r = redactor(&[]);
    let line = r#"{"b": 1,  "a": "x"}"#;
    assert_eq!(r.redact_line(line).text, line);
}

#[test]
fn redact_line_rewrites_a_json_line_with_an_escaped_secret() {
    let r = redactor(&[]);
    let line = r#"{"t":"AKIAIOSFODNN7EXAMPLE"}"#;
    let out = r.redact_line(line);
    assert_eq!(out.text, r#"{"t":"[REDACTED:aws_access_key]"}"#);
    assert_eq!(out.rules, ["aws_access_key"]);
}

#[test]
fn redact_line_redacts_raw_text_that_is_no_json() {
    let r = redactor(&[]);
    let out = r.redact_line(&format!("{{broken {AWS}"));
    assert_eq!(out.text, "{broken [REDACTED:aws_access_key]");
}

#[test]
fn a_marker_carries_nothing_of_the_secret() {
    let r = redactor(&[]);
    let out = r.redact_text(AWS);
    assert!(!out.text.contains("AKIA"));
    assert!(!out.text.contains("EXAMPLE"));
}
