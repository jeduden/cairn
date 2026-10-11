//! The record: writers, events, addresses and segments.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::commit::{Commitment, Key};
use crate::{Error, Result};

/// The segment format version every segment carries (REC-21).
pub const FORMAT: u64 = 1;

/// An event's address: (writer, seq) (REC-06).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address {
    pub writer: String,
    pub seq: u64,
}

/// Another writer's chain head at a seq, as this writer had seen it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Head {
    pub seq: u64,
    pub chain_head: [u8; 32],
}

/// The chained header: only the address, kind, provenance, `prev_hash`,
/// the heads seen and the commitment (REC-10, REC-17).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub writer: String,
    pub seq: u64,
    pub kind: String,
    pub provenance: String,
    pub prev_hash: Option<[u8; 32]>,
    pub heads: BTreeMap<String, Head>,
    pub commitment: Commitment,
}

/// Content and its commitment key, kept beside the header and erased
/// together by a purge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Body {
    pub key: Key,
    pub content: Value,
}

/// One event: its header, the hash over it, and its body unless purged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub header: Header,
    pub hash: [u8; 32],
    pub body: Option<Body>,
}

impl Event {
    pub fn address(&self) -> Address {
        Address {
            writer: self.header.writer.clone(),
            seq: self.header.seq,
        }
    }
}

/// A closed range of one writer, the unit peers exchange.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    pub writer: String,
    pub events: Vec<Event>,
}

impl Segment {
    /// The segment's bytes: canonical JSON lines, a format line first.
    pub fn encode(&self) -> Vec<u8> {
        Vec::new()
    }

    /// Reads a segment, refusing an unknown format version (REC-21).
    pub fn decode(_bytes: &[u8]) -> Result<Segment> {
        Err(Error::new("segment reader not built"))
    }
}

/// One seat's append-only log.
pub struct Writer {
    id: String,
}

impl Writer {
    /// Opens a writer named by its seat key, recording its deployment
    /// mode as its first, structural event.
    pub fn open(_seat_key: &[u8], _deployment_mode: &str) -> Result<Writer> {
        Err(Error::new("segment writer not built"))
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    /// Appends one event and returns its address.
    pub fn append(&mut self, _kind: &str, _provenance: &str, _content: Value) -> Result<Address> {
        Err(Error::new("segment writer not built"))
    }

    /// Records the other writer's sealed chain head as seen.
    pub fn see(&mut self, _other: &Writer) {}

    /// Closes the open segment, if it holds any event.
    pub fn close_segment(&mut self) -> Option<Segment> {
        None
    }

    /// Every closed segment as it stands now, purges applied.
    pub fn segments(&self) -> Vec<Segment> {
        Vec::new()
    }

    /// The event at `seq`, if appended.
    pub fn event(&self, _seq: u64) -> Option<&Event> {
        None
    }

    /// Purges the addresses by a principal act: erases each one's
    /// content and commitment key, and appends a tombstone holding only
    /// addresses, count, reason and commitments (ADM-07).
    pub fn purge(&mut self, _addresses: &[Address], _reason: &str) -> Result<Address> {
        Err(Error::new("purge not built"))
    }
}
