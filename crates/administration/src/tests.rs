use super::*;

#[test]
fn errors_explain_themselves_and_keep_their_cause() {
    let io = AdminError::Io {
        path: PathBuf::from("/h"),
        source: std::io::Error::other("x"),
    };
    assert_eq!(io.to_string(), "/h: x");
    assert!(std::error::Error::source(&io).is_some());
    let key = AdminError::from(key::KeyError::Malformed("k".into()));
    assert_eq!(key.to_string(), "malformed k");
    assert!(std::error::Error::source(&key).is_some());
    let store = AdminError::from(store::StoreError::Newer { found: 9 });
    assert!(store.to_string().contains("version 9"));
    assert!(std::error::Error::source(&store).is_some());
    assert!(std::error::Error::source(&AdminError::NoHome).is_none());
    assert!(std::error::Error::source(&AdminError::Node("n")).is_none());
}
