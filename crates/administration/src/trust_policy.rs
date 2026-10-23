//! The default trust policy (PRV-02, I2): the trust level of an event,
//! derived per event from what the record and the node's key set say of
//! it, never stored with it (I10). It trusts only the trusted sources:
//! on the recording node, its `operator` events and the structural
//! events it records itself, the `harness_meta` its launcher signed,
//! and, while the deployment mode is `interactive` and no run leaves R3
//! or R8 open, the `user` events matched to a prompt whose hook handler
//! found the attended mark. Every other event, every `ingested` one
//! included, is untrusted. A settings layer cannot widen this (PRV-05).
//!
//! Until OQ-40 provides a key that signs a sandbox state, every run
//! leaves R3 and R8 open (OWN-22), so no `user` event and no hook
//! observation is trusted, whatever the deployment mode. Stamps, trust
//! grants and the acts of a device key a principal key certified stay
//! out of the policy until PRV-10 ships (M7). Nothing here reads a
//! clock or randomness.

use record::event::{EventKind, Provenance};

use crate::origin::Origin;

/// An event's trust level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustLevel {
    Trusted,
    Untrusted,
}

impl TrustLevel {
    /// The level's name, as the envelope shows it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Trusted => "trusted",
            Self::Untrusted => "untrusted",
        }
    }
}

/// Whether a run on the node leaves R3 or R8 open: until OQ-40 every
/// run does (OWN-22).
pub const RISK_OPEN: bool = true;

/// What the trust policy reads of an event.
#[derive(Debug, Clone, Copy)]
pub struct Input<'a> {
    pub provenance: &'a Provenance,
    pub kind: EventKind,
    /// Derived by [`crate::origin`]; `witnessed` means this node's own
    /// capture or core recorded it.
    pub origin: Origin,
    /// A `harness_meta` event the launcher recorded and signed.
    pub launcher_signed: bool,
    /// The recorded deployment mode is `interactive`.
    pub interactive: bool,
    /// A `user` event matched to a prompt its hook handler got.
    pub prompt_matched: bool,
    /// That hook handler found the attended mark.
    pub attended: bool,
    /// A run on the node leaves R3 or R8 open.
    pub risk_open: bool,
}

/// The trust level the policy gives the event `input` describes.
#[must_use]
pub fn trust_level(_input: &Input<'_>) -> TrustLevel {
    TrustLevel::Untrusted
}

#[cfg(test)]
mod tests;
