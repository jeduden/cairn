use super::*;
use crate::commitment::Commitment;

fn writer(n: u8) -> WriterId {
    WriterId::derive(&[n; 32])
}

fn seq(n: u64) -> Seq {
    Seq::new(n).unwrap()
}

fn header() -> EventHeader {
    let (a, b, c) = (writer(1), writer(2), writer(3));
    let mut heads = vec![
        SeenHead::new(Address::new(c, seq(4)), Digest::of(&[b"c4"])),
        SeenHead::new(Address::new(b, seq(2)), Digest::of(&[b"b2"])),
    ];
    if b > c {
        heads.reverse();
    }
    EventHeader {
        address: Address::new(a, seq(5)),
        kind: EventKind::UnparsedLine,
        provenance: Provenance::Unparsed,
        prev_hash: Digest::of(&[b"a4"]),
        heads,
        commitment: Commitment::from_digest(Digest::of(&[b"content"])),
    }
}

#[test]
fn the_header_holds_exactly_its_field_set_with_heads_ordered_by_writer() {
    let h = header();
    let mut heads = h.heads.clone();
    heads.sort_by_key(|s| s.address.writer);

    let want = serde_json::json!({
        "address": {"writer": writer(1).to_string(), "seq": 5},
        "kind": "unparsed_line",
        "provenance": "unparsed",
        "prev_hash": h.prev_hash.to_hex(),
        "heads": heads.iter().map(|s| serde_json::json!({
            "writer": s.address.writer.to_string(),
            "seq": s.address.seq.get(),
            "head": s.head.to_hex(),
        })).collect::<Vec<_>>(),
        "commitment": h.commitment.to_string(),
    });
    assert_eq!(h.to_value(), want);
    assert_eq!(
        FIELD_SET,
        "address{writer,seq},kind,provenance,prev_hash,heads[{writer,seq,head}],commitment"
    );
    assert_eq!(ENCODING, "rfc8785+sha256/1");
}

#[test]
fn the_hash_covers_prev_hash_then_the_canonical_header() {
    let mut h = header();
    h.provenance = Provenance::ToolResult("q\"\u{1}é".into());
    let canonical = crate::canonical::encode(&h.to_value()).unwrap();

    assert_eq!(h.canonical(), canonical);
    assert_eq!(
        h.hash(),
        Digest::of(&[h.prev_hash.as_bytes(), canonical.as_bytes()])
    );
}

#[test]
fn changing_any_field_changes_the_hash() {
    let base = header().hash();
    let mut changed = Vec::new();
    let mut h = header();
    h.address.seq = seq(6);
    changed.push(h);
    let mut h = header();
    h.kind = EventKind::KeyEvent;
    changed.push(h);
    let mut h = header();
    h.provenance = Provenance::Structural;
    changed.push(h);
    let mut h = header();
    h.prev_hash = Digest::ZERO;
    changed.push(h);
    let mut h = header();
    h.heads.pop();
    changed.push(h);
    let mut h = header();
    h.commitment = Commitment::from_digest(Digest::ZERO);
    changed.push(h);

    for h in changed {
        assert_ne!(h.hash(), base, "{h:?}");
    }
}
