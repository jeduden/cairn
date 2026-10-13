---
n: 8
title: "The core's reach evidence and unsafe code (ENG-16)"
status: "✅"
result: false
---
# Phase 8: the core's reach evidence and unsafe code (ENG-16)

Requirement. ENG-16 (P0, Ver T). CI gates on clippy, fmt, cargo-deny,
cargo-audit, the room view client's TypeScript checks and custom
checks. Every crate that parses untrusted content or decides trust
forbids unsafe code. Build-time checks refuse clocks and randomness in
derived code. They also refuse restore content built from anything
but trusted text. CI fails when a component's reach evidence is
missing, or shows more reach than the register grants.

What exists: the clippy, fmt, deny and audit jobs, the workspace's
`unsafe_code = "forbid"`, the clippy bans and the core's reach test.
This phase binds the core row's steps and adds what is missing for
it: a check that every crate keeps the unsafe ban and that CI runs
the core's reach evidence. The room view client arrives in M7 and
TrustedText with plan 2609292013, so `@ENG-16` stays `@pending`.

RED: the steps fail on a crate that drops the workspace lints and on a
CI workflow that skips the reach test. GREEN: the checks, drift cases.

Gate: the core row's steps pass with the scenario still pending.
