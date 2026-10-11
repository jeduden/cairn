---
n: 1
title: "Scope proposal and the record/index proving slice"
status: "✅"
result: true
summary: >-
  Spike S12 passes: two writers' segments in either order rebuild a
  byte-identical index, foreign events stay untrusted, one branch link
  stands, a purge leaves nothing confirming a guess. ADM-08 needs a
  decision for an index kept per append.
---
# Phase 1 result

## Handoff

Outcome, in the SRS's terms:

- Spike S12 (§12.1) passes its exit criterion, this phase's gate test.
  Mechanism: `cargo test` in the [slice](slice/report.md)'s own crate,
  outside the cairn workspace: 4 tests in `tests/proving_slice.rs`
  over 4,006 events of S2's corpus, and 21 unit tests.
- ADM-08 and PEER-03, for a rebuild: on each node, segments in any
  arrival order, reordered or duplicated, rebuild a byte-identical
  search index and room state. The two nodes' indexes differ, since
  trust levels derive per node.
- PRV-02 and RCL-09: every event keeps its address and provenance.
  The other writer's events read `peer` and untrusted, the node's own
  trusted classes trusted.
- LANE-31: concurrent branch links resolve by the lower commitment,
  causal ones by the first in causal order, each conflict recorded
  with both acts.
- REC-17, ADM-07, SEC-31 and PEER-11: after a purge, no stored or
  served value confirms a guess at the content, whatever the order of
  copies. A peer erases only on its principal's erasure request.
- The test fails with the node's tombstone handling switched off.
- No SRS id closes and no scenario leaves `@pending`: this phase is a
  spike. The proposal and persona-review parts of the gate closed
  earlier, with SRS 2.0-draft and the stakeholder's OQ-29 decision.

What M1 inherits, from the report's findings:

- ADM-08's byte-identity holds only for an index rebuilt from nothing.
  An index kept per append keeps equal rows, but FTS5's structure
  record differs, even after `optimize` and `VACUUM`. The report
  recommends a canonical digest per derived artifact. The ADM-08
  wording is the stakeholder's call.
- Row ids derive from the address or event hash, not arrival.
- A purge sets SQLite's `secure_delete` and FTS5's `secure-delete`, or
  rebuilds the index. Without them, 18 of 40 purges left text, and
  every purge left postings. In WAL mode it also checkpoints.
- Applying an erasure request must be a recorded principal act (I10).
- Full rebuild runs at about 35,000 events/s on a 4 vCPU host: about
  five minutes per 10M events.

Not covered, for M1 or later: seals, segment merges, the payload
store, open segments, ingest, quarantine, purging an act, missing
ranges, a real peer exchange, WAL mode and Windows.
