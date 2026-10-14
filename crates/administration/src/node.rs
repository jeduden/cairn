//! The node in a home. At its first start it mints its device key, the
//! recovery key its genesis log commits to, and its device seat's key in
//! the personal room. That seat's writer records, in one transaction,
//! the genesis log's `minted` key event, then the personal room's create
//! room act, then the certificate record of its own seat certificate
//! (PRV-11, PRV-12). Later starts find that seat by the device key's
//! `minted` key event.
//!
//! A first start stages its keys and the personal room's nonce in one
//! directory that appears whole, records the genesis, places the keys,
//! the device key last, and removes the staging. A start that finds the
//! staging finishes what an interrupted one began, so the store never
//! names a key the home lost, and a crash never leaves a home with a
//! device key but no genesis.

use std::fs;
use std::path::Path;

use key::certificate::{SeatCertificate, SeatKind};
use key::file::{read_secret, staging, sync_directory, write_secret};
use key::key_event::KeyEvent;
use key::random::{Entropy, draw};
use key::signing::{DeviceKey, PublicKey, RecoveryKey, SeatKey};
use record::commitment::CommitmentKey;
use record::digest::{hex, unhex};
use record::event::{EventKind, Provenance};
use record::id::{Address, RoomId, Seq, WriterId};
use serde_json::{Value, json};
use store::{NewEvent, Store, Stored, StoredSeat};

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
    /// holds no device key, and finished when a first start was
    /// interrupted.
    ///
    /// # Errors
    ///
    /// Fails when a key cannot be minted, kept or read, when the staged
    /// genesis does not read, when the store fails, or when the home
    /// holds a device key whose genesis the store lacks.
    pub fn open(
        home: &Home,
        store: &mut Store,
        entropy: &mut dyn Entropy,
    ) -> Result<Self, AdminError> {
        let dir = home.genesis();
        if !dir.exists() && !home.device_key().exists() {
            stage(home, entropy)?;
        }
        if dir.exists() {
            let genesis = Genesis::load(home)?;
            genesis.record(store, entropy)?;
            genesis.place(home)?;
            fs::remove_dir_all(&dir).map_err(|source| io(&dir, source))?;
            sync_directory(&dir).map_err(|source| io(&dir, source))?;
        }

        Self::find(home, store)
    }

    /// The node whose device key `home` holds, by the device seat whose
    /// log begins with that key's `minted` key event: its genesis. It
    /// writes nothing, so it finds only a node a start made.
    ///
    /// # Errors
    ///
    /// Fails when the device key cannot be read, the store fails, or
    /// the store holds no genesis of the device key.
    pub fn find(home: &Home, store: &Store) -> Result<Self, AdminError> {
        let device = DeviceKey::load(&home.device_key())?;
        let key = device.public();
        for seat in store.seats()? {
            let cert = SeatCertificate::from_value(&seat.certificate)?;
            if cert.seat_kind != SeatKind::Device || cert.certifying_key != key {
                continue;
            }
            let first = store.event(Address::new(seat.writer, Seq::FIRST))?;
            if begins_genesis(first.as_ref(), &key) {
                return Ok(Self {
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
    pub fn personal_room(&self) -> RoomId {
        self.room
    }

    /// Starts an ingest seat in the personal room: mints its key, keeps
    /// it in the home, certifies it with the device key, and records the
    /// seat and the certificate record in the device seat in one
    /// transaction.
    ///
    /// # Errors
    ///
    /// Fails when the key cannot be minted or kept, or the store fails.
    pub fn certify_ingest_seat(
        &self,
        home: &Home,
        store: &mut Store,
        entropy: &mut dyn Entropy,
    ) -> Result<StoredSeat, AdminError> {
        let seat = SeatKey::mint(entropy)?;
        let key = draw(entropy)?;
        let cert = SeatCertificate::issue(&self.device, &seat, self.room, SeatKind::Run, true)?;
        seat.save(&home.seat_key(cert.seat_id()))?;
        let record = seat_record(&cert);
        let new_events = [new_event(
            key,
            EventKind::CertificateRecord,
            Provenance::Structural,
            certificate_record(&cert),
        )];
        store.add_seat_and_append(&record, self.device_writer, &new_events)?;

        Ok(record)
    }
}

/// What a first start mints before it records anything.
struct Genesis {
    device: DeviceKey,
    recovery: RecoveryKey,
    seat: SeatKey,
    nonce: [u8; 16],
}

/// The staged files, by name.
const DEVICE: &str = "device.key";
const RECOVERY: &str = "recovery.key";
const SEAT: &str = "seat.key";
const NONCE: &str = "nonce";

/// Mints a first start's keys and the personal room's nonce, and stages
/// them in the home's genesis directory, which appears whole or not at
/// all: they are written to a directory beside it, then renamed into
/// place.
fn stage(home: &Home, entropy: &mut dyn Entropy) -> Result<(), AdminError> {
    let device = DeviceKey::mint(entropy)?;
    let recovery = RecoveryKey::mint(entropy)?;
    let seat = SeatKey::mint(entropy)?;
    let nonce: [u8; 16] = draw(entropy)?;
    let dir = home.genesis();
    let building = staging(&dir);
    match fs::remove_dir_all(&building) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(io(&building, e)),
        _ => {}
    }
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder
        .create(&building)
        .map_err(|source| io(&building, source))?;
    device.save(&building.join(DEVICE))?;
    recovery.save(&building.join(RECOVERY))?;
    seat.save(&building.join(SEAT))?;
    write_secret(&building.join(NONCE), &hex(&nonce))?;
    fs::rename(&building, &dir).map_err(|source| io(&dir, source))?;
    sync_directory(&dir).map_err(|source| io(&dir, source))
}

impl Genesis {
    /// The genesis staged in `home`.
    fn load(home: &Home) -> Result<Self, AdminError> {
        let dir = home.genesis();
        let nonce = unhex(&read_secret(&dir.join(NONCE))?)
            .map_err(|_| AdminError::Node("the staged genesis nonce is not 32 hex digits"))?;

        Ok(Self {
            device: DeviceKey::load(&dir.join(DEVICE))?,
            recovery: RecoveryKey::load(&dir.join(RECOVERY))?,
            seat: SeatKey::load(&dir.join(SEAT))?,
            nonce,
        })
    }

    /// The device seat's certificate. Its signatures are deterministic,
    /// so every start that resumes the genesis issues the same one.
    fn certificate(&self) -> Result<SeatCertificate, AdminError> {
        let room = RoomId::derive(self.seat.public().as_bytes(), &self.nonce);

        Ok(SeatCertificate::issue(
            &self.device,
            &self.seat,
            room,
            SeatKind::Device,
            false,
        )?)
    }

    /// Records the device seat and its first three events in one
    /// transaction, unless an earlier start recorded them.
    fn record(&self, store: &mut Store, entropy: &mut dyn Entropy) -> Result<(), AdminError> {
        let cert = self.certificate()?;
        let record = seat_record(&cert);
        if store.seat(record.writer)?.is_some() {
            return Ok(());
        }
        let minted = KeyEvent::minted(&self.device, &self.recovery.public())?;
        let keys: [[u8; 32]; 3] = [draw(entropy)?, draw(entropy)?, draw(entropy)?];
        let room_act = json!({"room": cert.room.to_string(), "nonce": hex(&self.nonce)});
        let new_events = [
            new_event(
                keys[0],
                EventKind::KeyEvent,
                Provenance::Structural,
                minted.to_value(),
            ),
            new_event(
                keys[1],
                EventKind::CreateRoom,
                Provenance::Operator,
                room_act,
            ),
            new_event(
                keys[2],
                EventKind::CertificateRecord,
                Provenance::Structural,
                certificate_record(&cert),
            ),
        ];
        store.add_seat_and_append(&record, record.writer, &new_events)?;

        Ok(())
    }

    /// Places the device seat's key in the home, unless it is there.
    fn place_seat_key(&self, home: &Home) -> Result<(), AdminError> {
        let path = home.seat_key(self.certificate()?.seat_id());
        place(&path, |p| self.seat.save(p))
    }

    /// Places the keys in the home, the device key last, each unless it
    /// is there.
    fn place(&self, home: &Home) -> Result<(), AdminError> {
        self.place_seat_key(home)?;
        place(&home.recovery_key(), |p| self.recovery.save(p))?;
        place(&home.device_key(), |p| self.device.save(p))
    }
}

/// Saves a key at `path` through `save`, unless a file is there.
fn place(
    path: &Path,
    save: impl FnOnce(&Path) -> Result<(), key::KeyError>,
) -> Result<(), AdminError> {
    if path.exists() {
        return Ok(());
    }

    Ok(save(path)?)
}

/// Whether `first`, a device seat's first event, is the `minted` key
/// event of `key`: the event that makes that seat the key's genesis.
pub(crate) fn begins_genesis(first: Option<&Stored>, key: &PublicKey) -> bool {
    first
        .filter(|s| s.link.header.kind == EventKind::KeyEvent)
        .and_then(|s| s.content.as_ref())
        .and_then(|c| KeyEvent::from_value(c).ok())
        .is_some_and(|e| e.check_minted(key).is_ok())
}

/// The store's record of the seat `cert` certifies.
fn seat_record(cert: &SeatCertificate) -> StoredSeat {
    StoredSeat {
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

/// A new event of `content`, under the commitment key `key`.
fn new_event(key: [u8; 32], kind: EventKind, provenance: Provenance, content: Value) -> NewEvent {
    NewEvent {
        kind,
        provenance,
        content,
        commitment_key: CommitmentKey::from_bytes(key),
    }
}

/// The failure to make, rename or remove the directory at `path`.
fn io(path: &Path, source: std::io::Error) -> AdminError {
    AdminError::Io {
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(test)]
mod tests;
