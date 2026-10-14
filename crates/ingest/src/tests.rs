use super::*;
use key::random::OsEntropy;
use record::event::{EventKind, Provenance};
use std::ffi::OsStr;
use testkit::TempDir;

fn home(dir: &TempDir) -> Home {
    Home::locate(Some(dir.path().join(".cairn").as_os_str()), None).unwrap()
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

    let ingested = ingest_path(&home, &path, &mut OsEntropy).unwrap();

    assert_eq!(ingested.events, 2);
    let store = Store::open(&home.store()).unwrap();
    let log = store.log(ingested.writer).unwrap();
    assert_eq!(log.len(), 2);
    for (i, (stored, text)) in log.iter().zip(["{\"a\":1}", "{\"b\":2}"]).enumerate() {
        assert_eq!(stored.link.header.kind, EventKind::UnparsedLine);
        assert_eq!(stored.link.header.provenance, Provenance::Unparsed);
        assert_eq!(stored.content, Some(serde_json::json!({"line": i + 1, "text": text})));
    }
    let seat = store.seat(ingested.writer).unwrap().unwrap();
    assert_eq!((seat.seat, seat.room), (ingested.seat, ingested.room));
    assert!(administration::verify::verify(&store).is_ok());
}

#[test]
fn each_ingest_starts_its_own_seat_in_the_same_personal_room() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let path = transcript(&dir, "x\n");

    let a = ingest_path(&home, &path, &mut OsEntropy).unwrap();
    let b = ingest_path(&home, &path, &mut OsEntropy).unwrap();

    assert_ne!(a.writer, b.writer);
    assert_eq!(a.room, b.room);
}

#[test]
fn an_unreadable_empty_or_non_utf8_transcript_records_nothing() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);

    let missing = ingest_path(&home, &dir.path().join("none"), &mut OsEntropy).unwrap_err();
    assert!(matches!(missing, IngestError::Read { .. }), "{missing}");
    assert!(missing.to_string().starts_with("read the transcript "), "{missing}");
    assert!(std::error::Error::source(&missing).is_some());

    let empty = ingest_path(&home, &transcript(&dir, ""), &mut OsEntropy).unwrap_err();
    assert_eq!(empty.to_string(), format!("the transcript {} holds no line", dir.path().join("t.jsonl").display()));

    std::fs::write(dir.path().join("t.jsonl"), [0xff, b'\n']).unwrap();
    let bytes = ingest_path(&home, &dir.path().join("t.jsonl"), &mut OsEntropy).unwrap_err();
    assert_eq!(bytes.to_string(), format!("the transcript {} is not UTF-8", dir.path().join("t.jsonl").display()));
    assert!(std::error::Error::source(&bytes).is_none());

    assert!(!home.store().exists());
}

#[test]
fn a_home_that_cannot_be_made_is_returned() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("f"), "").unwrap();
    let home = Home::locate(Some(OsStr::new(dir.path().join("f").as_os_str())), None).unwrap();
    let path = transcript(&dir, "x\n");

    let err = ingest_path(&home, &path, &mut OsEntropy).unwrap_err();

    assert!(matches!(err, IngestError::Admin(_)), "{err}");
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn a_store_that_cannot_open_or_a_random_source_that_fails_is_returned() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    home.create().unwrap();
    let path = transcript(&dir, "x\n");
    ingest_path(&home, &path, &mut OsEntropy).unwrap();

    let err = ingest_path(&home, &path, &mut Failing).unwrap_err();
    assert!(matches!(err, IngestError::Admin(_)), "{err}");

    std::fs::write(home.store(), "not a database at all, but long enough to read a header").unwrap();
    let err = ingest_path(&home, &path, &mut OsEntropy).unwrap_err();
    assert!(matches!(err, IngestError::Store(_)), "{err}");
    assert!(std::error::Error::source(&err).is_some());
}

/// A random source that always fails.
struct Failing;

impl key::random::Entropy for Failing {
    fn fill(&mut self, _: &mut [u8]) -> Result<(), key::KeyError> {
        Err(key::KeyError::Entropy("none".into()))
    }
}
