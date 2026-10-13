---
n: 3
title: "Deadlines on every blocking operation (ENG-03)"
status: "🔳"
result: false
---
# Phase 3: deadlines on every blocking operation (ENG-03)

Requirement. ENG-03 (P0, Ver I, T): no mutable global state, I/O
behind injectable traits, every blocking operation takes a deadline,
and code that computes derived artifacts reads no clock or
randomness.

What exists: the clippy bans on clocks and hash collections in each
product crate, and two gate tests that hold every crate under
`crates/` to them. The Go plan's custom analyzers become these lists
and gate tests.

BDD coverage: `@ENG-03` comes off `@pending`. Its global-state and
clock steps bind to the existing checks. A new check fails the build
on a blocking call without a deadline.

RED: the scenario fails on the missing deadline check. GREEN: the
deadline check, the bindings, a drift case.

Gate: `@ENG-03` passes; a deadline-less blocking call is a caught
drift.

Status: open. The deadline check and its drift cases landed. But
`@ENG-03` is back on `@pending` until the stakeholder rules. Its step
"every public blocking operation takes a deadline" is bound to a ban
list. The list exempts local file I/O, in-process locks and standard
output, so the binding does not exercise the whole row.
