---
n: 1
title: "Proving slice: one transcript recorded, recalled and verified"
status: "🔲"
result: false
---
# Phase 1: one transcript recorded, recalled and verified

Requirements. REC-10 (P0) closes. The slice also lays the first thin
path through REC-06, REC-17, PRV-11, PRV-12 and RCL-04, which their
own phases complete.

The slice runs one Claude Code 2.1.296 transcript from
`testdata/claude-code` through the whole record, in an isolated
`CAIRN_HOME`:

- At its first start, the node mints its device key, kept in a `0600`
  file for now; phase 5 builds the key stores the plan's ruling 2
  names, and records their ADR. The device seat records the personal
  room's create room act, then the genesis log's `minted` key event.
- `cairn ingest <transcript>` starts an ingest seat in the personal
  room. Its seat certificate is signed by the device key and
  countersigned by the seat key.
- Each line becomes one event in that seat's writer. Its header holds
  the address, event kind, provenance class, `prev_hash`, the other
  writers' chain heads it has seen, and the commitment under its
  commitment key, never content. The header is hashed as §8.3 says:
  RFC 8785 over SHA-256.
- Events land in `store.db` through `rusqlite`, 100 per transaction.
- `cairn mcp` over stdio answers `event_get` for an address with the
  event's content inside the envelope of §9.3.
- `cairn verify` checks every writer's chain and seat certificate,
  exits 0, and exits non-zero naming the event when one header byte
  changes.

The slice fixes what later phases copy. It splits the crates, with
ENG-02's declared directions. It records the canonical field sets in
`meta`. It shows how a scenario step drives `cairn` in an isolated
home through the testkit, and how a test reads the store.

Dependencies. The ADRs for the hash, the signatures and the random
source are written and accepted with their modules (ENG-18, ENG-26).
ADR-2610110553 (`rusqlite`) and ADR-2610102211 (`serde_json` in the
core) are accepted when their modules land, and the reach test allows
their crates. ENG-18's ten counts only crates linked into a shipped
executable, on the stakeholder's ruling of 14 October 2026; the slice
adds four of the core's nine.

Names. Ask the domain-model agent before naming a crate, type, CLI
verb or MCP tool. Names follow ENG-02's list and the domain model.

BDD coverage: `@REC-10` comes off `@pending`. Its steps bind two
writers recorded under clocks an hour apart, and `cairn verify`. The
other rows stay pending until their phases.

RED, each test failing first. Unit tests cover the header encoding,
the commitment and the certificate check. Integration tests in the
core crate's `tests/` cover the store's 100-event transaction and
`event_get`'s envelope. Then the `@REC-10` steps, and one thin
end-to-end test of the whole slice.

GREEN: the crates, the store's first schema, `cairn ingest`, `cairn
mcp`'s `event_get` and `cairn verify`, through the core executable.

Gate: the slice's end-to-end test passes in an isolated home, and
`@REC-10` passes. A flipped header byte fails `cairn verify`, which
names the event's address.
