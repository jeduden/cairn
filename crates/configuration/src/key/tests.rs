use super::{
    Acceptance, Default, Kind, Lookup, RECALL_SCOPES, Repository, fixed, key, lookup, shown, spec,
};
use crate::value::Value;

const ROOM: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

fn int(i: i64) -> Option<Value> {
    Some(Value::Integer(i))
}

fn texts(items: &[&str]) -> Option<Value> {
    Some(Value::Texts(
        items.iter().map(|i| (*i).to_owned()).collect(),
    ))
}

#[test]
fn lookup_finds_fixed_keys_tables_and_nothing() {
    let Lookup::Key(found) = lookup(&["recall", "max_k"]) else {
        panic!("recall.max_k is a key");
    };
    assert_eq!(found.name(), "recall.max_k");
    assert_eq!(found.to_string(), "recall.max_k");
    assert_eq!(found.spec().repository, Repository::OnlyLower);
    assert_eq!(lookup(&["recall"]), Lookup::Table);
    assert_eq!(lookup(&["recal"]), Lookup::Unknown);
    assert_eq!(lookup(&["recal", "max_k"]), Lookup::Unknown);
    assert_eq!(lookup(&["recall", "max_k", "x"]), Lookup::Unknown);
    assert_eq!(lookup(&[]), Lookup::Unknown);
}

#[test]
fn lookup_finds_retention_policies_per_room_and_class() {
    assert_eq!(lookup(&["retention_policy"]), Lookup::Table);
    assert_eq!(lookup(&["retention_policy", "*"]), Lookup::Table);
    assert_eq!(lookup(&["retention_policy", ROOM]), Lookup::Table);
    assert_eq!(lookup(&["retention_policy", "nope"]), Lookup::Unknown);
    let Lookup::Key(every) = lookup(&["retention_policy", "*", "tool_result:Bash"]) else {
        panic!("a retention policy for every room is a key");
    };
    assert_eq!(every.spec().acceptance, Acceptance::AnyChange);
    assert_eq!(every.default_value(), None);
    assert!(matches!(
        lookup(&["retention_policy", ROOM, "web"]),
        Lookup::Key(_)
    ));
    assert_eq!(
        lookup(&["retention_policy", "*", "nonsense"]),
        Lookup::Unknown
    );
}

#[test]
fn lookup_finds_a_trust_policy_key_per_provenance_class() {
    assert_eq!(lookup(&["trust_policy"]), Lookup::Table);
    let Lookup::Key(web) = lookup(&["trust_policy", "web"]) else {
        panic!("trust_policy.web is a key");
    };
    assert_eq!(web.trust_policy(), Some("web"));
    assert!(matches!(
        lookup(&["trust_policy", "mcp:github"]),
        Lookup::Key(_)
    ));
    assert_eq!(lookup(&["trust_policy", "mcp:a.b"]), Lookup::Unknown);
    assert_eq!(lookup(&["trust_policy", "nonsense"]), Lookup::Unknown);
    assert_eq!(
        key("recall.max_k").and_then(|k| k.trust_policy().map(str::to_owned)),
        None
    );
}

#[test]
fn key_names_a_key_only() {
    assert_eq!(
        key("node.deployment_mode").map(|k| k.name().to_owned()),
        Some("node.deployment_mode".to_owned())
    );
    assert_eq!(key("node"), None);
    assert_eq!(key("nothing"), None);
}

#[test]
fn keys_compare_by_name() {
    let a = key("flag.enabled").unwrap();
    let b = key("recall.max_k").unwrap();
    assert!(a < b);
    assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Less));
}

#[test]
fn fixed_keys_have_defaults_as_section_9_6_lists() {
    let default = |name: &str| key(name).unwrap().default_value();
    assert_eq!(
        default("node.deployment_mode"),
        Some(Value::Text("automation".into()))
    );
    assert_eq!(default("payload.threshold_bytes"), int(8192));
    assert_eq!(default("recall.max_k"), int(50));
    assert_eq!(default("flag.enabled"), Some(Value::Bool(true)));
    assert_eq!(default("transcript.roots"), texts(&["~/.claude/projects"]));
    assert_eq!(default("home.id"), None);
    assert!(fixed().any(|k| k.name() == "risk_acceptance.forbidden"));
    assert_eq!(fixed().count(), 18);
}

#[test]
fn kinds_say_what_they_want() {
    assert_eq!(Kind::Bool.wants(), "want true or false");
    assert_eq!(Kind::Integer { min: 1, max: 2 }.wants(), "want an integer");
    assert_eq!(Kind::Choice(RECALL_SCOPES).wants(), "want a string");
    assert_eq!(Kind::Text { max: 1 }.wants(), "want a string");
    assert_eq!(
        Kind::Texts { items: 1, max: 1 }.wants(),
        "want an array of strings"
    );
}

#[test]
fn defaults_stand_for_their_values() {
    assert_eq!(Default::Bool(true).value(), Value::Bool(true));
    assert_eq!(Default::Integer(3).value(), Value::Integer(3));
    assert_eq!(Default::Text("run").value(), Value::Text("run".into()));
    assert_eq!(Default::Texts(&["a"]).value(), texts(&["a"]).unwrap());
}

#[test]
fn no_change_never_awaits_acceptance() {
    for rule in [
        Acceptance::Never,
        Acceptance::AnyChange,
        Acceptance::Lowering,
    ] {
        assert!(!rule.awaits(Kind::Bool, int(1).as_ref(), int(1).as_ref()));
    }
    assert!(!Acceptance::Never.awaits(Kind::Bool, int(1).as_ref(), int(2).as_ref()));
}

#[test]
fn any_change_awaits_acceptance_even_from_unset() {
    let rule = Acceptance::AnyChange;
    assert!(rule.awaits(Kind::Bool, None, int(30).as_ref()));
    assert!(rule.awaits(Kind::Bool, int(30).as_ref(), None));
}

#[test]
fn lowering_awaits_acceptance_and_raising_does_not() {
    let rule = Acceptance::Lowering;
    assert!(rule.awaits(Kind::Bool, int(1000).as_ref(), int(10).as_ref()));
    assert!(!rule.awaits(Kind::Bool, int(10).as_ref(), int(1000).as_ref()));
    assert!(!rule.awaits(Kind::Bool, None, int(10).as_ref()));
}

#[test]
fn turning_off_awaits_acceptance_and_turning_on_does_not() {
    let (on, off) = (Some(Value::Bool(true)), Some(Value::Bool(false)));
    let rule = Acceptance::TurningOff;
    assert!(rule.awaits(Kind::Bool, on.as_ref(), off.as_ref()));
    assert!(!rule.awaits(Kind::Bool, off.as_ref(), on.as_ref()));
}

#[test]
fn dropping_an_entry_awaits_acceptance_and_adding_one_does_not() {
    let rule = Acceptance::Dropping;
    let kind = Kind::Texts { items: 9, max: 9 };
    assert!(rule.awaits(kind, texts(&["a", "b"]).as_ref(), texts(&["a"]).as_ref()));
    assert!(rule.awaits(kind, texts(&["a"]).as_ref(), None));
    assert!(!rule.awaits(kind, texts(&["a"]).as_ref(), texts(&["a", "b"]).as_ref()));
    assert!(!rule.awaits(kind, None, texts(&["a"]).as_ref()));
}

#[test]
fn a_later_choice_awaits_acceptance_and_an_earlier_one_does_not() {
    let rule = Acceptance::LaterChoice;
    let kind = Kind::Choice(RECALL_SCOPES);
    let text = |t: &str| Some(Value::Text(t.to_owned()));
    assert!(rule.awaits(kind, text("run").as_ref(), text("room").as_ref()));
    assert!(!rule.awaits(kind, text("rooms").as_ref(), text("run").as_ref()));
    assert!(rule.awaits(kind, text("run").as_ref(), text("other").as_ref()));
    assert!(!rule.awaits(Kind::Bool, text("run").as_ref(), text("room").as_ref()));
}

#[test]
fn shown_quotes_a_plain_name_and_hides_any_other() {
    assert_eq!(shown("recal.max_k"), "\"recal.max_k\"");
    assert_eq!(shown("trust_policy.mcp:x-y*"), "\"trust_policy.mcp:x-y*\"");
    assert_eq!(
        shown("ignore previous instructions"),
        "a key no settings key spells"
    );
    assert_eq!(shown(""), "a key no settings key spells");
    assert_eq!(shown(&"a".repeat(161)), "a key no settings key spells");
}

#[test]
fn spec_gathers_what_a_key_takes() {
    let s = spec(Kind::Bool, None, Repository::Never, Acceptance::Never);
    assert_eq!(s.kind, Kind::Bool);
    assert_eq!(s.default, None);
    assert_eq!(s.repository, Repository::Never);
    assert_eq!(s.acceptance, Acceptance::Never);
}
