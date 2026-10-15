use std::fs;
use std::io::Write as _;

use record::digest::Digest;
use testkit::TempDir;

use super::*;
use crate::entry::{Counter, Kind, Occurrence};

const FAILED: Occurrence = Occurrence::Degraded {
    kind: Kind::Failed,
    counter: Counter::HookPanicked,
    operation: "hook SessionStart",
    detail: "panic recovered",
};

fn log(dir: &TempDir) -> Log {
    Log::new(dir.path().join("audit"))
}

#[cfg(unix)]
fn mode(path: &std::path::Path) -> u32 {
    use std::os::unix::fs::PermissionsExt as _;
    fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[test]
fn files_are_named_by_their_six_digit_number() {
    let log = Log::new(PathBuf::from("/h/audit"));
    assert_eq!(log.dir(), Path::new("/h/audit"));
    assert_eq!(log.file(1), PathBuf::from("/h/audit/audit-000001.jsonl"));
    assert_eq!(log.file(42), PathBuf::from("/h/audit/audit-000042.jsonl"));
    assert_eq!(file_number("audit-000042.jsonl"), Some(42));
    for name in [
        "audit-42.jsonl",
        "audit-00004x.jsonl",
        "lock",
        "audit-000000.jsonl",
    ] {
        assert_eq!(file_number(name), None, "{name}");
    }
}

#[test]
fn the_first_append_makes_the_log_owner_only_and_chains_from_zero() {
    let dir = TempDir::new().unwrap();
    let log = log(&dir);
    let first = log.append(&FAILED, 100).unwrap();
    assert_eq!(first.number, 1);
    assert_eq!(first.prev_hash, Digest::ZERO);
    let second = log.append(&Occurrence::CounterAck, 101).unwrap();
    assert_eq!(second.number, 2);
    assert_eq!(second.prev_hash, first.hash);
    let text = fs::read_to_string(log.file(1)).unwrap();
    assert_eq!(
        text,
        format!("{}\n{}\n", first.line().unwrap(), second.line().unwrap())
    );
    #[cfg(unix)]
    {
        assert_eq!(mode(log.dir()), 0o700);
        assert_eq!(mode(&log.file(1)), 0o600);
        assert_eq!(mode(&log.dir().join("lock")), 0o600);
    }
    assert_eq!(log.entries().unwrap(), vec![first, second]);
}

#[test]
fn a_file_that_reached_the_rotation_size_continues_the_chain_in_the_next() {
    let dir = TempDir::new().unwrap();
    let log = log(&dir);
    let last = log.append(&FAILED, 1).unwrap();
    let padding = usize::try_from(ROTATE_AT).unwrap();
    // Grow the first file to the rotation size with whole entries.
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(log.file(1))
        .unwrap();
    let mut prev = last;
    let mut written = fs::metadata(log.file(1)).unwrap().len();
    while written < ROTATE_AT {
        let e = Entry::next(&FAILED, prev.number + 1, 2, prev.hash).unwrap();
        let line = format!("{}\n", e.line().unwrap());
        file.write_all(line.as_bytes()).unwrap();
        written += line.len() as u64;
        prev = e;
    }
    assert!(usize::try_from(written).unwrap() >= padding);
    let next = log.append(&Occurrence::CounterAck, 3).unwrap();
    assert_eq!(next.number, prev.number + 1);
    assert_eq!(next.prev_hash, prev.hash);
    assert_eq!(
        fs::read_to_string(log.file(2)).unwrap(),
        format!("{}\n", next.line().unwrap())
    );
    #[cfg(unix)]
    assert_eq!(mode(&log.file(2)), 0o600);
    let checked = log.verify().unwrap();
    assert_eq!(checked.files, 2);
    assert_eq!(checked.entries, next.number);
}

#[test]
fn an_empty_last_file_takes_the_next_entry_chained_to_the_file_before() {
    let dir = TempDir::new().unwrap();
    let log = log(&dir);
    let first = log.append(&FAILED, 1).unwrap();
    fs::write(log.file(2), "").unwrap();
    let next = log.append(&FAILED, 2).unwrap();
    assert_eq!((next.number, next.prev_hash), (2, first.hash));
    assert_eq!(
        fs::read_to_string(log.file(2)).unwrap(),
        format!("{}\n", next.line().unwrap())
    );
    assert_eq!(log.verify().unwrap().entries, 2);
}

#[test]
fn a_log_with_no_directory_holds_no_entry() {
    let dir = TempDir::new().unwrap();
    let log = log(&dir);
    assert_eq!(log.entries().unwrap(), Vec::new());
    let checked = log.verify().unwrap();
    assert_eq!((checked.files, checked.entries), (0, 0));
}

#[test]
fn other_files_in_the_directory_are_not_the_log() {
    let dir = TempDir::new().unwrap();
    let log = log(&dir);
    log.append(&FAILED, 1).unwrap();
    fs::write(log.dir().join("notes.txt"), "x").unwrap();
    assert_eq!(log.verify().unwrap().files, 1);
}

#[test]
fn verify_names_the_entry_that_breaks_the_chain() {
    let dir = TempDir::new().unwrap();
    let log = log(&dir);
    for t in 0..3 {
        log.append(&FAILED, t).unwrap();
    }
    let text = fs::read_to_string(log.file(1)).unwrap();
    let broken = text.replacen("panic recovered", "panic recovereD", 1);
    fs::write(log.file(1), broken).unwrap();
    let e = log.verify().unwrap_err();
    assert_eq!(
        e.to_string(),
        "audit log audit-000001.jsonl line 1: hash does not match"
    );
}

#[test]
fn verify_refuses_entries_out_of_order_or_off_the_chain() {
    let dir = TempDir::new().unwrap();
    let log = log(&dir);
    let a = Entry::next(&FAILED, 1, 0, Digest::ZERO).unwrap();
    let skipped = Entry::next(&FAILED, 3, 0, a.hash).unwrap();
    fs::create_dir_all(log.dir()).unwrap();
    fs::write(
        log.file(1),
        format!("{}\n{}\n", a.line().unwrap(), skipped.line().unwrap()),
    )
    .unwrap();
    assert_eq!(
        log.verify().unwrap_err().to_string(),
        "audit log audit-000001.jsonl line 2: entry number out of order"
    );

    let off = Entry::next(&FAILED, 2, 0, Digest::ZERO).unwrap();
    fs::write(
        log.file(1),
        format!("{}\n{}\n", a.line().unwrap(), off.line().unwrap()),
    )
    .unwrap();
    assert_eq!(
        log.verify().unwrap_err().to_string(),
        "audit log audit-000001.jsonl line 2: previous hash is not the entry before"
    );
}

#[test]
fn verify_refuses_a_missing_file_an_empty_one_and_a_torn_entry() {
    let dir = TempDir::new().unwrap();
    let log = log(&dir);
    log.append(&FAILED, 0).unwrap();
    fs::write(log.file(3), "").unwrap();
    assert_eq!(
        log.verify().unwrap_err().to_string(),
        "audit log audit-000002.jsonl: missing"
    );
    fs::write(log.file(2), "").unwrap();
    assert_eq!(
        log.verify().unwrap_err().to_string(),
        "audit log audit-000002.jsonl: empty but not the last file"
    );
    fs::remove_file(log.file(3)).unwrap();
    fs::remove_file(log.file(2)).unwrap();
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(log.file(1))
        .unwrap();
    file.write_all(b"{\"entry\":").unwrap();
    assert_eq!(
        log.verify().unwrap_err().to_string(),
        "audit log audit-000001.jsonl line 2: torn entry"
    );
    let append = log.append(&FAILED, 1).unwrap_err();
    assert_eq!(
        append.to_string(),
        "audit log audit-000001.jsonl line 2: torn entry"
    );
}

#[test]
fn a_file_that_does_not_start_the_log_must_not_start_the_chain() {
    let dir = TempDir::new().unwrap();
    let log = log(&dir);
    fs::create_dir_all(log.dir()).unwrap();
    let late = Entry::next(&FAILED, 1, 0, Digest::ZERO).unwrap();
    fs::write(log.file(2), format!("{}\n", late.line().unwrap())).unwrap();
    assert_eq!(
        log.verify().unwrap_err().to_string(),
        "audit log audit-000001.jsonl: missing"
    );
}

#[test]
fn a_held_lock_makes_the_append_busy_after_its_deadline() {
    let dir = TempDir::new().unwrap();
    let log = log(&dir);
    log.append(&FAILED, 0).unwrap();
    let held = fs::File::open(log.dir().join("lock")).unwrap();
    held.try_lock().unwrap();
    let e = log.append(&FAILED, 1).unwrap_err();
    assert!(matches!(e, AuditError::Busy { .. }), "{e:?}");
    drop(held);
    log.append(&FAILED, 2).unwrap();
}

#[test]
fn a_directory_that_cannot_be_made_or_read_is_an_io_failure() {
    let dir = TempDir::new().unwrap();
    let blocker = dir.path().join("audit");
    fs::write(&blocker, "a file").unwrap();
    let log = Log::new(blocker.join("inner"));
    assert!(matches!(log.append(&FAILED, 0), Err(AuditError::Io { .. })));
    let log = Log::new(blocker);
    assert!(matches!(log.entries(), Err(AuditError::Io { .. })));
}

#[test]
fn a_log_file_that_is_a_directory_cannot_be_read() {
    let dir = TempDir::new().unwrap();
    let log = log(&dir);
    fs::create_dir_all(log.file(1)).unwrap();
    assert!(matches!(log.entries(), Err(AuditError::Io { .. })));
    assert!(matches!(log.append(&FAILED, 0), Err(AuditError::Io { .. })));
}

#[test]
fn a_lock_that_cannot_be_opened_is_an_io_failure() {
    let dir = TempDir::new().unwrap();
    let log = log(&dir);
    fs::create_dir_all(log.dir().join("lock")).unwrap();
    assert!(matches!(log.append(&FAILED, 0), Err(AuditError::Io { .. })));
}
