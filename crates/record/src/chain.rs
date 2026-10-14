//! Each writer's hash chain (REC-06, REC-10), checked from the events
//! the store holds, and the causal order across writers the cited heads
//! give. Order across writers is causal, never wall-clock: no header
//! holds a time, so no clock can reorder the record.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::digest::Digest;
pub use crate::header::{EventHeader, SeenHead};
pub use crate::id::{Address, Seq, WriterId};

/// One stored event as a check reads it: its header and the hash the
/// store holds for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub header: EventHeader,
    pub hash: Digest,
}

/// What is wrong at one event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fault {
    /// Its seq is not the one after its predecessor's: this seq was
    /// expected instead.
    Seq(u64),
    /// It belongs to another writer.
    Writer,
    /// Its `prev_hash` is not its predecessor's hash.
    PrevHash,
    /// Its stored hash is not the hash of its header.
    Hash,
    /// A head it cites names no event of another writer with that hash.
    Head(Address),
    /// A head it cites lies at or behind one its writer cited earlier:
    /// an event cites only heads it saw advance.
    HeadBackward(Address),
    /// Its cited heads, followed back, lead to itself.
    Cycle,
}

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Seq(want) => write!(f, "want seq {want} here: the writer's log has a gap"),
            Self::Writer => f.write_str("the event belongs to another writer"),
            Self::PrevHash => f.write_str("prev_hash does not name the writer's previous event"),
            Self::Hash => f.write_str("the stored hash does not match the event's header"),
            Self::Head(cited) => write!(
                f,
                "it cites {cited}, which no event of another writer matches"
            ),
            Self::HeadBackward(cited) => write!(
                f,
                "it cites {cited}, at or behind a head its writer cited before"
            ),
            Self::Cycle => f.write_str("its cited heads form a cycle"),
        }
    }
}

/// A broken chain: the event where it breaks, and how.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainError {
    pub address: Address,
    pub fault: Fault,
}

impl ChainError {
    /// The fault `fault` at `address`.
    #[must_use]
    pub fn new(address: Address, fault: Fault) -> Self {
        Self { address, fault }
    }
}

impl fmt::Display for ChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.address, self.fault)
    }
}

impl std::error::Error for ChainError {}

/// Checks `writer`'s log, `links` in seq order: seqs from 1 with no
/// gap, every event the writer's own, each `prev_hash` its
/// predecessor's hash, and each stored hash the hash of its header.
///
/// # Errors
///
/// Fails at the first event that breaks the chain.
pub fn check_writer(writer: WriterId, links: &[Link]) -> Result<(), ChainError> {
    let mut prev = Digest::ZERO;
    for (want, link) in (1..).zip(links) {
        let h = &link.header;
        let fail = |fault| Err(ChainError::new(h.address, fault));
        if h.address.writer != writer {
            return fail(Fault::Writer);
        }
        if h.address.seq.get() != want {
            return fail(Fault::Seq(want));
        }
        if h.prev_hash != prev {
            return fail(Fault::PrevHash);
        }
        if h.hash() != link.hash {
            return fail(Fault::Hash);
        }
        prev = link.hash;
    }

    Ok(())
}

/// Checks every head each event cites: it names an event of another
/// writer that the record holds, by that event's hash, and never lies
/// at or behind a head the same writer cited before.
///
/// # Errors
///
/// Fails at the first event citing a head that breaks this.
pub fn check_heads(record: &BTreeMap<WriterId, Vec<Link>>) -> Result<(), ChainError> {
    for (writer, links) in record {
        let mut cited: BTreeMap<WriterId, Seq> = BTreeMap::new();
        for link in links {
            for seen in &link.header.heads {
                let fail = |fault| Err(ChainError::new(link.header.address, fault));
                let other = seen.address;
                let holds = other.writer != *writer
                    && record
                        .get(&other.writer)
                        .and_then(|log| {
                            usize::try_from(other.seq.get() - 1)
                                .ok()
                                .and_then(|i| log.get(i))
                        })
                        .is_some_and(|l| l.header.address == other && l.hash == seen.head);
                if !holds {
                    return fail(Fault::Head(other));
                }
                if cited.get(&other.writer).is_some_and(|s| other.seq <= *s) {
                    return fail(Fault::HeadBackward(other));
                }
                cited.insert(other.writer, other.seq);
            }
        }
    }

    Ok(())
}

/// Every event of `record` in causal order: each after its writer's
/// previous event and after every head it cites. Of the events ready
/// at once, the least address comes first, so the order is a function
/// of the record alone (I10).
///
/// # Errors
///
/// Fails when cited heads form a cycle, naming the least event on it.
pub fn causal_order(record: &BTreeMap<WriterId, Vec<Link>>) -> Result<Vec<Address>, ChainError> {
    let mut waits: BTreeMap<Address, BTreeSet<Address>> = BTreeMap::new();
    for links in record.values() {
        for link in links {
            let h = &link.header;
            let mut before: BTreeSet<Address> = h.heads.iter().map(|s| s.address).collect();
            if let Some(prev) = Seq::new(h.address.seq.get() - 1) {
                before.insert(Address::new(h.address.writer, prev));
            }
            before.retain(|a| {
                record
                    .get(&a.writer)
                    .is_some_and(|l| l.len() as u64 >= a.seq.get())
            });
            waits.insert(h.address, before);
        }
    }
    let mut order = Vec::with_capacity(waits.len());
    while let Some(next) = waits
        .iter()
        .find(|(_, before)| before.is_empty())
        .map(|(a, _)| *a)
    {
        waits.remove(&next);
        for before in waits.values_mut() {
            before.remove(&next);
        }
        order.push(next);
    }
    match waits.keys().next() {
        Some(stuck) => Err(ChainError::new(*stuck, Fault::Cycle)),
        None => Ok(order),
    }
}

#[cfg(test)]
mod tests;
