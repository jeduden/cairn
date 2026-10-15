use super::*;
use serde_json::json;

#[test]
fn literals_and_integers_encode_as_json_writes_them() {
    assert_eq!(encode(&json!(null)).unwrap(), "null");
    assert_eq!(encode(&json!(true)).unwrap(), "true");
    assert_eq!(encode(&json!(false)).unwrap(), "false");
    assert_eq!(encode(&json!(0)).unwrap(), "0");
    assert_eq!(encode(&json!(-42)).unwrap(), "-42");
    assert_eq!(encode(&json!(MAX_SAFE)).unwrap(), "9007199254740991");
    assert_eq!(
        encode(&json!(-9_007_199_254_740_991_i64)).unwrap(),
        "-9007199254740991"
    );
}

#[test]
fn a_number_past_the_safe_integers_or_with_a_fraction_is_refused() {
    for n in [
        json!(MAX_SAFE + 1),
        json!(-9_007_199_254_740_992_i64),
        json!(1.5),
    ] {
        assert_eq!(encode(&n), Err(Error::Number(n.to_string())), "{n}");
    }
    assert_eq!(
        Error::Number("1.5".into()).to_string(),
        "canonical encoding: 1.5 is not an integer within ±(2^53 - 1)"
    );
}

#[test]
fn objects_sort_their_members_by_utf16_code_units_and_drop_whitespace() {
    // RFC 8785 §3.2.3: U+E000 sorts before U+1F600, whose UTF-16 form
    // starts with a surrogate (0xD83D), though UTF-8 orders them the
    // other way round.
    let value = json!({"b": [1, {"z": null, "a": "x"}], "a": {}, "\u{1F600}": 1, "\u{E000}": 2});

    assert_eq!(
        encode(&value).unwrap(),
        "{\"a\":{},\"b\":[1,{\"a\":\"x\",\"z\":null}],\"\u{1F600}\":1,\"\u{E000}\":2}"
    );
}

#[test]
fn strings_escape_only_what_rfc_8785_escapes() {
    let value = json!("q\"b\\\u{8}\t\n\u{c}\r\u{1}\u{1f}é\u{2028}/");

    assert_eq!(
        encode(&value).unwrap(),
        "\"q\\\"b\\\\\\b\\t\\n\\f\\r\\u0001\\u001fé\u{2028}/\""
    );
}

#[test]
fn a_refused_number_inside_an_array_or_object_refuses_the_whole() {
    assert!(encode(&json!([1, 2.5])).is_err());
    assert!(encode(&json!({"a": 2.5})).is_err());
}
