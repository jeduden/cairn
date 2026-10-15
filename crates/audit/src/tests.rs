use std::error::Error as _;
use std::path::PathBuf;

use super::*;

#[test]
fn each_error_says_what_failed_and_keeps_its_cause() {
    let io = AuditError::Io {
        path: PathBuf::from("/h/audit"),
        source: std::io::Error::other("disk full"),
    };
    assert_eq!(io.to_string(), "/h/audit: disk full");
    assert_eq!(io.source().unwrap().to_string(), "disk full");

    let busy = AuditError::Busy {
        path: PathBuf::from("/h/audit/lock"),
    };
    assert_eq!(
        busy.to_string(),
        "/h/audit/lock: the audit log stays locked"
    );
    assert!(busy.source().is_none());

    let file = AuditError::Broken {
        file: "audit-000002.jsonl".into(),
        line: 0,
        why: "missing",
    };
    assert_eq!(file.to_string(), "audit log audit-000002.jsonl: missing");
    let line = AuditError::Broken {
        file: "audit-000001.jsonl".into(),
        line: 3,
        why: "hash does not match",
    };
    assert_eq!(
        line.to_string(),
        "audit log audit-000001.jsonl line 3: hash does not match"
    );
    assert!(line.source().is_none());

    let encode = AuditError::Encode(record::canonical::Error::Number("1.5".into()));
    assert!(
        encode
            .to_string()
            .starts_with("audit entry: canonical encoding")
    );
    assert!(encode.source().is_some());
}
