---
n: 1
title: "Scope proposal and the record/index proving slice"
status: "🔲"
result: false
---
# Phase 1: scope proposal and the record/index proving slice

Requirements. No SRS id closes, and no scenario leaves `@pending`.
The phase writes evidence and a proposal; the SRS changes follow in
phase 2, for the stakeholder.

BDD coverage: none yet. The proposal names the scenarios each new
requirement would add, so phase 2 can land them `@pending`.

The proposal, `proposal.md` beside this plan, covers, per fleet
setting (worktrees, cloud sandboxes, machines, open source, real
time):

- what Cairn does there, and what stays out of scope;
- the invariant that governs it, restated for many nodes where
  needed (I1 stable addresses, I2 foreign trust, I4 core versus relay,
  I8 tenant across nodes);
- the identity model: origin (node and session), origin seq, segment
  hash, and how a project is keyed so worktrees of one clone share it.

RED: a test in a separate module beside this plan, reusing S2's corpus,
writes events as segments for two origins, A and B. It builds the
SQLite index from A then B, and from B then A. It fails until both
indexes are byte-identical, every event keeps its origin and trust,
and B's events read as untrusted on A's node.

GREEN sites: the segment writer and reader, and the index rebuild,
inside that module only. Nothing enters cairn's module graph.

Gate: the test passes on both orders, and the proposal covers every
fleet setting above.
