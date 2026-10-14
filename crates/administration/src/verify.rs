//! `cairn verify`'s check of the record (ADM-08's first part): every
//! writer's chain (REC-10), every stored content against its commitment
//! (REC-17), every cited head, and every writer's seat certificate
//! (PRV-11): it checks, names its own seat, was made by a key whose
//! `minted` key event this node recorded (PRV-12), and a device seat
//! holds its certificate record. The first break is named by the event's
//! full address, or by the writer.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use key::certificate::{SeatCertificate, SeatKind};
use key::key_event::KeyEvent;
use record::chain::{self, ChainError, Link};
use record::commitment::commit;
use record::digest::Digest;
use record::event::EventKind;
use record::id::{Address, RoomId, WriterId};
use store::{Store, StoreError, Stored, StoredSeat};

/// What a verified record holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Report {
    pub writers: usize,
    pub events: usize,
}

/// Where and how the record fails verification.
#[derive(Debug)]
pub enum VerifyError {
    /// A writer's chain breaks at this event.
    Broken { address: String, reason: String },
    /// A key event does not check.
    KeyEvent { address: String, reason: String },
    /// A writer's seat certificate does not check.
    Seat { writer: WriterId, reason: String },
    /// The store failed.
    Store(StoreError),
}

impl fmt::Display for VerifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Broken { address, reason } => write!(f, "chain broken at {address}: {reason}"),
            Self::KeyEvent { address, reason } => {
                write!(f, "the key event at {address} does not check: {reason}")
            }
            Self::Seat { writer, reason } => write!(f, "writer {writer}: {reason}"),
            Self::Store(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for VerifyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Store(e) => Some(e),
            _ => None,
        }
    }
}

impl From<StoreError> for VerifyError {
    fn from(e: StoreError) -> Self {
        Self::Store(e)
    }
}

/// Verifies the whole record in `store`.
///
/// # Errors
///
/// Fails at the first break, naming it, or when the store fails.
pub fn verify(store: &Store) -> Result<Report, VerifyError> {
    let seats: BTreeMap<WriterId, StoredSeat> =
        store.seats()?.into_iter().map(|s| (s.writer, s)).collect();
    let rooms: BTreeMap<WriterId, RoomId> = seats.iter().map(|(w, s)| (*w, s.room)).collect();
    let full = |a: Address| {
        rooms
            .get(&a.writer)
            .map_or_else(|| a.to_string(), |r| a.full(r))
    };
    let broken = |a: Address, reason: String| VerifyError::Broken {
        address: full(a),
        reason,
    };
    let writers = store.writers()?;
    let mut logs = BTreeMap::new();
    for writer in &writers {
        let log = match store.log(*writer) {
            Ok(log) => log,
            Err(StoreError::Corrupt {
                address: Some(a),
                what,
            }) => {
                return Err(broken(
                    a,
                    format!("the stored header does not parse: {what}"),
                ));
            }
            Err(e) => return Err(e.into()),
        };
        let links: Vec<Link> = log.iter().map(|s| s.link.clone()).collect();
        chain::check_writer(*writer, &links).map_err(|e| broken(e.address, e.fault.to_string()))?;
        if let Some((s, reason)) = log.iter().find_map(|s| unopened(s).map(|r| (s, r))) {
            return Err(broken(s.link.header.address, reason.to_owned()));
        }
        logs.insert(*writer, log);
    }
    let record: BTreeMap<WriterId, Vec<Link>> = logs
        .iter()
        .map(|(w, log)| (*w, log.iter().map(|s| s.link.clone()).collect()))
        .collect();
    chain::check_heads(&record)
        .map_err(|ChainError { address, fault }| broken(address, fault.to_string()))?;
    let certs = certificates(&writers, &seats)?;
    let (minted, recorded) = device_records(&certs, &logs, &full)?;
    for (writer, cert) in &certs {
        let seat_error = |reason: &str| VerifyError::Seat {
            writer: *writer,
            reason: reason.to_owned(),
        };
        let certifying_key_minted = minted
            .get(&cert.certifying_key.fingerprint())
            .is_some_and(|e| e.check_minted(&cert.certifying_key).is_ok());
        if !certifying_key_minted {
            return Err(seat_error(
                "no minted key event of this node names its certifying_key",
            ));
        }
        if !recorded.contains(&cert.digest()) {
            return Err(seat_error(
                "no device seat holds a certificate record of its seat certificate",
            ));
        }
    }

    Ok(Report {
        writers: writers.len(),
        events: logs.values().map(Vec::len).sum(),
    })
}

/// Why a stored event's content does not open its commitment, if it
/// does not. Content removed with its key is a loss until purge records
/// a tombstone for it (ADM-07), so it is named too.
fn unopened(stored: &Stored) -> Option<&'static str> {
    let opens = match (&stored.content, &stored.commitment_key) {
        (Some(content), Some(key)) => {
            commit(key, content).is_ok_and(|c| c == stored.link.header.commitment)
        }
        (None, None) => return Some("its content was removed, and no tombstone records it"),
        _ => false,
    };

    (!opens).then_some("the content does not open its commitment")
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

/// The `minted` key events the device seats hold, by their key's
/// fingerprint, and the certificate digests their certificate records
/// hold.
fn device_records(
    certs: &BTreeMap<WriterId, SeatCertificate>,
    logs: &BTreeMap<WriterId, Vec<Stored>>,
    full: &dyn Fn(Address) -> String,
) -> Result<(BTreeMap<Digest, KeyEvent>, BTreeSet<Digest>), VerifyError> {
    let (mut minted, mut recorded) = (BTreeMap::new(), BTreeSet::new());
    let device_logs = certs
        .iter()
        .filter(|(_, c)| c.seat_kind == SeatKind::Device)
        .filter_map(|(w, _)| logs.get(w));
    for stored in device_logs.flatten() {
        let header = &stored.link.header;
        match header.kind {
            EventKind::KeyEvent => {
                let failed = |reason: String| VerifyError::KeyEvent {
                    address: full(header.address),
                    reason,
                };
                // Content removed with no tombstone failed the chain
                // check already; a key event's is never removed.
                let content = stored.content.as_ref().unwrap_or(&serde_json::Value::Null);
                let event = KeyEvent::from_value(content).map_err(|e| failed(e.to_string()))?;
                minted.insert(event.key, event);
            }
            EventKind::CertificateRecord => {
                let digest = stored
                    .content
                    .as_ref()
                    .and_then(|c| c["certificate"].as_str());
                recorded.extend(digest.and_then(|d| Digest::from_hex(d).ok()));
            }
            _ => {}
        }
    }

    Ok((minted, recorded))
}

#[cfg(test)]
mod tests;
