---
n: 3
title: "Deadlines on every blocking operation (ENG-03)"
status: "✅"
result: true
summary: >-
  The core's clippy configuration bans 15 blocking calls that take no deadline, a gate test holds every product crate to them, and the ENG-03 scenario is off pending.
---
## Handoff

What landed:

- `crates/cairn/clippy.toml` bans the standard library's blocking
  calls that take no deadline, each reason naming the deadline form:
  channel receive and send, condition and barrier waits, thread join
  and park, child waits, `Command::output` and `status`, file locks and
  `io::stdin`.
- Not banned, by decision: local file reads and writes (bounded by the
  disk and the hook budget), `thread::sleep` (it carries its own bound)
  and in-process locks. `docs/development.md` says so.
- The gate test `the_product_crates_ban_every_wait_with_no_deadline`
  holds every product crate to the list; five drift cases guard it.
- `@ENG-03` is off `@pending`, bound in `engineering_analyzers.rs`; its
  global-state and clock steps bind to the existing checks.

What remains: "all I/O through injectable traits" has no mechanical
check, and a crate's own blocking loop or a dependency's is not seen by
the list.
