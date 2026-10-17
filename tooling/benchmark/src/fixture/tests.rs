use super::*;
use testkit::TempDir;

#[test]
fn a_transcript_repeats_the_recorded_lines_each_copy_with_its_own_uuids() {
    let recorded = RECORDED.lines().count();

    let text = transcript(recorded + 2);

    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), recorded + 2);
    let first_uuid = RECORDED
        .lines()
        .find(|l| l.contains("\"uuid\":\""))
        .unwrap();
    let at = RECORDED.lines().position(|l| l == first_uuid).unwrap();
    assert!(lines[at].contains("\"uuid\":\"c0-"));
    assert!(text.ends_with('\n'));
    assert!(transcript(0).is_empty());
}

#[test]
fn a_home_holds_its_transcript_and_is_removed_when_dropped() {
    let base = TempDir::new().unwrap();

    let home = Home::with_transcript(base.path(), "x\n").unwrap();
    let [(home_var, root), (cairn_var, cairn)] = home.env();

    assert_eq!((home_var, cairn_var), ("HOME", "CAIRN_HOME"));
    assert_eq!(cairn, root.join(".cairn"));
    let placed = root.join(".claude/projects/-w-bench/bench.jsonl");
    assert_eq!(fs::read_to_string(placed).unwrap(), "x\n");
    let other = Home::with_transcript(base.path(), "y\n").unwrap();
    assert_ne!(other.env()[0].1, root);
    drop(home);
    assert!(!root.exists());
    assert!(Home::with_transcript(&base.path().join("f\0"), "").is_err());
}
