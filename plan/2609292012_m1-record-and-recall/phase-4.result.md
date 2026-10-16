---
n: 4
title: "Redaction before anything is written, input limits and paths"
status: "✅"
result: true
summary: >-
  @ENG-05 off @pending and passing. cairn ingest redacts every line
  before it writes, reads only within the transcript roots, and a
  release build ingests again; hook input past 1 MiB or depth 64 is
  refused. SEC-05 rows 1-2 and SEC-08's ingest steps pass narrowed.
---
# Phase 4 result

## Handoff

Outcome:

- ENG-05 holds: `@ENG-05` is off `@pending`, its scenario passing
  against the built core, confined.
- SEC-08's M1 part holds in `cairn ingest`: the rule set, version 1,
  the harness account identifiers and the principal's redaction
  patterns, each match `[REDACTED:<rule>]`, each redaction audited.
  `harness_account.rs` finds no identifier from any recorded
  transcript in the store.
- SEC-05's rows 1 and 2 and SEC-18's transcript roots hold;
  `e2e_redaction.rs` drives all three, confined.
- `@SEC-05` rows 1 and 2 and SEC-08's ingest steps pass with their
  scenarios narrowed locally; none comes off `@pending`.
- I1, I6, I8 and I9 hold: only secrets leave the record, every
  redaction and refusal is counted, nothing outside the roots is read,
  and the hook still fails open.

Inherited:

- The `redaction` crate: `Redactor::new(&patterns)`,
  `with_identifiers`, `redact_line`, `redact_text`;
  `HarnessAccountIdentifiers::learn`.
- `allowed_root`: `AllowedRoots::with`, `with_worktree`, `resolve`.
  Phase 9's session binding adds the worktree root.
- `structured_log`: `StructuredLog`, installed per run by `main`.
- `cairn::bounded_lines` caps every input line at 1 MiB; `cairn mcp`
  answers a longer request and goes on.
- Counters `redacted`, `redaction_pattern_refused`,
  `hook_input_too_large`, `hook_input_too_deep`,
  `path_outside_allowed_roots`, `log_line_dropped`.
- `testkit::place_transcript` puts a fixture under the isolated home's
  transcript root, which tests now need.

For the stakeholder:

- `@SEC-08` cannot pass in M1: `cairn purge --run` is ADM-07's run
  scope, in M3.
- The domain model defines neither "allowed root" nor "structured
  log".
- When the principal's configuration fails to load, ingest records
  without its redaction patterns (ADM-04's fail-open). Refuse instead?

Parked:

- The redactor's fuzz target, to phase 7's first `fuzz/` crate.
- Streaming a long transcript line (SEC-05 row 3), to phase 6.
- Ingest reads a path after resolving it, so a same-user swap can race
  it, and a FIFO in a root blocks the read: phase 8's ingest.
- Ingest failures other than refusals reach stderr, unaudited.
- Logs redact with the rule set alone, not the principal's patterns.
- Redactions are audited just after the write, not with it.
