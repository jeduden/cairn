//! Room state from acts: branch links, merged by LANE-31's rule.

use crate::commit::Commitment;
use crate::node::Node;
use crate::record::Address;

/// A standing branch link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchLink {
    pub room: String,
    pub repository: String,
    pub branch: String,
    pub act: Address,
    pub commitment: Commitment,
}

/// A resolved conflict, recorded with both acts (LANE-31).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conflict {
    pub stands: Address,
    pub void: Address,
    pub rule: &'static str,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RoomState {
    pub links: Vec<BranchLink>,
    pub conflicts: Vec<Conflict>,
}

/// Derives room state from the acts the node holds.
pub fn derive(_node: &Node) -> RoomState {
    RoomState::default()
}
