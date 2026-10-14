use super::*;
use key::random::OsEntropy;
use record::event::{EventKind, Provenance};
use testkit::TempDir;

fn home(dir: &TempDir) -> Home {
    let home = Home::locate(Some(dir.path().join(".cairn").as_os_str()), None).unwrap();
    home.create().unwrap();
    home
}

#[test]
fn the_first_start_mints_the_device_key_and_records_the_personal_room() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();

    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();

    let log = store.log(node.device_writer()).unwrap();
    let kinds: Vec<(EventKind, Provenance)> = log
        .iter()
        .map(|s| (s.link.header.kind, s.link.header.provenance.clone()))
        .collect();
    assert_eq!(
        kinds,
        [
            (EventKind::KeyEvent, Provenance::Structural),
            (EventKind::CreateRoom, Provenance::Operator),
            (EventKind::CertificateRecord, Provenance::Structural),
        ]
    );
    let device = DeviceKey::load(&home.device_key()).unwrap();
    let minted = KeyEvent::from_value(log[0].content.as_ref().unwrap()).unwrap();
    assert!(minted.check_minted(&device.public()).is_ok());
    let seat = store.seat(node.device_writer()).unwrap().unwrap();
    let cert = SeatCertificate::from_value(&seat.certificate).unwrap();
    assert_eq!(
        (cert.seat_kind, cert.ingest_seat, cert.room),
        (SeatKind::Device, false, node.personal_room())
    );
    assert_eq!(
        log[1].content.as_ref().unwrap()["room"],
        node.personal_room().to_string()
    );
    assert_eq!(
        log[2].content.as_ref().unwrap()["certificate"],
        cert.digest().to_hex()
    );
    assert!(home.recovery_key().is_file());
    assert!(home.seat_key(seat.seat).is_file());
}

#[test]
fn the_room_id_derives_from_the_device_seat_key_and_the_recorded_nonce() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();

    let log = store.log(node.device_writer()).unwrap();
    let seat = store.seat(node.device_writer()).unwrap().unwrap();
    let cert = SeatCertificate::from_value(&seat.certificate).unwrap();
    let nonce = log[1].content.as_ref().unwrap()["nonce"]
        .as_str()
        .unwrap()
        .to_owned();

    let mut bytes = [0u8; 16];
    for (i, b) in bytes.iter_mut().enumerate() {
        *b = u8::from_str_radix(&nonce[2 * i..2 * i + 2], 16).unwrap();
    }
    assert_eq!(
        RoomId::derive(cert.seat_key.as_bytes(), &bytes),
        node.personal_room()
    );
}

#[test]
fn a_later_start_finds_the_same_node() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let first = Node::open(&home, &mut store, &mut OsEntropy).unwrap();

    let again = Node::open(&home, &mut store, &mut OsEntropy).unwrap();

    assert_eq!(
        (again.device_writer(), again.personal_room()),
        (first.device_writer(), first.personal_room())
    );
    assert_eq!(again.device().public(), first.device().public());
    assert_eq!(store.log(first.device_writer()).unwrap().len(), 3);
}

#[test]
fn a_device_key_without_its_device_seat_is_refused() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    std::fs::remove_file(home.store()).unwrap();
    let mut empty = Store::open(&home.store()).unwrap();
    // A seat the device key certified, but no device seat.
    let device = DeviceKey::load(&home.device_key()).unwrap();
    let seat = SeatKey::mint(&mut OsEntropy).unwrap();
    let room = RoomId::derive(&[1; 32], &[1; 16]);
    let cert = SeatCertificate::issue(&device, &seat, room, SeatKind::Run, true).unwrap();
    empty.add_seat(&seat_record(&cert)).unwrap();

    let err = Node::open(&home, &mut empty, &mut OsEntropy).unwrap_err();

    assert_eq!(
        err.to_string(),
        "the home holds a device key but the store no device seat it certified"
    );
}

#[test]
fn certify_starts_an_ingest_seat_in_the_personal_room_and_records_its_certificate() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();

    let seat = node
        .certify_ingest_seat(&home, &mut store, &mut OsEntropy)
        .unwrap();

    let cert = SeatCertificate::from_value(&seat.certificate).unwrap();
    assert!(cert.check().is_ok());
    assert_eq!((cert.seat_kind, cert.ingest_seat), (SeatKind::Run, true));
    assert_eq!(
        (seat.room, cert.certifying_key),
        (node.personal_room(), node.device().public())
    );
    assert_eq!(store.seat(seat.writer).unwrap(), Some(seat.clone()));
    assert!(SeatKey::load(&home.seat_key(seat.seat)).is_ok());
    let log = store.log(node.device_writer()).unwrap();
    assert_eq!(log.len(), 4);
    assert_eq!(log[3].link.header.kind, EventKind::CertificateRecord);
    assert_eq!(
        log[3].content.as_ref().unwrap()["certificate"],
        cert.digest().to_hex()
    );
    assert_eq!(
        log[3].content.as_ref().unwrap()["writer"],
        seat.writer.to_string()
    );
}

#[test]
fn a_failure_on_the_way_is_returned() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    std::fs::write(home.device_key(), "x").unwrap();

    assert!(matches!(
        Node::open(&home, &mut store, &mut OsEntropy),
        Err(AdminError::Key(_))
    ));
    assert!(matches!(
        Node::open(&home, &mut store, &mut Failing),
        Err(AdminError::Key(_))
    ));
}

/// A random source that always fails.
struct Failing;

impl key::random::Entropy for Failing {
    fn fill(&mut self, _: &mut [u8]) -> Result<(), key::KeyError> {
        Err(key::KeyError::Entropy("none".into()))
    }
}

#[test]
fn a_store_that_fails_mid_start_is_returned() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    // A seat certificate stored by hand under an unparseable value
    // makes the device seat unreadable.
    let conn = rusqlite::Connection::open(home.store()).unwrap();
    conn.execute("UPDATE seats SET certificate = '{}'", [])
        .unwrap();

    assert!(matches!(
        Node::open(&home, &mut store, &mut OsEntropy),
        Err(AdminError::Key(_))
    ));
    conn.execute("UPDATE seats SET certificate = '{'", [])
        .unwrap();
    assert!(matches!(
        Node::open(&home, &mut store, &mut OsEntropy),
        Err(AdminError::Store(_))
    ));
    assert!(matches!(
        node.certify_ingest_seat(&home, &mut store, &mut Failing),
        Err(AdminError::Key(_))
    ));
}
