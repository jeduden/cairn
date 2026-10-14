//! The node in a home. At its first start it mints its device key and
//! the recovery key its genesis log commits to, and its device seat in
//! the personal room, whose writer records, in one transaction, the
//! genesis log's `minted` key event, then the personal room's create
//! room act, then the certificate record of its own seat certificate
//! (PRV-11, PRV-12). Later starts find that seat by the device key.
//!
//! The keys are written before the store, so the store never names a
//! key the home lost.

use key::certificate::{SeatCertificate, SeatKind};
use key::key_event::KeyEvent;
use key::random::{Entropy, draw};
use key::signing::{DeviceKey, RecoveryKey, SeatKey};
use record::commitment::CommitmentKey;
use record::digest::hex;
use record::event::{EventKind, Provenance};
use record::id::{RoomId, WriterId};
use serde_json::{Value, json};
use store::{Draft, SeatRecord, Store};

use crate::AdminError;
use crate::home::Home;

/// The node: its device key, its device seat's writer and its personal
/// room.
#[derive(Debug)]
pub struct Node {
    device: DeviceKey,
    device_writer: WriterId,
    room: RoomId,
}

impl Node {
    /// The node in `home`, started for the first time when the home
    /// holds no device key.
    ///
    /// # Errors
    ///
    /// Fails when a key cannot be minted, kept or read, when the store
    /// fails, or when the home holds a device key whose device seat the
    /// store lacks.
    pub fn open(
        home: &Home,
        store: &mut Store,
        entropy: &mut dyn Entropy,
    ) -> Result<Self, AdminError> {
        if home.device_key().exists() {
            return find(home, store);
        }
        let device = DeviceKey::mint(entropy)?;
        let recovery = RecoveryKey::mint(entropy)?;
        let seat = SeatKey::mint(entropy)?;
        let nonce: [u8; 16] = draw(entropy)?;
        let room = RoomId::derive(seat.public().as_bytes(), &nonce);
        let cert = SeatCertificate::issue(&device, &seat, room, SeatKind::Device, false)?;
        device.save(&home.device_key())?;
        recovery.save(&home.recovery_key())?;
        seat.save(&home.seat_key(cert.seat_id()))?;
        let record = seat_record(&cert);
        store.add_seat(&record)?;
        let minted = KeyEvent::minted(&device, &recovery.public())?;
        let keys: [[u8; 32]; 3] = [draw(entropy)?, draw(entropy)?, draw(entropy)?];
        let room_act = json!({"room": room.to_string(), "nonce": hex(&nonce)});
        let drafts = [
            draft(
                keys[0],
                EventKind::KeyEvent,
                Provenance::Structural,
                minted.to_value(),
            ),
            draft(
                keys[1],
                EventKind::CreateRoom,
                Provenance::Operator,
                room_act,
            ),
            draft(
                keys[2],
                EventKind::CertificateRecord,
                Provenance::Structural,
                certificate_record(&cert),
            ),
        ];
        store.append(record.writer, &drafts)?;

        Ok(Self {
            device,
            device_writer: record.writer,
            room,
        })
    }

    /// The device key.
    #[must_use]
    pub fn device(&self) -> &DeviceKey {
        &self.device
    }

    /// The device seat's writer.
    #[must_use]
    pub fn device_writer(&self) -> WriterId {
        self.device_writer
    }

    /// The personal room.
    #[must_use]
    pub fn room(&self) -> RoomId {
        self.room
    }

    /// Starts an ingest seat in the personal room: mints its key, keeps
    /// it in the home, certifies it with the device key, and records the
    /// certificate record in the device seat.
    ///
    /// # Errors
    ///
    /// Fails when the key cannot be minted or kept, or the store fails.
    pub fn certify_ingest_seat(
        &self,
        home: &Home,
        store: &mut Store,
        entropy: &mut dyn Entropy,
    ) -> Result<SeatRecord, AdminError> {
        let seat = SeatKey::mint(entropy)?;
        let key = draw(entropy)?;
        let cert = SeatCertificate::issue(&self.device, &seat, self.room, SeatKind::Run, true)?;
        seat.save(&home.seat_key(cert.seat_id()))?;
        let record = seat_record(&cert);
        store.add_seat(&record)?;
        let content = certificate_record(&cert);
        let drafts = [draft(
            key,
            EventKind::CertificateRecord,
            Provenance::Structural,
            content,
        )];
        store.append(self.device_writer, &drafts)?;

        Ok(record)
    }
}

/// The node whose device key `home` holds, by the device seat that key
/// certified.
fn find(home: &Home, store: &Store) -> Result<Node, AdminError> {
    let device = DeviceKey::load(&home.device_key())?;
    for seat in store.seats()? {
        let cert = SeatCertificate::from_value(&seat.certificate)?;
        if cert.seat_kind == SeatKind::Device && cert.certifier == device.public() {
            return Ok(Node {
                device,
                device_writer: seat.writer,
                room: seat.room,
            });
        }
    }

    Err(AdminError::Node(
        "the home holds a device key but the store no device seat it certified",
    ))
}

/// The store's record of the seat `cert` certifies.
fn seat_record(cert: &SeatCertificate) -> SeatRecord {
    SeatRecord {
        writer: cert.writer_id(),
        seat: cert.seat_id(),
        room: cert.room,
        certificate: cert.to_value(),
    }
}

/// A certificate record's content: the certificate's digest and the
/// seat and writer it names.
fn certificate_record(cert: &SeatCertificate) -> Value {
    json!({
        "certificate": cert.digest().to_hex(),
        "seat": cert.seat_id().to_string(),
        "writer": cert.writer_id().to_string(),
    })
}

/// A draft of `content`, under the commitment key `key`.
fn draft(key: [u8; 32], kind: EventKind, provenance: Provenance, content: Value) -> Draft {
    Draft {
        kind,
        provenance,
        content,
        commitment_key: CommitmentKey::from_bytes(key),
    }
}

#[cfg(test)]
mod tests;
