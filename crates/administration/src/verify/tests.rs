use super::*;
use crate::home::Home;
use crate::node::Node;
use key::random::OsEntropy;
use key::signing::{DeviceKey, SeatKey};
use record::commitment::CommitmentKey;
use record::event::{EventKind, Provenance};
use record::id::Seq;
use serde_json::json;
use store::{NewEvent, StoredSeat};
use testkit::TempDir;

struct Fixture {
    _dir: TempDir,
    home: Home,
    store: Store,
    node: Node,
    ingest: StoredSeat,
}

fn new_event(n: u8) -> NewEvent {
    NewEvent {
        kind: EventKind::UnparsedLine,
        provenance: Provenance::Unparsed,
        content: json!({"line": n, "text": format!("line {n}")}),
        commitment_key: CommitmentKey::from_bytes([n; 32]),
    }
}

/// A node with one ingest seat holding three events.
fn fixture() -> Fixture {
    let dir = TempDir::new().unwrap();
    let home = Home::locate(Some(dir.path().join(".cairn").as_os_str()), None).unwrap();
    home.create().unwrap();
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let ingest = node
        .certify_ingest_seat(&home, &mut store, &mut OsEntropy)
        .unwrap();
    store
        .append(ingest.writer, &[new_event(1), new_event(2), new_event(3)])
        .unwrap();
    Fixture {
        _dir: dir,
        home,
        store,
        node,
        ingest,
    }
}

impl Fixture {
    fn sql(&self, statement: &str) {
        let conn = rusqlite::Connection::open(self.home.store()).unwrap();
        conn.execute(statement, []).unwrap();
    }

    fn at(&self, seq: u64) -> Address {
        Address::new(self.ingest.writer, Seq::new(seq).unwrap())
    }
}

#[test]
fn an_intact_record_verifies_and_counts_its_writers_and_events() {
    let f = fixture();

    let report = verify(&f.store).unwrap();

    assert_eq!(
        report,
        Report {
            writers: 2,
            events: 7
        }
    );
}

#[test]
fn a_changed_header_byte_is_named_by_its_full_address() {
    let f = fixture();
    let stored = f.store.event(f.at(2)).unwrap().unwrap();
    let mut flipped = stored.link.header.commitment.to_string();
    let last = if flipped.ends_with('0') { "1" } else { "0" };
    flipped.replace_range(63.., last);
    f.sql(&format!(
        "UPDATE events SET commitment = '{flipped}' WHERE seq = 2 AND writer = '{}'",
        f.ingest.writer
    ));

    let err = verify(&f.store).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "chain broken at {}: the stored hash does not match the event's header",
            f.at(2).full(&f.node.personal_room())
        )
    );
}

#[test]
fn content_that_does_not_open_its_commitment_is_named() {
    let f = fixture();
    f.sql(&format!(
        "UPDATE events SET content = '{{\"line\":3,\"text\":\"other\"}}' WHERE seq = 3 AND writer = '{}'",
        f.ingest.writer
    ));

    let err = verify(&f.store).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "chain broken at {}: the content does not open its commitment",
            f.at(3).full(&f.node.personal_room())
        )
    );
}

#[test]
fn content_removed_with_no_tombstone_is_named() {
    // Purge, which records a tombstone for what it removes, lands with
    // ADM-07; until then removed content is a loss verify reports (I1).
    let f = fixture();
    f.sql(&format!(
        "UPDATE events SET content = NULL, commitment_key = NULL WHERE seq = 3 AND writer = '{}'",
        f.ingest.writer
    ));

    let err = verify(&f.store).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "chain broken at {}: its content was removed, and no tombstone records it",
            f.at(3).full(&f.node.personal_room())
        )
    );
}

#[test]
fn a_commitment_key_without_its_content_does_not_open_its_commitment() {
    let f = fixture();
    f.sql(&format!(
        "UPDATE events SET content = NULL WHERE seq = 3 AND writer = '{}'",
        f.ingest.writer
    ));

    let err = verify(&f.store).unwrap_err();

    assert!(
        err.to_string()
            .ends_with("the content does not open its commitment"),
        "{err}"
    );
}

#[test]
fn a_row_that_does_not_parse_is_named() {
    let f = fixture();
    f.sql(&format!(
        "UPDATE events SET kind = 'x' WHERE seq = 2 AND writer = '{}'",
        f.ingest.writer
    ));

    let err = verify(&f.store).unwrap_err();

    assert!(
        err.to_string().starts_with(&format!(
            "chain broken at {}: ",
            f.at(2).full(&f.node.personal_room())
        )),
        "{err}"
    );
}

#[test]
fn a_changed_cited_head_changes_the_hash_of_the_event_citing_it() {
    let f = fixture();
    f.sql("UPDATE event_heads SET other_seq = 1");

    let err = verify(&f.store).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "chain broken at {}: the stored hash does not match the event's header",
            f.at(1).full(&f.node.personal_room())
        )
    );
}

#[test]
fn a_cited_head_naming_no_event_is_named_even_when_the_chain_is_rehashed() {
    let f = fixture();
    // The ingest seat's first event cited the device seat's fourth
    // event; point it at the first, and rehash the ingest seat's chain
    // after it so only the cited head is wrong.
    f.sql("UPDATE event_heads SET other_seq = 1");
    let mut prev = Digest::ZERO;
    for stored in f.store.log(f.ingest.writer).unwrap() {
        let mut header = stored.link.header.clone();
        header.prev_hash = prev;
        prev = header.hash();
        f.sql(&format!(
            "UPDATE events SET prev_hash = '{}', hash = '{prev}' WHERE writer = '{}' AND seq = {}",
            header.prev_hash, f.ingest.writer, header.address.seq
        ));
    }

    let err = verify(&f.store).unwrap_err();

    assert!(
        err.to_string()
            .ends_with("which no event of another writer matches"),
        "{err}"
    );
}

#[test]
fn a_writer_without_a_seat_certificate_is_named() {
    let mut f = fixture();
    let stray = WriterId::derive(&[9; 32]);
    f.store.append(stray, &[new_event(1)]).unwrap();

    let err = verify(&f.store).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!("writer {stray}: no seat certificate names it")
    );
}

#[test]
fn a_changed_seat_certificate_is_named() {
    let f = fixture();
    let mut cert = f.ingest.certificate.clone();
    cert["ingest_seat"] = json!(false);
    f.sql(&format!(
        "UPDATE seats SET certificate = '{cert}' WHERE writer = '{}'",
        f.ingest.writer
    ));

    let err = verify(&f.store).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "writer {}: its seat certificate: a seat_certificate signature does not verify",
            f.ingest.writer
        )
    );
}

#[test]
fn a_seat_certificate_naming_another_seat_or_room_is_named() {
    let f = fixture();
    let other = RoomId::derive(&[1; 32], &[1; 16]);
    f.sql(&format!(
        "UPDATE seats SET room = '{other}' WHERE writer = '{}'",
        f.ingest.writer
    ));

    let err = verify(&f.store).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "writer {}: its seat certificate names another seat",
            f.ingest.writer
        )
    );
}

#[test]
fn a_seat_certificate_no_certificate_record_holds_is_named() {
    let mut f = fixture();
    let device = DeviceKey::load(&f.home.device_key()).unwrap();
    let seat = SeatKey::mint(&mut OsEntropy).unwrap();
    let cert = SeatCertificate::issue(&device, &seat, f.node.personal_room(), SeatKind::Run, true)
        .unwrap();
    let record = StoredSeat {
        writer: cert.writer_id(),
        seat: cert.seat_id(),
        room: cert.room,
        certificate: cert.to_value(),
    };
    f.store.add_seat(&record).unwrap();

    let err = verify(&f.store).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "writer {}: no device seat holds a certificate record of its seat certificate",
            record.writer
        )
    );
}

#[test]
fn a_seat_certificate_from_a_key_with_no_minted_key_event_is_named() {
    let mut f = fixture();
    let stranger = DeviceKey::mint(&mut OsEntropy).unwrap();
    let seat = SeatKey::mint(&mut OsEntropy).unwrap();
    let cert = SeatCertificate::issue(
        &stranger,
        &seat,
        f.node.personal_room(),
        SeatKind::Run,
        true,
    )
    .unwrap();
    let record = StoredSeat {
        writer: cert.writer_id(),
        seat: cert.seat_id(),
        room: cert.room,
        certificate: cert.to_value(),
    };
    f.store.add_seat(&record).unwrap();

    let err = verify(&f.store).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "writer {}: no minted key event of this node names its certifying_key",
            record.writer
        )
    );
}

#[test]
fn a_removed_key_event_is_named_as_lost() {
    let f = fixture();
    let device = f.node.device_writer();
    f.sql(&format!("UPDATE events SET content = NULL, commitment_key = NULL WHERE seq = 1 AND writer = '{device}'"));

    let err = verify(&f.store).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "chain broken at {}: its content was removed, and no tombstone records it",
            Address::new(device, Seq::FIRST).full(&f.node.personal_room())
        )
    );
}

#[test]
fn a_seat_certificate_that_does_not_parse_is_named() {
    let f = fixture();
    f.sql(&format!(
        "UPDATE seats SET certificate = '{{}}' WHERE writer = '{}'",
        f.ingest.writer
    ));
    assert!(
        verify(&f.store)
            .unwrap_err()
            .to_string()
            .contains("its seat certificate: malformed")
    );
}

#[test]
fn a_store_error_is_passed_on() {
    let f = fixture();
    f.sql("UPDATE seats SET room = 'x'");

    assert!(matches!(verify(&f.store), Err(VerifyError::Store(_))));
    let broken = VerifyError::Broken {
        address: "a".into(),
        reason: "r".into(),
    };
    assert!(std::error::Error::source(&broken).is_none());
    let err = VerifyError::Store(store::StoreError::Full(f.ingest.writer));
    assert!(std::error::Error::source(&err).is_some());
    assert!(err.to_string().contains("no seq left"));
}

#[test]
fn a_key_event_whose_content_is_no_key_event_is_named() {
    let mut f = fixture();
    let device = f.node.device_writer();
    f.store
        .append(
            device,
            &[NewEvent {
                kind: EventKind::KeyEvent,
                provenance: Provenance::Structural,
                content: json!({}),
                commitment_key: CommitmentKey::from_bytes([1; 32]),
            }],
        )
        .unwrap();

    let err = verify(&f.store).unwrap_err();

    assert!(
        err.to_string()
            .contains("does not check: malformed key event"),
        "{err}"
    );
}

#[test]
fn content_without_its_commitment_key_does_not_open_its_commitment() {
    let f = fixture();
    f.sql("UPDATE events SET commitment_key = NULL WHERE seq = 3");

    let err = verify(&f.store).unwrap_err();

    assert!(
        err.to_string()
            .ends_with("the content does not open its commitment"),
        "{err}"
    );
}

#[test]
fn a_store_error_reading_a_log_is_passed_on() {
    let f = fixture();
    f.sql(&format!(
        "UPDATE events SET seq = -1 WHERE seq = 3 AND writer = '{}'",
        f.ingest.writer
    ));

    assert!(matches!(verify(&f.store), Err(VerifyError::Store(_))));
}
