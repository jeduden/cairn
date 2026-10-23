use std::fs;

use super::*;
use crate::home::Home;
use crate::node::Node;
use crate::node_identity::IdentitySource;
use key::random::OsEntropy;
use key::signing::SeatKey;
use record::id::RoomId;
use testkit::TempDir;

fn home(dir: &TempDir) -> Home {
    fs::write(dir.path().join("machine-id"), "first\n").unwrap();
    let home = Home::locate(Some(dir.path().join(".parinn").as_os_str()), None)
        .unwrap()
        .with_identity(IdentitySource::Files(vec![dir.path().join("machine-id")]));
    home.create().unwrap();
    home
}

fn origin(store: &Store, writer: WriterId) -> Origin {
    derive(store).unwrap()[&writer]
}

#[test]
fn the_device_seat_and_a_run_seat_a_key_of_this_node_certified_are_witnessed() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let key = SeatKey::mint(&mut OsEntropy).unwrap();
    let run = node
        .certify_run_seat(&mut store, &key, &mut OsEntropy)
        .unwrap();

    assert_eq!(origin(&store, node.device_writer()), Origin::Witnessed);
    assert_eq!(origin(&store, run.writer), Origin::Witnessed);
}

#[test]
fn an_ingest_seat_reads_ingested() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let ingest = node
        .certify_ingest_seat(&home, &mut store, &mut OsEntropy)
        .unwrap();

    assert_eq!(origin(&store, ingest.writer), Origin::Ingested);
}

#[test]
fn a_seat_whose_certificate_record_the_device_seat_lacks_reads_peer() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let key = SeatKey::mint(&mut OsEntropy).unwrap();
    let cert = SeatCertificate::issue(
        node.device(),
        &key,
        node.personal_room(),
        SeatKind::Run,
        false,
    )
    .unwrap();
    let seat = StoredSeat {
        writer: cert.writer_id(),
        seat: cert.seat_id(),
        room: cert.room,
        certificate: cert.to_value(),
    };
    store.add_seat(&seat).unwrap();

    assert_eq!(origin(&store, seat.writer), Origin::Peer);
}

#[test]
fn a_seat_another_key_certified_reads_peer() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let other = key::signing::DeviceKey::mint(&mut OsEntropy).unwrap();
    let key = SeatKey::mint(&mut OsEntropy).unwrap();
    let cert =
        SeatCertificate::issue(&other, &key, node.personal_room(), SeatKind::Run, false).unwrap();
    let seat = StoredSeat {
        writer: cert.writer_id(),
        seat: cert.seat_id(),
        room: cert.room,
        certificate: cert.to_value(),
    };
    store.add_seat(&seat).unwrap();

    assert_eq!(origin(&store, seat.writer), Origin::Peer);
}

#[test]
fn a_seat_whose_certificate_does_not_read_or_names_another_writer_reads_peer() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let mut seat = store.seat(node.device_writer()).unwrap().unwrap();
    let keys = KeySet::load(&store).unwrap();
    assert_eq!(keys.origin_of(&seat), Origin::Witnessed);

    seat.room = RoomId::derive(&[9; 32], &[9; 16]);
    assert_eq!(keys.origin_of(&seat), Origin::Peer);
    seat.certificate = serde_json::json!({"unrelated": true});
    assert_eq!(keys.origin_of(&seat), Origin::Peer);
}

#[test]
fn after_a_node_identity_change_no_writer_reads_witnessed_or_ingested() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let old = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let ingest = old
        .certify_ingest_seat(&home, &mut store, &mut OsEntropy)
        .unwrap();
    fs::write(dir.path().join("machine-id"), "second\n").unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let key = SeatKey::mint(&mut OsEntropy).unwrap();
    let run = node
        .certify_run_seat(&mut store, &key, &mut OsEntropy)
        .unwrap();

    let origins = derive(&store).unwrap();

    assert_eq!(origins.len(), 4);
    assert!(origins.values().all(|o| *o == Origin::Peer));
    assert_eq!(origins[&old.device_writer()], Origin::Peer);
    assert_eq!(origins[&ingest.writer], Origin::Peer);
    assert_eq!(origins[&run.writer], Origin::Peer);
}

#[test]
fn the_origins_names_are_fixed() {
    assert_eq!(
        [
            Origin::Witnessed,
            Origin::Ingested,
            Origin::Bundle,
            Origin::Peer
        ]
        .map(Origin::as_str),
        ["witnessed", "ingested", "bundle", "peer"]
    );
}
