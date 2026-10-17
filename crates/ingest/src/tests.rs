use super::*;
use key::random::OsEntropy;
use record::event::{EventKind, Provenance};
use std::ffi::OsStr;
use testkit::TempDir;

fn home(dir: &TempDir) -> Home {
    Home::locate(Some(dir.path().join(".cairn").as_os_str()), None).unwrap()
}

fn redactor() -> Redactor {
    Redactor::new(&[]).unwrap().0
}

fn transcript(dir: &TempDir, text: &str) -> std::path::PathBuf {
    let path = dir.path().join("t.jsonl");
    std::fs::write(&path, text).unwrap();
    path
}

#[test]
fn lines_splits_on_newlines_and_drops_only_the_empty_tail() {
    assert_eq!(lines("a\nb\n"), ["a", "b"]);
    assert_eq!(lines("a\n\nb"), ["a", "", "b"]);
    assert_eq!(lines("a\r\n"), ["a\r"]);
    assert!(lines("").is_empty());
}

#[test]
fn every_line_becomes_one_unparsed_event_of_a_new_ingest_seat() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let path = transcript(&dir, "{\"a\":1}\n{\"b\":2}\n");

    let ingested = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    assert_eq!(ingested.events, 2);
    let store = Store::open(&home.store()).unwrap();
    let log = store.log(ingested.writer).unwrap();
    assert_eq!(log.len(), 2);
    for (i, (stored, text)) in log.iter().zip(["{\"a\":1}", "{\"b\":2}"]).enumerate() {
        assert_eq!(stored.link.header.kind, EventKind::UnparsedLine);
        assert_eq!(stored.link.header.provenance, Provenance::Unparsed);
        assert_eq!(
            stored.content,
            Some(serde_json::json!({"line": i + 1, "text": text}))
        );
    }
    let seat = store.seat(ingested.writer).unwrap().unwrap();
    assert_eq!((seat.seat, seat.room), (ingested.seat, ingested.room));
    let device = key::signing::DeviceKey::load(&home.device_key()).unwrap();
    assert!(administration::verify::verify(&store, &device.public()).is_ok());
}

#[test]
fn each_ingest_starts_its_own_seat_in_the_same_personal_room() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let path = transcript(&dir, "x\n");

    let a = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    let b = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    assert_ne!(a.writer, b.writer);
    assert_eq!(a.room, b.room);
}

#[test]
fn an_unreadable_empty_or_non_utf8_transcript_records_nothing() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);

    let missing =
        ingest_path(&home, &dir.path().join("none"), &redactor(), &mut OsEntropy).unwrap_err();
    assert!(matches!(missing, IngestError::Read { .. }), "{missing}");
    assert!(
        missing.to_string().starts_with("read the transcript "),
        "{missing}"
    );
    assert!(std::error::Error::source(&missing).is_some());

    let empty = ingest_path(&home, &transcript(&dir, ""), &redactor(), &mut OsEntropy).unwrap_err();
    assert_eq!(
        empty.to_string(),
        format!(
            "the transcript {} holds no line",
            dir.path().join("t.jsonl").display()
        )
    );

    std::fs::write(dir.path().join("t.jsonl"), [0xff, b'\n']).unwrap();
    let bytes = ingest_path(
        &home,
        &dir.path().join("t.jsonl"),
        &redactor(),
        &mut OsEntropy,
    )
    .unwrap_err();
    assert_eq!(
        bytes.to_string(),
        format!(
            "the transcript {} is not UTF-8",
            dir.path().join("t.jsonl").display()
        )
    );
    assert!(std::error::Error::source(&bytes).is_none());

    assert!(!home.store().exists());
}

#[test]
fn a_home_that_cannot_be_made_is_returned() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("f"), "").unwrap();
    let home = Home::locate(Some(OsStr::new(dir.path().join("f").as_os_str())), None).unwrap();
    let path = transcript(&dir, "x\n");

    let err = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap_err();

    assert!(matches!(err, IngestError::Admin(_)), "{err}");
    assert!(
        err.to_string()
            .starts_with(&dir.path().join("f").display().to_string()),
        "{err}"
    );
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn a_store_that_cannot_open_or_a_random_source_that_fails_is_returned() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    home.create().unwrap();
    let path = transcript(&dir, "x\n");
    ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    let err = ingest_path(&home, &path, &redactor(), &mut Failing).unwrap_err();
    assert!(matches!(err, IngestError::Key(_)), "{err}");
    assert_eq!(err.to_string(), "draw random bytes: none");
    assert!(std::error::Error::source(&err).is_some());

    std::fs::write(
        home.store(),
        "not a database at all, but long enough to read a header",
    )
    .unwrap();
    let err = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap_err();
    assert!(matches!(err, IngestError::Store(_)), "{err}");
    assert!(err.to_string().starts_with("the store: "), "{err}");
    assert!(std::error::Error::source(&err).is_some());
}

/// A random source that always fails.
struct Failing;

impl key::random::Entropy for Failing {
    fn fill(&mut self, _: &mut [u8]) -> Result<(), key::KeyError> {
        Err(key::KeyError::Entropy("none".into()))
    }
}

#[test]
fn a_store_that_fails_part_way_names_the_writer_and_what_landed() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    ingest_path(&home, &transcript(&dir, "x\n"), &redactor(), &mut OsEntropy).unwrap();
    let conn = rusqlite::Connection::open(home.store()).unwrap();
    conn.execute_batch(
        "CREATE TRIGGER refuse BEFORE INSERT ON events WHEN NEW.seq = 150 \
         BEGIN SELECT RAISE(ABORT, 'refused'); END",
    )
    .unwrap();
    let text: String = (1..=250).map(|n| format!("line {n}\n")).collect();

    let err =
        ingest_path(&home, &transcript(&dir, &text), &redactor(), &mut OsEntropy).unwrap_err();

    let IngestError::Partial { writer, landed, .. } = &err else {
        panic!("{err}");
    };
    assert_eq!(*landed, 100);
    let store = Store::open(&home.store()).unwrap();
    assert_eq!(store.log(*writer).unwrap().len(), 100);
    assert!(
        err.to_string().starts_with(&format!(
            "writer {writer} recorded 100 of 250 lines before the store failed: the store: "
        )),
        "{err}"
    );
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn the_redacted_path_is_redacted_like_the_lines() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let named = dir.path().join("user@example.com");
    std::fs::create_dir_all(&named).unwrap();
    let path = named.join("ACME-12345678.jsonl");
    let context =
        r#"{"attachment":{"type":"session_context","context":{"userEmail":"user@example.com"}}}"#;
    std::fs::write(&path, format!("{context}\n")).unwrap();
    let (redactor, _) = Redactor::new(&["ACME-[0-9]{8}".to_owned()]).unwrap();

    let ingested = ingest_path(&home, &path, &redactor, &mut OsEntropy).unwrap();

    let shown = &ingested.redacted_path;
    assert!(shown.contains("[REDACTED:account_email]"), "{shown}");
    assert!(shown.contains("[REDACTED:extra_pattern_1]"), "{shown}");
}

#[test]
fn every_line_is_redacted_before_anything_is_written() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let context = r#"{"attachment":{"type":"session_context","context":{"userEmail":"is user@example.com."}},"type":"attachment"}"#;
    let org = r#"{"attachment":{"type":"credential_org","organizationUuid":"0f0e0d0c-0000-0000-0000-000000000001"}}"#;
    let tool = r#"{"message":{"content":"AKIAIOSFODNN7EXAMPLE ACME-12345678"},"toolUseResult":"AKIAIOSFODNN7EXAMPLE","from":"user@example.com","org":"0f0e0d0c-0000-0000-0000-000000000001"}"#;
    let path = transcript(&dir, &format!("{tool}\n{context}\n{org}\nclean\n"));
    let (redactor, _) = Redactor::new(&["ACME-[0-9]{8}".to_owned()]).unwrap();

    let ingested = ingest_path(&home, &path, &redactor, &mut OsEntropy).unwrap();

    assert_eq!(
        ingested.redactions,
        [
            "aws_access_key",
            "extra_pattern_1",
            "aws_access_key",
            "account_email",
            "organization_id",
            "account_email",
            "organization_id",
        ]
    );
    let store = Store::open(&home.store()).unwrap();
    let log = store.log(ingested.writer).unwrap();
    let contents: Vec<serde_json::Value> = log.iter().map(|e| e.content.clone().unwrap()).collect();
    let all = serde_json::to_string(&contents).unwrap();
    for secret in [
        "AKIAIOSFODNN7EXAMPLE",
        "ACME-12345678",
        "user@example.com",
        "0f0e0d0c-0000",
    ] {
        assert!(!all.contains(secret), "{secret} reached the store");
    }
    assert_eq!(contents[0]["redactions"], 5);
    assert_eq!(contents[3], serde_json::json!({"line": 4, "text": "clean"}));
    let bytes = std::fs::read(home.store()).unwrap();
    assert!(!bytes.windows(20).any(|w| w == b"AKIAIOSFODNN7EXAMPLE"));
}

#[test]
fn an_identifier_too_long_to_redact_records_nothing() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let email = format!("{}@example.com", "a".repeat(1 << 20));
    let line = serde_json::json!({"attachment": {"type": "session_context", "context": {"userEmail": email}}});
    let path = transcript(&dir, &format!("{line}\n"));

    let err = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap_err();

    assert!(matches!(err, IngestError::Redaction(_)));
    assert!(err.to_string().starts_with("redact the transcript: "));
    assert!(std::error::Error::source(&err).is_some());
    assert!(!home.store().exists());
}

/// A recorded Claude Code 2.1.296 transcript: 57 lines, a Bash call and
/// its result among them.
const RECORDED: &str = "testdata/claude-code/2.1.296/transcripts/projects/-w-my-app/686aa14f-b535-45bc-9ed0-5400ea84bec7.jsonl";

#[test]
fn each_line_is_recorded_with_the_class_its_structure_gives() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let text = std::fs::read_to_string(testkit::repo_root().join(RECORDED)).unwrap();
    let path = transcript(&dir, &text);

    let ingested = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    assert_eq!((ingested.events, ingested.unparsed), (57, 0));
    assert_eq!(ingested.drifts, Vec::<String>::new());
    let store = Store::open(&home.store()).unwrap();
    let log = store.log(ingested.writer).unwrap();
    let class = |line: usize| log[line - 1].link.header.provenance.name();
    assert!(
        log.iter()
            .all(|e| e.link.header.kind == EventKind::TranscriptLine)
    );
    assert_eq!(class(1), "harness_meta");
    assert_eq!(class(3), "harness_text");
    assert_eq!(class(6), "harness_text");
    // A prompt is no `user` event until a hook observation records the
    // harness session's process ancestry (PRV-08).
    assert_eq!(class(7), "harness_text");
    assert_eq!(class(20), "assistant");
    assert_eq!(class(21), "tool_call");
    assert_eq!(class(22), "tool_result:Bash");
    assert_eq!(class(57), "harness_meta");
    assert_eq!(
        log[21].content.as_ref().unwrap()["text"],
        text.lines().nth(21).unwrap()
    );
    let device = key::signing::DeviceKey::load(&home.device_key()).unwrap();
    assert!(administration::verify::verify(&store, &device.public()).is_ok());
}

#[test]
fn an_unparsed_or_drifting_line_is_kept_redacted_counted_and_described() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let key = "sk-ant-api03-ABCDEFGHIJKLMNOPQRSTUVWXYZ012345";
    let lines = [
        format!("{{not json {key}"),
        format!(r#"{{"type":"future_kind","secret":"{key}"}}"#),
        format!(r#"{{"type":"assistant","version":"2.1.296","note":"{key}"}}"#),
    ];
    let path = transcript(&dir, &format!("{}\n", lines.join("\n")));

    let ingested = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    assert_eq!((ingested.events, ingested.unparsed), (3, 2));
    assert_eq!(ingested.drifts.len(), 2);
    assert_eq!(
        ingested.drifts[0],
        "claude-code:unversioned:future_kind:unrecorded-type"
    );
    assert!(
        ingested.drifts[1].starts_with("claude-code:2.1.296:assistant:missing:"),
        "{:?}",
        ingested.drifts
    );
    let store = Store::open(&home.store()).unwrap();
    let log = store.log(ingested.writer).unwrap();
    let headers: Vec<(EventKind, Provenance)> = log
        .iter()
        .map(|e| (e.link.header.kind, e.link.header.provenance.clone()))
        .collect();
    assert_eq!(
        headers,
        [
            (EventKind::UnparsedLine, Provenance::Unparsed),
            (EventKind::UnparsedLine, Provenance::Unparsed),
            (EventKind::TranscriptLine, Provenance::HarnessText),
        ]
    );
    for stored in &log {
        let text = stored.content.as_ref().unwrap()["text"].as_str().unwrap();
        assert!(!text.contains(key), "{text}");
        assert!(text.contains("[REDACTED:anthropic_api_key]"), "{text}");
    }
}

#[test]
fn a_drifts_description_passes_the_redactor() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let line = r#"{"type":"assistant","version":"AKIAIOSFODNN7EXAMPLE","message":{}}"#;
    let path = transcript(&dir, &format!("{line}\n"));

    let ingested = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    assert_eq!(
        ingested.drifts,
        ["claude-code:_REDACTED_aws_access_key_:assistant:unsupported-version"]
    );
}

#[test]
fn the_transcripts_under_a_root_are_its_jsonl_files_in_path_order() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("projects");
    std::fs::create_dir_all(root.join("-w-a/s1/subagents")).unwrap();
    std::fs::create_dir_all(root.join("-w-b")).unwrap();
    for file in [
        "-w-b/s2.jsonl",
        "-w-a/s1.jsonl",
        "-w-a/s1/subagents/agent-1.jsonl",
        "-w-a/s1/subagents/agent-1.meta.json",
        "-w-a/notes.txt",
    ] {
        std::fs::write(root.join(file), "x\n").unwrap();
    }

    let found = transcripts(&root).unwrap();

    assert_eq!(
        found,
        // Path order compares component by component: `s1` before
        // `s1.jsonl`.
        [
            root.join("-w-a/s1/subagents/agent-1.jsonl"),
            root.join("-w-a/s1.jsonl"),
            root.join("-w-b/s2.jsonl"),
        ]
    );
    assert_eq!(
        transcripts(&dir.path().join("none")).unwrap(),
        Vec::<std::path::PathBuf>::new()
    );
    let file = root.join("-w-b/s2.jsonl");
    let err = transcripts(&file).unwrap_err();
    assert!(
        err.to_string()
            .starts_with(&format!("list the transcripts in {}: ", file.display())),
        "{err}"
    );
    assert!(std::error::Error::source(&err).is_some());
}

#[cfg(unix)]
#[test]
fn the_transcripts_under_a_root_list_a_linked_transcript_but_descend_no_linked_directory() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("projects");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(dir.path().join("elsewhere")).unwrap();
    std::fs::write(dir.path().join("elsewhere/s.jsonl"), "x\n").unwrap();
    std::os::unix::fs::symlink(dir.path().join("elsewhere"), root.join("dir")).unwrap();
    std::os::unix::fs::symlink(dir.path().join("elsewhere/s.jsonl"), root.join("s.jsonl")).unwrap();

    // The linked transcript is listed as the link; ingest resolves it
    // within the allowed roots (SEC-18). The linked directory, which
    // could loop, is not descended.
    assert_eq!(transcripts(&root).unwrap(), [root.join("s.jsonl")]);
}
