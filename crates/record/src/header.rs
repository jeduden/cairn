//! The event header REC-10 chains: the address, the event kind, the
//! provenance class, `prev_hash`, the heads of the other writers' logs
//! the writer had seen, and the commitment, and nothing else; never
//! content, never a clock. An event's hash is SHA-256 over `prev_hash`
//! followed by the header's canonical encoding (§8.3).
//!
//! An event cites the heads its writer saw advance since its previous
//! event, so a writer's chain carries, event by event, everything it
//! had seen; an event that saw nothing new cites none.

use serde_json::{Value, json};

use crate::canonical::write_string;
use crate::commitment::Commitment;
use crate::digest::Digest;
use crate::event::{EventKind, Provenance};
use crate::id::{Address, Seq, WriterId};

/// The header's field set, recorded in the store's `meta` so a later
/// change stays verifiable (§8.3).
pub const FIELD_SET: &str =
    "address{writer,seq},kind,provenance,prev_hash,heads[{writer,seq,head}],commitment";

/// The encoding and its version, recorded in `meta` beside the field
/// set (§8.3).
pub const ENCODING: &str = "rfc8785+sha256/1";

/// Another writer's chain head as an event cites it: the address of
/// that writer's event and the event's hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeenHead {
    pub address: Address,
    pub head: Digest,
}

impl SeenHead {
    /// The head at `address`, whose hash is `head`.
    #[must_use]
    pub fn new(address: Address, head: Digest) -> Self {
        Self { address, head }
    }
}

/// An event's header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventHeader {
    pub address: Address,
    pub kind: EventKind,
    pub provenance: Provenance,
    /// The hash of the writer's previous event; [`Digest::ZERO`] for its
    /// first.
    pub prev_hash: Digest,
    /// The other writers' heads cited, in any order: the encoding sorts
    /// them by writer id.
    pub heads: Vec<SeenHead>,
    pub commitment: Commitment,
}

impl EventHeader {
    /// The header as a JSON value, its field set exactly.
    #[must_use]
    pub fn to_value(&self) -> Value {
        let heads: Vec<Value> = self
            .sorted_heads()
            .iter()
            .map(|s| {
                json!({
                    "writer": s.address.writer.to_string(),
                    "seq": s.address.seq.get(),
                    "head": s.head.to_hex(),
                })
            })
            .collect();

        json!({
            "address": {"writer": self.address.writer.to_string(), "seq": self.address.seq.get()},
            "kind": self.kind.as_str(),
            "provenance": self.provenance.name(),
            "prev_hash": self.prev_hash.to_hex(),
            "heads": heads,
            "commitment": self.commitment.to_string(),
        })
    }

    /// The header's canonical encoding: [`to_value`](Self::to_value)
    /// through RFC 8785, written directly. Every number in a header is
    /// a [`Seq`], which the encoding writes exactly, so it cannot fail.
    #[must_use]
    pub fn canonical(&self) -> String {
        let mut out = String::from("{\"address\":");
        address(&mut out, self.address.writer, self.address.seq);
        out.push_str(",\"commitment\":");
        write_string(&self.commitment.to_string(), &mut out);
        out.push_str(",\"heads\":[");
        for (i, s) in self.sorted_heads().iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str("{\"head\":");
            write_string(&s.head.to_hex(), &mut out);
            out.push_str(&format!(",\"seq\":{},\"writer\":", s.address.seq));
            write_string(&s.address.writer.to_string(), &mut out);
            out.push('}');
        }
        out.push_str("],\"kind\":");
        write_string(self.kind.as_str(), &mut out);
        out.push_str(",\"prev_hash\":");
        write_string(&self.prev_hash.to_hex(), &mut out);
        out.push_str(",\"provenance\":");
        write_string(&self.provenance.name(), &mut out);
        out.push('}');

        out
    }

    /// The event's hash: SHA-256 over `prev_hash`, then the canonical
    /// header.
    #[must_use]
    pub fn hash(&self) -> Digest {
        Digest::of(&[self.prev_hash.as_bytes(), self.canonical().as_bytes()])
    }

    fn sorted_heads(&self) -> Vec<SeenHead> {
        let mut heads = self.heads.clone();
        heads.sort_by_key(|s| s.address);

        heads
    }
}

/// Appends an address object, its members in canonical order.
fn address(out: &mut String, writer: WriterId, seq: Seq) {
    out.push_str(&format!("{{\"seq\":{seq},\"writer\":"));
    write_string(&writer.to_string(), out);
    out.push('}');
}

#[cfg(test)]
mod tests;
