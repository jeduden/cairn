//! NFR-09's store overhead, the evidence that freezes it (§12.3): every
//! recorded Claude Code transcript ingested into one fresh home takes
//! at most 1.5 times its stored text in the store, `store.db` with its
//! journal and the payload store together. The stored text is what the
//! record keeps of each line: its text after redaction.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::fs;
use std::path::{Path, PathBuf};

use administration::home::Home;
use key::random::OsEntropy;
use record::event::EventKind;
use redaction::Redactor;
use store::Store;
use testkit::TempDir;

/// NFR-09's bound on the store's size over its stored text.
const OVERHEAD: f64 = 1.5;

/// Every `.jsonl` file under `dir`.
fn transcripts(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            transcripts(&path, found);
        } else if path.extension().is_some_and(|e| e == "jsonl") {
            found.push(path);
        }
    }
}

/// The bytes every file under `dir` holds.
fn bytes(dir: &Path) -> u64 {
    if !dir.exists() {
        return 0;
    }
    fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .map(|p| {
            if p.is_dir() {
                bytes(&p)
            } else {
                fs::metadata(&p).unwrap().len()
            }
        })
        .sum()
}

#[test]
fn the_store_holds_every_recorded_transcript_within_1_5_times_its_stored_text() {
    let dir = TempDir::new().unwrap();
    let home = Home::locate(Some(dir.path().join(".cairn").as_os_str()), None).unwrap();
    let mut found = Vec::new();
    transcripts(
        &testkit::repo_root().join("testdata/claude-code/2.1.296/transcripts"),
        &mut found,
    );
    found.sort();
    let redactor = Redactor::new(&[]).unwrap().0;
    let mut writers = Vec::new();
    for path in &found {
        writers.push(
            ingest::ingest_path(&home, path, &redactor, &mut OsEntropy)
                .unwrap()
                .writer,
        );
    }

    let store = Store::open(&home.store()).unwrap();
    let mut text = 0;
    for writer in writers {
        for stored in store.log(writer).unwrap() {
            assert_ne!(stored.link.header.kind, EventKind::KeyEvent);
            text += stored.content.unwrap()["text"].as_str().unwrap().len() as u64;
        }
    }
    drop(store);
    let held = ["store.db", "store.db-wal", "store.db-shm"]
        .iter()
        .map(|f| fs::metadata(home.root().join(f)).map_or(0, |m| m.len()))
        .sum::<u64>()
        + bytes(&home.root().join("payloads"));

    let overhead = held as f64 / text as f64;
    println!(
        "{} transcripts: {text} bytes of stored text in {held} bytes of store, {overhead:.3} times",
        found.len()
    );
    assert!(
        overhead <= OVERHEAD,
        "the store holds {held} bytes for {text} bytes of stored text: {overhead:.3} times"
    );
}
