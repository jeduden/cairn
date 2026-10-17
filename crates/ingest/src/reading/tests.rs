use super::*;

fn writer(n: u8) -> WriterId {
    WriterId::derive(&[n; 32])
}

#[test]
fn a_transaction_holds_one_writer_s_consecutive_lines() {
    let (a, b) = (writer(1), writer(2));
    let ends = [
        (a, false, 1),
        (a, true, 2),
        (a, false, 3),
        (b, false, 4),
        (a, false, 5),
    ];

    assert_eq!(transactions(&ends), [0..3, 3..4, 4..5]);
    assert_eq!(last_of_each_writer(&ends)[&a], 4);
    assert_eq!(last_of_each_writer(&ends)[&b], 3);
    assert!(transactions(&[]).is_empty());
}

#[test]
fn a_transaction_holds_at_most_a_batch() {
    let ends: Vec<_> = (0..250).map(|i| (writer(1), false, i)).collect();

    assert_eq!(transactions(&ends), [0..100, 100..200, 200..250]);
}

#[test]
fn a_source_continues_while_its_consumed_prefix_stands() {
    let position = Position {
        offset: 2,
        prefix: Digest::of(&[b"ab"]),
        lines: 1,
        seen: 2,
    };

    assert!(continues(&position, b"abc"));
    assert!(!continues(&position, b"xbc"));
    assert!(!continues(&position, b"a"));
    assert_eq!(as_index(3), 3);
}

#[test]
fn the_directory_s_other_transcripts_give_their_uuids() {
    let dir = testkit::TempDir::new().unwrap();
    let own = dir.path().join("own.jsonl");
    std::fs::write(&own, "{\"uuid\":\"mine\"}\n").unwrap();
    std::fs::write(
        dir.path().join("other.jsonl"),
        "{\"uuid\":\"u1\"}\nnot json\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("notes.txt"), "{\"uuid\":\"u2\"}\n").unwrap();
    std::fs::create_dir(dir.path().join("sub.jsonl")).unwrap();

    let found = sibling_uuids(&dir.path().to_string_lossy(), &own.to_string_lossy());

    assert_eq!(found, BTreeSet::from(["u1".to_owned()]));
    assert!(sibling_uuids(&dir.path().join("none").to_string_lossy(), "").is_empty());
}
