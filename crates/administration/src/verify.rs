//! `cairn verify`'s check of the record (ADM-08's first part), read from
//! one snapshot of the store and checked against this node's device key:
//!
//! - every writer's chain (REC-10) and every cited head;
//! - every stored content against its commitment (REC-17);
//! - the node's genesis: a device seat its device key certified whose
//!   log begins with that key's `minted` key event (PRV-12), and whose
//!   create room act records the nonce the personal room's id derives
//!   from;
//! - every writer's seat certificate (PRV-11): it checks, names its own
//!   seat, was certified by this node's device key, and the genesis
//!   device seat holds its certificate record, as each certificate
//!   record there names a seat the store holds;
//! - trusted provenance only in device seats.
//!
//! The first failure is named by the event's full address, or by the
//! writer. A record that passes yields its causal order.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use key::certificate::{SeatCertificate, SeatKind};
use key::key_event::KeyEvent;
use key::signing::PublicKey;
use record::chain::{self, ChainError, Link};
use record::commitment::commit;
use record::digest::unhex;
use record::event::{EventKind, Provenance};
use record::id::{Address, RoomId, WriterId};
use serde_json::Value;
use store::{Store, StoreError, Stored, StoredSeat};

use crate::node::begins_genesis;

/// What a verified record holds, and the causal order its cited heads
/// give.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub writers: usize,
    pub events: usize,
    pub order: Vec<Address>,
    /// The room each writer's seat is in.
    pub rooms: BTreeMap<WriterId, RoomId>,
}

impl Report {
    /// The causal order, each event by its full address.
    #[must_use]
    pub fn full_order(&self) -> Vec<String> {
        self.order
            .iter()
            .map(|a| {
                self.rooms
                    .get(&a.writer)
                    .map_or_else(|| a.to_string(), |r| a.full(r))
            })
            .collect()
    }
}

/// Where and how the record fails verification.
#[derive(Debug)]
pub enum VerifyError {
    /// A writer's chain, or a head it cites, breaks at this event.
    Broken { address: String, reason: String },
    /// This event's stored content does not open its commitment, or was
    /// removed with no tombstone to record it.
    Content { address: String, removed: bool },
    /// The key event at this address does not check.
    KeyEvent { address: String, reason: String },
    /// The certificate record at this address names no seat the store
    /// holds.
    Record { address: String },
    /// The event at this address claims trusted provenance outside a
    /// device seat.
    Provenance { address: String, provenance: String },
    /// A writer's seat certificate does not check.
    Seat { writer: WriterId, reason: String },
    /// This node's genesis does not check.
    Genesis(&'static str),
    /// A stored value does not parse.
    Corrupt(StoreError),
    /// The store failed.
    Store(StoreError),
}

impl VerifyError {
    /// The failure's class, as `cairn verify --json` names it.
    #[must_use]
    pub fn class(&self) -> &'static str {
        match self {
            Self::Broken { .. } => "chain",
            Self::Content { .. } => "content",
            Self::KeyEvent { .. } => "key_event",
            Self::Record { .. } => "certificate_record",
            Self::Provenance { .. } => "provenance",
            Self::Seat { .. } => "seat",
            Self::Genesis(_) => "genesis",
            Self::Corrupt(_) => "corrupt",
            Self::Store(_) => "store",
        }
    }

    /// The full address of the event the failure lies at, if it lies
    /// at one.
    #[must_use]
    pub fn address(&self) -> Option<&str> {
        match self {
            Self::Broken { address, .. }
            | Self::Content { address, .. }
            | Self::KeyEvent { address, .. }
            | Self::Record { address }
            | Self::Provenance { address, .. } => Some(address),
            Self::Seat { .. } | Self::Genesis(_) | Self::Corrupt(_) | Self::Store(_) => None,
        }
    }
}

impl fmt::Display for VerifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Broken { address, reason } => write!(f, "chain broken at {address}: {reason}"),
            Self::Content {
                address,
                removed: true,
            } => write!(
                f,
                "the content at {address} was removed, and no tombstone records it"
            ),
            Self::Content {
                address,
                removed: false,
            } => write!(f, "the content at {address} does not open its commitment"),
            Self::KeyEvent { address, reason } => {
                write!(f, "the key event at {address} does not check: {reason}")
            }
            Self::Record { address } => write!(
                f,
                "the certificate record at {address} names a seat this node does not hold"
            ),
            Self::Provenance {
                address,
                provenance,
            } => write!(
                f,
                "the event at {address} is of {provenance} provenance, which only a device seat records"
            ),
            Self::Seat { writer, reason } => write!(f, "writer {writer}: {reason}"),
            Self::Genesis(why) => f.write_str(why),
            Self::Corrupt(e) | Self::Store(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for VerifyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Corrupt(e) | Self::Store(e) => Some(e),
            _ => None,
        }
    }
}

impl From<StoreError> for VerifyError {
    fn from(e: StoreError) -> Self {
        match e {
            StoreError::Corrupt { .. } => Self::Corrupt(e),
            e => Self::Store(e),
        }
    }
}

/// The provenance classes only a device seat records: this node's own
/// operator, structural and launcher events (I2).
const TRUSTED: [Provenance; 3] = [
    Provenance::Operator,
    Provenance::Structural,
    Provenance::HarnessMeta,
];

/// Each writer's log as one snapshot read it.
type Logs = Vec<(WriterId, Result<Vec<Stored>, StoreError>)>;

/// Verifies the whole record in `store` against the node whose device
/// key is `node_key`.
///
/// # Errors
///
/// Fails at the first failure, naming it, or when the store fails.
pub fn verify(store: &Store, node_key: &PublicKey) -> Result<Report, VerifyError> {
    let (seats, writers, logs) = store.snapshot(|s| {
        let seats = s.seats()?;
        let writers = s.writers()?;
        let logs: Logs = writers.iter().map(|w| (*w, s.log(*w))).collect();
        Ok((seats, writers, logs))
    })?;
    let seats: BTreeMap<WriterId, StoredSeat> = seats.into_iter().map(|s| (s.writer, s)).collect();
    let full = |a: Address| {
        seats
            .get(&a.writer)
            .map_or_else(|| a.to_string(), |s| a.full(&s.room))
    };
    let broken = |e: ChainError| VerifyError::Broken {
        address: full(e.address),
        reason: e.fault.to_string(),
    };
    let mut held = BTreeMap::new();
    for (writer, log) in logs {
        let log = log.map_err(|e| match e {
            StoreError::Corrupt {
                address: Some(a),
                what,
            } => VerifyError::Broken {
                address: full(a),
                reason: format!("the stored header does not parse: {what}"),
            },
            e => e.into(),
        })?;
        let links: Vec<Link> = log.iter().map(|s| s.link.clone()).collect();
        chain::check_writer(writer, &links).map_err(broken)?;
        if let Some(s) = log.iter().find(|s| !opens(s)) {
            return Err(VerifyError::Content {
                address: full(s.link.header.address),
                removed: s.content.is_none() && s.commitment_key.is_none(),
            });
        }
        held.insert(writer, log);
    }
    let record: BTreeMap<WriterId, Vec<Link>> = held
        .iter()
        .map(|(w, log)| (*w, log.iter().map(|s| s.link.clone()).collect()))
        .collect();
    chain::check_heads(&record).map_err(broken)?;
    let certs = certificates(&writers, &seats)?;
    let genesis = genesis(node_key, &certs, &held)?;
    let genesis_log = held.get(&genesis).map_or(&[][..], Vec::as_slice);
    let recorded = certificate_records(genesis_log, &certs, &full)?;
    for (writer, cert) in &certs {
        let seat_error = |reason: &str| VerifyError::Seat {
            writer: *writer,
            reason: reason.to_owned(),
        };
        if cert.certifying_key != *node_key {
            return Err(seat_error(
                "its seat certificate was certified by a key that is not this node's own",
            ));
        }
        if !recorded.contains(writer) {
            return Err(seat_error(
                "no certificate record in this node's genesis device seat holds its seat certificate",
            ));
        }
    }
    for (writer, log) in &held {
        let device = certs
            .get(writer)
            .is_some_and(|c| c.seat_kind == SeatKind::Device);
        events(log, device, &full)?;
    }
    let order = chain::causal_order(&record).map_err(broken)?;

    Ok(Report {
        writers: writers.len(),
        events: order.len(),
        order,
        rooms: certs.iter().map(|(w, c)| (*w, c.room)).collect(),
    })
}

/// Whether a stored event's content opens its commitment. Content
/// removed with its key does not: it is a loss until purge records a
/// tombstone for it (ADM-07).
fn opens(stored: &Stored) -> bool {
    match (&stored.content, &stored.commitment_key) {
        (Some(content), Some(key)) => {
            commit(key, content).is_ok_and(|c| c == stored.link.header.commitment)
        }
        _ => false,
    }
}

/// Every writer's seat certificate, each checked and naming its own
/// seat.
fn certificates(
    writers: &[WriterId],
    seats: &BTreeMap<WriterId, StoredSeat>,
) -> Result<BTreeMap<WriterId, SeatCertificate>, VerifyError> {
    let mut out = BTreeMap::new();
    for writer in writers {
        let seat_error = |reason: String| VerifyError::Seat {
            writer: *writer,
            reason,
        };
        let seat = seats
            .get(writer)
            .ok_or_else(|| seat_error("no seat certificate names it".to_owned()))?;
        let cert = SeatCertificate::from_value(&seat.certificate)
            .and_then(|c| c.check().map(|()| c))
            .map_err(|e| seat_error(format!("its seat certificate: {e}")))?;
        if cert.writer_id() != *writer || cert.seat_id() != seat.seat || cert.room != seat.room {
            return Err(seat_error(
                "its seat certificate names another seat".to_owned(),
            ));
        }
        out.insert(*writer, cert);
    }

    Ok(out)
}

/// The writer of this node's genesis: the device seat `node_key`
/// certified whose log begins with that key's `minted` key event, and
/// whose create room act records the nonce its room's id derives from.
fn genesis(
    node_key: &PublicKey,
    certs: &BTreeMap<WriterId, SeatCertificate>,
    held: &BTreeMap<WriterId, Vec<Stored>>,
) -> Result<WriterId, VerifyError> {
    let log = |w: &WriterId| held.get(w).map_or(&[][..], Vec::as_slice);
    let (writer, cert) = certs
        .iter()
        .filter(|(_, c)| c.seat_kind == SeatKind::Device && c.certifying_key == *node_key)
        .find(|(w, _)| begins_genesis(log(w).first(), node_key))
        .ok_or(VerifyError::Genesis(
            "no device seat of this node's device key begins with its minted key event",
        ))?;
    let room = log(writer)
        .get(1)
        .filter(|s| s.link.header.kind == EventKind::CreateRoom)
        .and_then(|s| s.content.as_ref())
        .and_then(|act| derived_room(&cert.seat_key, act));
    if room != Some(cert.room) {
        return Err(VerifyError::Genesis(
            "the personal room's id does not derive from its device seat's key and the nonce its create room act records",
        ));
    }

    Ok(*writer)
}

/// The room a create room act names, when its id derives from the seat
/// key `seat_key` and the nonce the act records.
fn derived_room(seat_key: &PublicKey, act: &Value) -> Option<RoomId> {
    let nonce = act["nonce"].as_str().and_then(|n| unhex(n).ok())?;
    let room = RoomId::derive(seat_key.as_bytes(), &nonce);

    (act["room"].as_str() == Some(room.to_string().as_str())).then_some(room)
}

/// The writers whose seat certificates the certificate records in
/// `log`, the genesis device seat's, hold; each record must name a seat
/// the store holds, by its writer, its seat and its certificate's
/// digest.
fn certificate_records(
    log: &[Stored],
    certs: &BTreeMap<WriterId, SeatCertificate>,
    full: &dyn Fn(Address) -> String,
) -> Result<BTreeSet<WriterId>, VerifyError> {
    let mut out = BTreeSet::new();
    let records = log
        .iter()
        .filter(|s| s.link.header.kind == EventKind::CertificateRecord);
    for stored in records {
        let content = stored.content.as_ref().unwrap_or(&Value::Null);
        let writer = content["writer"]
            .as_str()
            .and_then(|w| WriterId::parse(w).ok());
        let held = writer.and_then(|w| certs.get(&w).map(|c| (w, c)));
        let named = held.filter(|(_, c)| {
            content["certificate"].as_str() == Some(c.digest().to_hex().as_str())
                && content["seat"].as_str() == Some(c.seat_id().to_string().as_str())
        });
        let (writer, _) = named.ok_or_else(|| VerifyError::Record {
            address: full(stored.link.header.address),
        })?;
        out.insert(writer);
    }

    Ok(out)
}

/// Checks each event of one writer's `log`: a key event's content is a
/// key event, and trusted provenance appears only when `device`, the
/// writer a device seat's.
fn events(
    log: &[Stored],
    device: bool,
    full: &dyn Fn(Address) -> String,
) -> Result<(), VerifyError> {
    for stored in log {
        let header = &stored.link.header;
        if !device && TRUSTED.contains(&header.provenance) {
            return Err(VerifyError::Provenance {
                address: full(header.address),
                provenance: header.provenance.name(),
            });
        }
        if header.kind == EventKind::KeyEvent {
            // Content that is not there failed the commitment already.
            let content = stored.content.as_ref().unwrap_or(&Value::Null);
            KeyEvent::from_value(content).map_err(|e| VerifyError::KeyEvent {
                address: full(header.address),
                reason: e.to_string(),
            })?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
