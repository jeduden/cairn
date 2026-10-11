use super::*;
use serde_json::json;

#[test]
fn at_follows_a_dotted_path() {
    let v = json!({"attachment": {"type": "skill_listing"}});

    assert_eq!(at(&v, "attachment.type"), Some(&json!("skill_listing")));
    assert_eq!(at(&v, "attachment.missing"), None);
    assert_eq!(at(&v, "nowhere.type"), None);
}

#[test]
fn text_is_empty_unless_a_string_is_there() {
    let v = json!({"source": "resume", "n": 1});

    assert_eq!(text(&v, "source"), "resume");
    assert_eq!(text(&v, "n"), "");
    assert_eq!(text(&v, "missing"), "");
}
