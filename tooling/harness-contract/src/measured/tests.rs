use super::*;
use serde_json::json;

fn doc() -> Value {
    json!({
        "trials": {
            "both": {"mode": "print", "configured": ["Stop", "SessionEnd"]},
            "quick-stop": {"mode": "print", "configured": ["Stop", "SessionEnd"]},
            "interactive": {"mode": "interactive", "configured": ["Stop", "SessionEnd"]},
            "stop-only": {"mode": "print", "configured": ["Stop"]},
            "end-only": {"mode": "print", "configured": ["SessionEnd"]}
        },
        "measurements": [
            {"mode": "print", "trial": "both", "event": "Stop", "configured_timeout_s": 4, "ran_ms": 3933, "finished": false},
            {"mode": "print", "trial": "quick-stop", "event": "Stop", "configured_timeout_s": 4, "ran_ms": 1034, "finished": true},
            {"mode": "print", "trial": "quick-stop", "event": "SessionEnd", "configured_timeout_s": 4, "ran_ms": 3935, "finished": false},
            {"mode": "interactive", "trial": "interactive", "event": "Stop", "configured_timeout_s": 4, "ran_ms": 3933, "finished": false},
            {"mode": "print", "trial": "stop-only", "event": "Stop", "configured_timeout_s": 4, "ran_ms": 3933, "finished": false},
            {"mode": "print", "trial": "end-only", "event": "SessionEnd", "configured_timeout_s": null, "ran_ms": 1447, "finished": false}
        ]
    })
}

#[test]
fn measurements_read_each_entry() {
    let all = measurements(&doc()).unwrap();

    assert_eq!(all.len(), 6);
    assert_eq!(
        all[5],
        Measurement {
            mode: "print".into(),
            trial: "end-only".into(),
            event: "SessionEnd".into(),
            timeout_s: None,
            ran_ms: 1447,
            finished: false
        }
    );
    assert_eq!(all[0].timeout_s, Some(4.0));
}

#[test]
fn measurements_refuse_an_empty_or_broken_document() {
    assert!(measurements(&json!({})).is_err());
    assert!(measurements(&json!({"measurements": []})).is_err());
    assert!(
        measurements(&json!({"measurements": [{"event": "Stop"}]}))
            .unwrap_err()
            .contains("no ran_ms")
    );
}

#[test]
fn matching_selects_by_event_mode_and_timeout() {
    let all = measurements(&doc()).unwrap();

    assert_eq!(matching(&all, "SessionEnd", "print", None), [&all[5]]);
    assert_eq!(matching(&all, "Stop", "print", Some(4.0)).len(), 3);
    assert!(matching(&all, "Stop", "print", Some(10.0)).is_empty());
}

#[test]
fn about_allows_the_tolerance_either_way() {
    assert!(about(3933, 4.0));
    assert!(about(1447, 1.5));
    assert!(about(4500, 4.0));
    assert!(about(4600, 4.0));
    assert!(!about(4601, 4.0));
    assert!(!about(2000, 4.0));
    assert!(!about(4700, 4.0));
}

#[test]
fn session_end_after_stop_timeout_checks_print_runs_only() {
    assert_eq!(session_end_after_stop_timeout(&doc()).unwrap(), (1, vec![]));
}

#[test]
fn session_end_after_stop_timeout_names_a_run_that_started_it() {
    let mut d = doc();
    d["measurements"].as_array_mut().unwrap().push(json!(
        {"mode": "print", "trial": "both", "event": "SessionEnd", "ran_ms": 10, "finished": true}
    ));

    assert_eq!(
        session_end_after_stop_timeout(&d).unwrap(),
        (
            1,
            vec!["trial both: SessionEnd started after Stop reached its timeout".to_owned()]
        )
    );
}

#[test]
fn session_end_after_stop_timeout_needs_runs_and_measurements() {
    assert!(session_end_after_stop_timeout(&json!({})).is_err());
    let mut d = doc();
    d.as_object_mut().unwrap().remove("trials");
    assert_eq!(
        session_end_after_stop_timeout(&d).unwrap_err(),
        "timeouts: no trials"
    );
}

#[test]
fn cleanup_splits_deleted_from_kept() {
    let d = json!({
        "before": [{"path": "a", "age_days": 40}, {"path": "b", "age_days": 20}, {"path": "c", "age_days": 0}],
        "after": ["b", "c"]
    });

    let (deleted, kept) = cleanup(&d).unwrap();

    assert_eq!(
        deleted,
        [Aged {
            path: "a".into(),
            age_days: 40.0
        }]
    );
    assert_eq!(
        kept.iter().map(|a| a.age_days).collect::<Vec<_>>(),
        [20.0, 0.0]
    );
}

#[test]
fn cleanup_needs_both_lists() {
    assert!(cleanup(&json!({"after": []})).is_err());
    assert!(cleanup(&json!({"before": [{"path": "a"}]})).is_err());
}

#[test]
fn measurements_refuse_one_without_finished() {
    let d = json!({"measurements": [{"event": "Stop", "ran_ms": 1}]});

    assert!(measurements(&d).unwrap_err().contains("no finished"));
}

#[test]
fn cleanup_refuses_a_transcript_without_its_age() {
    let d = json!({"before": [{"path": "a"}], "after": []});

    assert!(cleanup(&d).unwrap_err().contains("no age_days"));
}
