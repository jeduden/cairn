---
n: 2
title: "Declared dependency directions between crates (ENG-02)"
status: "🔲"
result: false
---
# Phase 2: declared dependency directions between crates (ENG-02)

Requirement. ENG-02 (P0, Ver I, T): product code is a Cargo workspace,
one binary crate per executable and library crates with one
responsibility each; the allowed dependency directions are declared
and a CI check enforces them. The Go plan's depguard rules become a
declaration read beside every workspace manifest.

BDD coverage: `@ENG-02` comes off `@pending`. Its steps read the
declared directions, inject a crate depending on one its direction
forbids, and see the check fail naming both crates; every library
crate declares exactly one responsibility.

RED: the scenario fails on the missing declaration. GREEN: the
declaration, the check in the engineering gate tests, a drift case.

Gate: `@ENG-02` passes; a forbidden direction is a caught drift.
