---
n: 1
title: "Proving slice: one transcript recorded, recalled and verified"
status: "✅"
result: true
summary: >-
  @REC-10 off @pending and passing. A recorded transcript is ingested,
  recalled through event_get in the envelope and verified, end to end
  and confined; a flipped header byte exits 3 naming the event.
---
# Phase 1 result

## Handoff

Outcome:

- REC-10 closes: `@REC-10` passes in the cucumber runner, with two
  writers taking turns, one clock an hour behind.
- The gate holds. `crates/cairn/tests/e2e_slice.rs` drives the
  confined core in an isolated home. It ingests the 57-line `-w-my-app`
  transcript, recalls line 3 inside the envelope, and verifies. Then
  one hex digit of event 10's commitment changes, and `cairn verify`
  exits 3, naming `cairn:room/<room>/w/<writer>/10`.
- First paths through REC-06, REC-17, PRV-11, PRV-12 and RCL-04, which
  their phases complete. I2, I4 and I10 hold: recall is pull-only and
  enveloped, and the reach test weighs every new crate.
- ADR-2610142218 to ADR-2610142221 are written as accepted:
  - `sha2`;
  - `ed25519-dalek`;
  - `getrandom`;
  - `serde_json` in the core, superseding ADR-2610101913.

  ADR-2610110553 and ADR-2610102211 are accepted. That makes seven
  direct dependencies, five of them in the core. `@ENG-18`, `@ENG-26`
  and `@ENG-02`, `cargo deny`, the drift suite and the coverage floors
  pass.

Inherited:

- The crates follow ENG-02's directions:
  - `record`: the laws, with no I/O;
  - `key`;
  - `store`;
  - `administration`: home, node, verify;
  - `ingest`, `envelope` and `mcp`;
  - `cairn`: the CLI, with `run(args, &Context, ..)` injecting the home
    and stdin.
- `store.db` is format version 1. Its tables are `meta`, `seats`,
  `events` and `event_heads`. `meta` holds the header's field set and
  encoding, and a newer store is refused.
- An append transaction's first event cites the other writers' heads
  that advanced since its writer last cited them. A head at or behind
  an earlier one fails verify.
- The first start records three events, in one transaction: the
  `minted` key event, the create room act, then the device seat's own
  certificate record. Each `cairn ingest --path` starts an ingest seat
  and records each line as an `unparsed_line` event of provenance
  `unparsed`, holding `{"line", "text"}`.
- Content removed with its key, with no tombstone, fails verify, and
  `event_get` refuses it. Phase 14's tombstones turn that into a pass.
- Scenario steps start the core through `testkit::core_executable()`
  and `Confinement::run` or `run_with_input`. They build Given state
  through the product crates, and tamper through rusqlite. The steps
  `an isolated Cairn home`, `the person runs "cairn …"` and `the
  command exits N` are bound.
- `.cargo/config.toml` compiles out SQLite's extension loading.
  `build.yml` pins `clang-18` and builds musl 1.2.4's headers for each
  musl target, checked by SHA-256. Both musl targets were built here.

Deviations, now in phase-1.md:

- Device seat order, per PRV-11 and PRV-12 and the domain-model agent:
  `minted`, then create room, then the device seat's certificate.
- `cairn ingest --path P`, as §9.5 defines it.
- `event_get` serves no run until phase 9, so it needs `scope: "room"`
  and shows `run: other` and `integrity: unsigned`. RCL-05's audit and
  RCL-07's recall event are phase 13's.
- The integration tests sit in `crates/store/tests` and
  `crates/mcp/tests`.
- Keys, the recovery key included, are `0600` files under `keys/`.
  Phase 5 decides where they live.

For the stakeholder:

- PRV-12's list of signature kinds leaves out room acts and seals. The
  create room act stays unsigned until the list is ruled closed or
  extended.
- PRV-11 needs its I2 security review before it ships.

Parked:

- `cargo test -p scenario` alone can start a stale `cairn`. The
  workspace run builds it fresh.
- The testkit waits on a fed start with no deadline (ENG-03).
- `cargo deny` warns of two `syn` versions.
- Some unit tests use OS randomness where `Counted` would do.
