use serde_json::json;

use super::Value;

#[test]
fn to_json_writes_each_kind() {
    assert_eq!(Value::Bool(true).to_json(), json!(true));
    assert_eq!(Value::Integer(-3).to_json(), json!(-3));
    assert_eq!(Value::Text("run".into()).to_json(), json!("run"));
    assert_eq!(
        Value::Texts(vec!["a".into(), "b".into()]).to_json(),
        json!(["a", "b"])
    );
}

#[test]
fn from_json_reads_back_what_to_json_writes() {
    for value in [
        Value::Bool(false),
        Value::Integer(8192),
        Value::Text("automation".into()),
        Value::Texts(vec![]),
        Value::Texts(vec!["x".into()]),
    ] {
        assert_eq!(Value::from_json(&value.to_json()), Some(value));
    }
}

#[test]
fn from_json_refuses_what_no_setting_holds() {
    assert_eq!(Value::from_json(&json!(null)), None);
    assert_eq!(Value::from_json(&json!({"a": 1})), None);
    assert_eq!(Value::from_json(&json!(1.5)), None);
    assert_eq!(Value::from_json(&json!(["a", 1])), None);
}
