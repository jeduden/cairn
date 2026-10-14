use super::*;
use crate::random::Counted;
use record::id::RoomId;

fn keys() -> (DeviceKey, SeatKey, RoomId) {
    let mut e = Counted::default();
    let device = DeviceKey::mint(&mut e).unwrap();
    let seat = SeatKey::mint(&mut e).unwrap();
    let room = RoomId::derive(device.public().as_bytes(), &[9; 16]);
    (device, seat, room)
}

#[test]
fn an_issued_certificate_checks_and_names_its_seat_and_writer() {
    let (device, seat, room) = keys();

    let cert = SeatCertificate::issue(&device, &seat, room, SeatKind::Run, true).unwrap();

    assert!(cert.check().is_ok());
    assert_eq!(cert.seat_id(), SeatId::derive(&room, seat.public().as_bytes()));
    assert_eq!(cert.writer_id(), WriterId::derive(seat.public().as_bytes()));
    assert_eq!(cert.certifier, device.public());
    assert!(cert.ingest_seat);
}

#[test]
fn a_certificate_round_trips_through_its_value() {
    let (device, seat, room) = keys();
    for (kind, ingest) in [(SeatKind::Device, false), (SeatKind::Run, true)] {
        let cert = SeatCertificate::issue(&device, &seat, room, kind, ingest).unwrap();

        assert_eq!(SeatCertificate::from_value(&cert.to_value()).unwrap(), cert);
        assert_eq!(cert.digest(), Digest::of(&[record::canonical::encode(&cert.to_value()).unwrap().as_bytes()]));
    }
}

#[test]
fn a_changed_field_or_signature_fails_the_check() {
    let (device, seat, room) = keys();
    let cert = SeatCertificate::issue(&device, &seat, room, SeatKind::Run, true).unwrap();
    let other = SeatKey::mint(&mut Counted::default()).unwrap();

    let mut c = cert.clone();
    c.ingest_seat = false;
    assert!(c.check().is_err());
    let mut c = cert.clone();
    c.seat_kind = SeatKind::Device;
    assert!(c.check().is_err());
    let mut c = cert.clone();
    c.seat_key = other.public();
    assert!(c.check().is_err());
    let mut c = cert.clone();
    c.countersignature = c.signature;
    assert!(c.check().is_err());
}

#[test]
fn a_malformed_value_is_refused_naming_the_field() {
    let (device, seat, room) = keys();
    let good = SeatCertificate::issue(&device, &seat, room, SeatKind::Run, true)
        .unwrap()
        .to_value();
    for (field, bad) in [
        ("room", serde_json::json!("x")),
        ("seat_key", serde_json::json!(1)),
        ("seat_kind", serde_json::json!("ingest")),
        ("ingest_seat", serde_json::json!("yes")),
        ("certifier", serde_json::json!("x")),
        ("signature", serde_json::json!("x")),
        ("countersignature", serde_json::json!(null)),
    ] {
        let mut v = good.clone();
        v[field] = bad;
        match SeatCertificate::from_value(&v) {
            Err(KeyError::Malformed(what)) => assert!(what.contains(field), "{what}"),
            other => panic!("{field}: {other:?}"),
        }
    }
}

#[test]
fn seat_kinds_have_their_names() {
    assert_eq!(SeatKind::Run.as_str(), "run");
    assert_eq!(SeatKind::Device.as_str(), "device");
    assert_eq!(SeatKind::parse("run"), Some(SeatKind::Run));
    assert_eq!(SeatKind::parse("device"), Some(SeatKind::Device));
    assert_eq!(SeatKind::parse("ingest"), None);
}
