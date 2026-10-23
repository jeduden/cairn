---
n: 11
title: "Trust, origin, branch links and repository identity"
status: "✅"
result: true
summary: >-
  @MEM-01, @OWN-01 and @LANE-02 off @pending and passing. Each event's
  trust level and origin derive from the record and the node's key set
  alone, nothing stores them, and parinn rebuild and parinn verify keep
  the kept origins honest. A repository's identity is read inside the
  core and bound once. @PRV-02 passes narrowed; @PRV-06 and @PRV-13 are
  proven by named tests; no scenario row needing M7 or OQ-40 could pass.
---
# Phase 11 result

## Handoff

Outcome:

- MEM-01, OWN-01 and LANE-02 hold: all three are off `@pending`, passing
  against the built core, confined. After every hook and `parinn
  rebuild`, no file outside the store holds a canary. A run's seats are
  all certified by the node's one device key. A repository's identity is
  its lowest parentless commit, read from `.git` inside the core and
  bound once by a structural event.
- PRV-02 holds for what M1 can show. `administration::trust_policy`
  trusts only `operator` events and the core's own structural events in
  a device seat of origin `witnessed`. No `user` event is trusted (R3 and
  R8 stay open until OQ-40), no hook observation, no `ingested` event,
  no `harness_meta` the launcher did not sign. Nothing stores a level.
  `@PRV-02` stays `@pending`: 11 rows pass narrowed locally, end to end;
  the rest need a launcher, a signed sandbox state, another node's key,
  a hook ancestry or a purge.
- PRV-06 is proven by `Taint`'s unit tests: an artifact records its
  sources and is untrusted if any is. `@PRV-06` stays pending: its rows
  need a launcher's `harness_meta`, a landmark, kernel output and an
  export.
- PRV-13's M1 part holds: `administration::origin` derives `witnessed`,
  `ingested` or `peer` per writer from the record and this node's device
  key alone, in one place that recall and `parinn verify` share. A
  replaced key's writers read `peer`, so do another node's. `parinn
  rebuild` writes the `origins` table and the search index, appending no
  event; `parinn verify` exits 3 naming "derived artifact consistency"
  when a kept origin differs. `@PRV-13` stays pending: no arrival event
  exists yet.
- LANE-01 and LANE-31's M1 part is proven by the `room` crate's unit and
  property tests: `standing_link` lets the first link in causal order
  stand, the lower commitment among concurrent ones, whatever the
  arrival order; `seat_for_run_event` routes a run's event to a joined
  room's run seat or else the personal-room seat. Both stay pending.
- ADM-13's M1 part holds: `e2e_no_git_write.rs` finds every file of a
  repository unchanged after every component ran. It stays pending (M2).
- I2 security review recorded (persona-security-officer, PRV-02 and
  PRV-13). It found no path from untrusted content to a trusted event.
  It found the git reader open to a hostile checkout, now fixed: only
  regular files open, every object's SHA-1 is verified, the ancestor
  walk stops short of the home, `commondir` must be the git directory's
  ancestor. The key set is this node's key alone, and the policy reads
  the seat kind.
- I2, I4, I6, I8, I9 and I10 hold: the core starts no program and opens
  no socket, and a changed repository identity is audited and counted.

Inherited:

- Crates `repository` (`git_dir`, `root_commit`, `has_commit`,
  `commit_object_id`) and `room` (`branch`, `route`);
  `provenance::structural_field::sanitize`.
- `administration`: `origin::{Origin, KeySet, derive, is_device_seat}`,
  `trust_policy::{trust_level, Input, TrustLevel, Taint}`,
  `repository::{bind, show, plan, apply, Outcome}`, `Act::RepositoryBind`,
  `verify`'s `VerifyError::Derived`.
- `store`: `origins` and `replace_origins`, `local_secret`,
  `directory_identity`; `EventKind::RepositoryBind`; counters
  `repository_identity_changed` and `repository_read_failed`.
- `parinn rebuild`, `parinn repository show|bind`; `mcp::Server::new`
  takes the device key; `event_search`'s `trust` filter applies after
  the ranking.

For phases 13 to 16: the envelope's shape is untouched, its `origin` and
`trust` come from `administration`; restore (M2) reads `trust_level`;
phase 14's quarantine must change no origin; phase 15's SEC-01 row
counts the `repository` crate as reached.

For the stakeholder:

- Deviation: the bind is recorded when `parinn ingest` or an MCP server
  starts, not in the hook handler: reading git cannot stay within
  NFR-02's 150 ms.
- Deviation, PRV-13: after a node identity change every writer reads
  `peer` until a PRV-12 confirm, which no M1 command makes.
- Residual after the review, raised: a file swapped for a FIFO between
  the check and the open still blocks (closing it needs `O_NONBLOCK`
  through `libc`, a new dependency); a `gitdir:` file may name any
  directory, so a hostile clone can bind another repository's commit id,
  which replicates. Its first bind should be a principal act.
- Stakeholder paths touched: `tooling/scenario`, `features/`, `fuzz/`
  (LMK-03's sanitizer and its `field_sanitizer` target) and `reach.rs`.

Parked: PRV-02's and PRV-06's remaining rows; PRV-13's arrival steps;
the trust of `user` events once OQ-40 closes.
