---
n: 8
title: "The core's reach evidence and unsafe code (ENG-16)"
status: "✅"
result: true
summary: >-
  Gate tests prove the core row: CI gates on the static checks under
  the ci job, every crate keeps the unsafe ban, the core's reach
  evidence and the determinism bans are in place; ENG-16 stays pending.
---
## Handoff

What landed, in the engineering crate's `static_analysis` module and
its gate tests (target `static_analysis`):

- `the_ci_workflow_gates_on_the_static_checks`: clippy with warnings
  denied, `cargo fmt --check`, cargo-deny, cargo-audit and
  `cargo test --workspace` (the custom checks, the core's reach test
  among them) each run in a job the `ci` job needs, none allowed to
  fail, and the `ci` job fails unless every job it needs succeeded.
- `every_crate_forbids_unsafe_code`: the workspace lints forbid
  `unsafe_code`, and every member crate inherits them or carries
  `#![forbid(unsafe_code)]` in each crate root.
- `the_core_s_reach_evidence_is_present_and_gated`: the core's
  clippy.toml bans every socket type and `Command`, its reach test is
  in place, and CI gates on clippy and the custom checks.
- `derived_code_is_refused_a_clock_or_randomness_at_build_time`:
  every product crate's clippy.toml bans the clocks and the hash
  collections' random order, and CI gates on clippy.
- Nine drift cases, one or more per check.
- The ENG-16 core-row steps are bound in `engineering_reach.rs`. With
  the outline narrowed locally to its core row, every core step
  passes; the static-check step then fails only on the browser room
  view client's TypeScript checks.

`@ENG-16` stays `@pending`. Unbound or failing until their owners
land: the TypeScript compile and lint and the client's imports (M7,
plan 2610112057); the launcher's and room-view component's evidence
(M7); the peer component's (M8, plan 2610112058); the publish and
bridge components' (M9, plan 2610112059); the trusted-text check on
restore content (TrustedText, plan 2609292013).
