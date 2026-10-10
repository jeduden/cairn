---
n: 1
title: "Every gate in Rust, beside its Go original"
status: "✅"
result: true
summary: >-
  Every Go gate has a Rust port under tooling/, and cucumber-rs runs
  the six bound scenarios. CI run 38060936531 on 594102c passed the Go
  drift job and the Rust one side by side: every registered drift is
  caught by both languages.
---
# Phase 1 handoff

## Done

- A Cargo workspace under `tooling/`, pinned to Rust 1.99.0, holds
  `srs`, `scenario`, `adr`, `finding-ledger`, `review-gate`, `drift`,
  `engineering` and `testkit`.
- The `bdd` target runs `features/` through cucumber-rs in an isolated
  `HOME`. ENG-01, ENG-18, ENG-24, ENG-26, ENG-27 and ENG-28 pass in it
  and in godog.
- CI run 38060936531 on 594102c passed every job, `drift` and
  `rust-drift` among them: the 28 registered drifts are caught by the
  Go and the Rust checks alike, and the known gap by neither.
- gherkin-rs needed two wording changes: feature descriptions open
  "The scenarios for SRS", and OWN-07's pending step writes `{id}`.

## Next

- Phase 2 deletes the Go gates, since the evidence above is in.
- The Rust stack's ADR is proposed; phase 2 accepts it once ENG-18
  reads `cargo metadata`.
