use serde_json::json;

use super::*;

fn good() -> Value {
    json!({
        "name": "main",
        "target": "branch",
        "enforcement": "active",
        "conditions": {"ref_name": {"exclude": [], "include": ["~DEFAULT_BRANCH"]}},
        "bypass_actors": [],
        "rules": [
            {"type": "deletion"},
            {"type": "non_fast_forward"},
            {"type": "required_signatures"},
            {"type": "pull_request", "parameters": {
                "required_approving_review_count": 1,
                "dismiss_stale_reviews_on_push": true,
                "require_code_owner_review": true,
                "require_last_push_approval": true,
                "required_review_thread_resolution": false
            }},
            {"type": "required_status_checks", "parameters": {
                "strict_required_status_checks_policy": false,
                "required_status_checks": [{"context": "CI"}]
            }}
        ]
    })
}

/// `good()` with the rule `kind` removed.
fn without(kind: &str) -> Value {
    let mut r = good();
    r["rules"]
        .as_array_mut()
        .unwrap()
        .retain(|x| x["type"] != kind);
    r
}

fn with_param(kind: &str, name: &str, value: Value) -> Value {
    let mut r = good();
    for rule in r["rules"].as_array_mut().unwrap() {
        if rule["type"] == kind {
            rule["parameters"][name] = value.clone();
        }
    }
    r
}

fn has(problems: &[String], want: &str) {
    assert!(
        problems.iter().any(|p| p.contains(want)),
        "{want:?} not in {problems:#?}"
    );
}

#[test]
fn the_good_ruleset_has_no_problem() {
    let r = good();

    assert!(direct_pushes_rejected(&r).is_empty());
    assert!(commits_signed(&r).is_empty());
    assert!(approval_from_another_reviewer(&r).is_empty());
    assert!(weaker_than(&r, &r).is_empty());
}

#[test]
fn load_and_parse_read_a_ruleset_object() {
    let (_d, c) = crate::tests::checkout(&[(RULESET, &good().to_string())]);
    assert_eq!(load(&c).unwrap(), good());

    assert!(parse("nope").unwrap_err().starts_with("the ruleset: "));
    assert_eq!(parse("[]").unwrap_err(), "the ruleset is not a JSON object");
    let (_d, c) = crate::tests::checkout(&[]);
    assert!(load(&c).unwrap_err().starts_with("read .github/rulesets"));
}

#[test]
fn direct_pushes_need_an_active_ruleset_on_main_with_no_way_round() {
    let mut r = good();
    r["enforcement"] = json!("evaluate");
    has(&direct_pushes_rejected(&r), "enforcement is not active");

    let mut r = good();
    r["target"] = json!("tag");
    has(&direct_pushes_rejected(&r), "does not target branches");

    let mut r = good();
    r["conditions"]["ref_name"]["include"] = json!(["refs/heads/dev"]);
    has(
        &direct_pushes_rejected(&r),
        "does not include the main branch",
    );

    let mut r = good();
    r["conditions"]["ref_name"]["include"] = json!(["refs/heads/main"]);
    assert!(direct_pushes_rejected(&r).is_empty());

    let mut r = good();
    r["conditions"]["ref_name"]["exclude"] = json!(["refs/heads/main"]);
    has(&direct_pushes_rejected(&r), "excludes a branch");

    let mut r = good();
    r["bypass_actors"] = json!([{"actor_id": 5}]);
    has(&direct_pushes_rejected(&r), "lets a bypass actor bypass");

    for kind in ["pull_request", "non_fast_forward", "deletion"] {
        has(
            &direct_pushes_rejected(&without(kind)),
            &format!("no {kind} rule"),
        );
    }
    has(
        &direct_pushes_rejected(&without("required_status_checks")),
        "does not require the CI check",
    );
}

#[test]
fn signed_commits_need_the_signature_rule() {
    has(
        &commits_signed(&without("required_signatures")),
        "no required_signatures rule",
    );
}

#[test]
fn an_approval_needs_a_count_and_the_flags_that_keep_it_honest() {
    has(
        &approval_from_another_reviewer(&with_param(
            "pull_request",
            "required_approving_review_count",
            json!(0),
        )),
        "requires no approving review",
    );
    has(
        &approval_from_another_reviewer(&without("pull_request")),
        "requires no approving review",
    );
    for flag in [
        "require_last_push_approval",
        "dismiss_stale_reviews_on_push",
        "require_code_owner_review",
    ] {
        has(
            &approval_from_another_reviewer(&with_param("pull_request", flag, json!(false))),
            &format!("does not set {flag}"),
        );
    }
}

#[test]
fn two_approvals_on_security_crates_need_a_required_check() {
    has(
        &security_crates_need_two_approvals(&good()),
        "requires no \"Security approvals\" check",
    );

    let r = with_param(
        "required_status_checks",
        "required_status_checks",
        json!([{"context": "CI"}, {"context": "Security approvals"}]),
    );
    assert!(security_crates_need_two_approvals(&r).is_empty());
}

#[test]
fn a_live_ruleset_weaker_than_the_committed_one_is_named() {
    let committed = good();

    let mut live = good();
    live["enforcement"] = json!("disabled");
    has(
        &weaker_than(&committed, &live),
        "enforcement is \"disabled\" live, \"active\" committed",
    );

    let mut live = good();
    live.as_object_mut().unwrap().remove("enforcement");
    has(
        &weaker_than(&committed, &live),
        "enforcement is absent live",
    );

    let mut live = good();
    live["conditions"]["ref_name"]["include"] = json!([]);
    has(
        &weaker_than(&committed, &live),
        "live does not include ~DEFAULT_BRANCH",
    );

    let mut live = good();
    live["conditions"]["ref_name"]["exclude"] = json!(["refs/heads/main"]);
    has(
        &weaker_than(&committed, &live),
        "live excludes refs/heads/main",
    );

    has(
        &weaker_than(&committed, &without("required_signatures")),
        "live has no required_signatures rule",
    );

    has(
        &weaker_than(
            &committed,
            &with_param("pull_request", "required_approving_review_count", json!(0)),
        ),
        "live pull_request required_approving_review_count is 0, committed 1",
    );
    has(
        &weaker_than(&committed, &without("pull_request")),
        "live has no pull_request rule",
    );
    let mut live = good();
    for rule in live["rules"].as_array_mut().unwrap() {
        if rule["type"] == "pull_request" {
            rule["parameters"]
                .as_object_mut()
                .unwrap()
                .remove("required_approving_review_count");
        }
    }
    has(
        &weaker_than(&committed, &live),
        "required_approving_review_count is absent, committed 1",
    );
    has(
        &weaker_than(
            &committed,
            &with_param("pull_request", "require_code_owner_review", json!(false)),
        ),
        "live pull_request does not set require_code_owner_review",
    );
    has(
        &weaker_than(
            &committed,
            &with_param(
                "required_status_checks",
                "required_status_checks",
                json!([{"context": "lint"}]),
            ),
        ),
        "live does not require the CI check",
    );

    let mut live = good();
    live["bypass_actors"] = json!([{"actor_id": 5}]);
    has(
        &weaker_than(&committed, &live),
        "live lets a bypass actor bypass",
    );
}

#[test]
fn a_live_ruleset_on_another_target_is_weaker() {
    let mut live = good();
    live["target"] = json!("tag");

    has(
        &weaker_than(&good(), &live),
        "target is \"tag\" live, \"branch\" committed",
    );
}

#[test]
fn a_do_not_flag_set_live_but_not_committed_is_weaker() {
    let mut committed = good();
    for rule in committed["rules"].as_array_mut().unwrap() {
        if rule["type"] == "required_status_checks" {
            rule["parameters"]["do_not_enforce_on_create"] = json!(false);
        }
    }
    let mut live = committed.clone();
    for rule in live["rules"].as_array_mut().unwrap() {
        if rule["type"] == "required_status_checks" {
            rule["parameters"]["do_not_enforce_on_create"] = json!(true);
        }
    }

    has(
        &weaker_than(&committed, &live),
        "live required_status_checks sets do_not_enforce_on_create",
    );
    assert!(weaker_than(&committed, &committed).is_empty());
}

#[test]
fn a_stronger_live_ruleset_or_one_without_bypass_actors_is_not_weaker() {
    let committed = good();

    let mut live = good();
    live.as_object_mut().unwrap().remove("bypass_actors");
    live["rules"]
        .as_array_mut()
        .unwrap()
        .push(json!({"type": "creation"}));
    for rule in live["rules"].as_array_mut().unwrap() {
        if rule["type"] == "pull_request" {
            rule["parameters"]["required_approving_review_count"] = json!(2);
            rule["parameters"]["required_review_thread_resolution"] = json!(true);
        }
    }
    assert!(weaker_than(&committed, &live).is_empty());

    let mut weird = good();
    weird["rules"] = json!([{"type": "deletion", "parameters": 3}, {"type": "pull_request"}]);
    assert!(weaker_than(&weird, &weird).is_empty());
}

#[test]
fn an_unreadable_ruleset_has_no_rules() {
    let r = json!({"rules": 3});

    assert!(rule_types(&r).is_empty());
    assert!(required_checks(&r).is_empty());
    assert_eq!(show(None), "absent");
}
