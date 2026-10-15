---
n: 2
title: "The home, the audit log and counters"
status: "✅"
result: true
summary: >-
  @SEC-02, @SEC-03, @OPS-02, @OPS-03 and @ENG-04 off @pending and
  passing, confined. Every command checks the home first; the audit log
  chains across rotation; cairn hook fails open and audits.
---
# Phase 2 result

## Handoff

Outcome:

- SEC-02, SEC-03, OPS-02, OPS-03 and ENG-04 close: their five
  scenarios, eight runs, pass against the built core, confined. The
  workspace's 76 scenario runs pass, and `cargo run -p coverage` holds
  every crate at 100%.
- I6, I8 and I9 hold: a degraded operation is audited and counted, a
  refused home is left untouched, and a hook exits 0 on every path.

Inherited:

- `Context::home()` locates the home, then `Home::check` (owner-only,
  the running OS user's, no symbolic link inside) and
  `check_home_id` against `CAIRN_HOME_ID`. Every command goes through
  it; the running OS user is a pipe's owner.
- The `audit` crate: `Log::append(&Occurrence, time)`, `entries`,
  `verify`; `counters::counts`. An `Occurrence` holds only
  `&'static str`, so no content reaches the log. New counters join
  `audit::entry::Counter`; new kinds of failure each get a variant.
- The `hook` crate: `Hook`, `Failure`, `handle`, `guarded`. `cairn
  hook` reads the hook input with a 500 ms wait per line.
- `mcp::serve` takes the handler and a callback per caught panic.
- The testkit's `Confinement::run_in` takes environment variables; the
  steps' `run_core` and `last_run` live in `bdd/record.rs`, and
  `World::env` carries the variables.

Deviations, now in phase-2.md: counter names `hook_panicked` and
`mcp_panicked`, and the entry kind `counter_ack`, on the domain-model
agent's review.

For phase 3:

- `cairn counter ack` only appends a `counter_ack` entry. Record the
  neutral principal act in the device seat, ask for a terminal
  (OWN-12), and move doctor's last acknowledgement to it. The `@OPS-03`
  step then needs a terminal from the testkit.
- `home.json` is written by hand; configuration's `home.id` should
  write it.

Parked:

- A hook input that times out counts as `hook_rejected`; phase 9's
  hook budgets add `hook_budget_exceeded` (NFR-02).
- Audit appends are not synced to disk, and a torn last entry fails
  every later append until repaired (ENG-06, phase 16).
- A hook whose home is refused or whose audit log takes no entry
  leaves no trace; `cairn status` and `cairn doctor` name the cause.
- The home check walks every file, payloads included once phase 6
  writes them; it may need narrowing to fit the hook budget.
- Windows' ACLs are not checked yet.
- The `@SEC-02` steps for another OS user need root or `sudo -n`.
- OQ-50 (i): every counter so far counts as a failure counter.
