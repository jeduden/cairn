---
id: 2609292012
title: "M1: record and recall"
status: "🔲"
summary: >-
  The append-only record of writer logs under seat keys a node's
  device key certifies, provenance, trust and origin, pull-only recall
  over MCP, and the security and operations floor under them.
milestone-order: 1
model: opus
depends-on: [2609292003, 2609292004, 2609292005, 2609292009, 2609292010, 2610012322]
---
# M1: record and recall

## Goal

Record every agent run in writer logs under seat keys its node's
device key certifies. Claude recalls exact history through MCP recall
tools, inside the envelope. Nothing is lost, and every failure is
counted.

## Context

Milestone M1 of [§12.2](../../docs/srs/12-delivery-plan.md). This plan
replaces the one written before SRS 2.0. That one stored events in one
table keyed by a local sequence. SRS 2.0 to 3.0 put the record on the
seats, writers and keys of the [domain model](../../docs/domain-model/index.md),
under the invariants [ADR-2610112209] and [ADR-2610112004] approved
and the key event log of [ADR-2610122234]:

- A node's device key starts its genesis log, a key event log
  (PRV-12), and certifies every seat key with a seat certificate
  (PRV-11).
- Each seat has its own writer. Each event's address is (writer,
  `seq`), on a hash chain over a header that carries a commitment
  instead of content (REC-06, REC-10, REC-17).
- The personal room exists from the first start. A witnessed run's MCP
  server records it in its run seat's writer. An ingest seat records what
  `cairn ingest` reads, with origin `ingested` (REC-19, REC-22).
- Origin and trust derive from the record and the node's key set
  alone (PRV-02, PRV-13). The search index rebuilds identical in content in
  any arrival order (I10, [ADR-2610112004]).

The language and the shell are settled: [ADR-2610050528] and
[ADR-2610042341] were accepted on 14 October 2026. OQ-32's open packaging
question decides how the components ship, not what M1's core does.

### Scope

§12.2 lists M1's scope, and this plan takes all of it. These rows
land in full:

- REC P0, PRV P0 (PRV-01 to PRV-06, PRV-08, PRV-09, PRV-11, PRV-14)
  and RCL P0, with RCL-08
- MEM-01, SEC-02 to SEC-08, SEC-10, SEC-12, SEC-15 to SEC-19 and SEC-22
- OPS-01 to OPS-03, ADM-04, ADM-05, ADM-09, ADM-11 and ADM-13
- LANE-01, LANE-02 and OWN-01

These land in part, as §12.2 bounds them or as their scenarios reach past M1:

| Row            | M1's part                                                               | The rest                            |
| -------------- | ----------------------------------------------------------------------- | ----------------------------------- |
| REC-19         | ingest seats, holding a run's events, closing segments, verify's report | sealing, M7                         |
| LANE-16        | the owner's device seat, a run seat's contributor role                  | other roles, M7                     |
| LANE-23        | how a seat id derives                                                   | joins, M2 and M7                    |
| LANE-31        | the rule for two branch links naming one branch                         | room state, M7                      |
| SEC-01         | the core's row: no socket, no program start, one component per process  | the B1 components, M7               |
| SEC-27         | the new seat and writer a key minted in place starts                    | rotation, M7                        |
| OWN-11, OWN-12 | the act classes; widening acts only through the CLI at a terminal       | other surfaces, M7                  |
| OWN-22         | the precondition: every run counts as unsandboxed                       | sandbox state, M7                   |
| PRV-12         | the key event log, rotation and recovery                                | M3, M7, M8, PRV-10                  |
| PRV-13         | origin from the record and the key set                                  | arrival, M3, M8, M9                 |
| ENG-08         | every property but the restore delimiters'                              | restore delimiters, M2              |
| ENG-10         | the envelope's golden files                                             | the restore block's, M2             |
| ENG-17         | the contract replay, drift and residual-identifier checks               | the nightly run through restore, M2 |

A scenario of a row that lands in part stays `@pending`, its M1 part
proven by its bound steps or by named tests. That follows the pattern
of ENG-12 in plan 2609292011. A scenario that cannot pass as written
is a finding to raise, never one to re-pend.

Each test kind lands with the code it tests: the fuzz targets of
ENG-07, the properties of ENG-08, crash consistency (ENG-06), the
contract replay (ENG-17) and the benchmarks of NFR-01 to NFR-05
(ENG-15). Plan 2609292011 built the other ENG gates. It keeps ENG-03
and ENG-15 open until M1 adds injectable I/O traits and benchmarks of
real code, so M1 no longer depends on it. M1's last phase closes both
with it instead. The CON rows need no scenario of their own: the
reach test and the release build prove CON-02 to CON-04, and the
record's laws (REC-06, REC-10, ENG-08) prove CON-05.

### What the spikes left

- **S2** ([ADR-2610110553], proposed): `rusqlite` without default
  features, SQLite built without extension loading, and a C compiler
  plus both musl targets' headers pinned on the Linux builder. BM25
  ranks only the newest 2,000 matches, 172 ms p95 against NFR-03.
  Ingest commits 100 events per transaction, 26.1k events per second.
- **S7** ([ADR-2610102211], proposed): an in-house stdio MCP server on
  `serde_json`. The reach test then allows `serde_json` and its four
  crates.
- **S1, S3 and S13**: Claude Code 2.1.296's hook inputs and
  transcripts under `testdata/claude-code`. The transcript parser gets
  its cargo-fuzz target in the change that adds it, seeded from them
  (ENG-07). A fork's repeated lines are recorded once (REC-04). The
  MCP server ingests live (REC-13). A line that drifts from its
  version's shape stays untrusted `harness_text` (REC-26).
- **S8**: the conservative model-token estimator of §8.4. Six targets
  freeze when M1 builds what uses them (§12.3): NFR-09's store
  overhead, REC-11's indexing cap, REC-17's secret, RCL-02's hits,
  RCL-03's cap and SEC-04's terms.
- **S12** (plan 2610012322): search-index row ids derive from the address,
  never arrival. A purge sets SQLite's `secure_delete` and FTS5's
  `secure-delete` and checkpoints the WAL. The trust of a `user` event
  needs the recorded deployment mode.

### Decisions this plan needs

1. **The dependency budget (ENG-18).** ENG-18 caps direct dependencies
   at ten, and three serve tooling today. M1 needs SQLite, `tracing`
   (ENG-05), TOML (ADM-04), SHA-256, Ed25519 signatures, a random
   source, and `libfuzzer-sys` (ENG-07). Key stores per OS (SEC-10)
   add more. Phase 1 lists the set; the stakeholder rules before an
   eleventh crate lands.
2. **Key stores (SEC-10).** Linux's Secret Service is reached over
   D-Bus, a socket the core may not open (I4, SEC-01). Linux then
   keeps keys in `0600` files, and `cairn status` says so. macOS
   Keychain and Windows need `unsafe` platform calls, which the
   workspace forbids outside listed shims. Phase 5 records the
   choice in an ADR.
3. **Migrations before backup (ADM-05).** A migration needs ADM-06's
   verified backup, which lands in M3. Phase 6 builds the automatic
   verified backup a migration runs first; the `cairn backup` verbs
   stay in M3.
4. **Purge before M3.** REC-17, RCL-06 and M1's exit criterion need a
   purge's record mechanics. The `cairn purge` verb is ADM-07, in M3.
   Phase 14 builds the tombstone and the erasure of the commitment
   key.

## Tasks

1. Proving slice: one recorded transcript ingested into an ingest
   seat's writer and recalled through `event_get` in the envelope, the
   chain checked by `cairn verify`
2. The home, the audit log and counters: SEC-02, SEC-03, OPS-02,
   OPS-03; typed errors per counted failure class (ENG-04)
3. Principal acts at the terminal, configuration and managed policy:
   OWN-11, OWN-12 and OWN-22 in part, ADM-04, PRV-04, PRV-05, SEC-22;
   the configuration parser's fuzz target
4. Redaction before anything is written, input limits and paths:
   SEC-05, SEC-08, SEC-18; logs through redaction (ENG-05); the
   redactor's fuzz target
5. Keys, seats and the personal room: PRV-09, PRV-11, PRV-12 in part,
   SEC-10, SEC-27 in part, REC-24, LANE-16 and LANE-23 in part; the
   key-store ADR, with Miri over any `unsafe` shim (ENG-09)
6. The store: writer logs, commitments, payloads, segments and
   migrations: REC-06, REC-09, REC-12, REC-17's commitments, ADM-05,
   REC-19's segment closing; the gap-free `seq` property (ENG-08);
   crash consistency over ingest and migration (ENG-06); freeze
   NFR-09's overhead and REC-17's secret
7. The transcript parser and provenance: PRV-01, PRV-03, PRV-08,
   REC-05, REC-26; the parser's fuzz target and its nightly run; the
   transcript contract replay and the residual-identifier check
   (ENG-17)
8. `cairn ingest`: REC-01 to REC-04, REC-07, REC-08, REC-22; the
   properties of a harness fork's ingest order and of ingesting twice
   (ENG-08); the NFR-04 benchmark
9. The hook handlers: SEC-16, the session binding, prompt commitments
   and the attended mark (PRV-04, PRV-08), holding a run's
   hook observations, held at its ingest marker, until its MCP server
   holds its run-seat key (REC-19);
   the hook input decoder's fuzz target and the hook input contract
   replay (ENG-17); the NFR-01 and NFR-02 benchmarks
10. The MCP server and live ingest: REC-13, PRV-14, run-seat keys in
    memory (SEC-10), concurrent appends (REC-06), with the gap-free
    `seq` property rerun under concurrency
11. Trust, origin, branch links and repository identity: PRV-02,
    PRV-06, PRV-13 in part, LANE-01, LANE-02, LANE-31 in part, OWN-01,
    MEM-01, ADM-13; the field sanitizer's fuzz target
12. Search: REC-11, SEC-04, RCL-02 with bounded BM25; the query
    parser's fuzz target; the rebuild-digest property (ENG-08); the
    NFR-03 benchmark; freeze REC-11's cap, RCL-02's hits and SEC-04's
    terms
13. Recall and the envelope: RCL-01, RCL-03 to RCL-05, RCL-07, RCL-08,
    SEC-06, SEC-07; the estimator ported from S8; the envelope's golden
    files (ENG-10), its round-trip property and its encoder's fuzz
    target; freeze RCL-03's cap
14. Quarantine and purge's record mechanics: SEC-12, RCL-06, REC-17;
    crash consistency over purge (ENG-06)
15. Operations: ADM-09, ADM-11, OPS-01, SEC-01's core row, SEC-15,
    SEC-17, SEC-19; every privacy statement tied to its test (ENG-22)
16. Exit: the NFR-05 benchmark; ENG-11's coverage floors; ENG-23's
    migration tests; the `cairn-eval` module of §11.1 and its recall
    evaluation; the two-writer rebuild test (S12)

## Execution

Phase 1 runs first and fixes the test approach the others copy. Then
four tracks run in parallel: 2 then 3; 4; 5 then 6; 7. Phases 8 to 16
follow in order, but 12 may start once 8 lands. Each later phase's
spec is written once the phase before it hands off.

| Phase | Model  | Gate                                                                                                                                                                |
| ----- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1     | opus   | The slice's end-to-end test in an isolated home; `@REC-10` passes                                                                                                   |
| 2     | sonnet | `@SEC-02`, `@SEC-03`, `@OPS-02`, `@OPS-03` and `@ENG-04` pass                                                                                                       |
| 3     | opus   | `@ADM-04`, `@PRV-04`, `@PRV-05` and `@SEC-22` pass; OWN-11, OWN-12 and OWN-22's M1 steps bound and passing                                                          |
| 4     | sonnet | `@SEC-05`, `@SEC-08`, `@SEC-18` and `@ENG-05` pass; no fixture secret reaches the store or a log                                                                    |
| 5     | opus   | `@PRV-09`, `@PRV-11`, `@SEC-10` and `@REC-24` pass; PRV-12, SEC-27, LANE-16 and LANE-23's M1 steps bound and passing                                                |
| 6     | opus   | `@REC-09`, `@REC-12` and `@ADM-05` pass; REC-19's segment steps pass; the crash-consistency job runs nightly                                                        |
| 7     | opus   | `@PRV-01`, `@PRV-03`, `@PRV-08`, `@REC-05` and `@REC-26` pass; the parser's fuzz target and the residual-identifier drift case run                                  |
| 8     | opus   | `@REC-01` to `@REC-04`, `@REC-07`, `@REC-08` and `@REC-22` pass; both properties hold; NFR-04 benchmarked                                                           |
| 9     | opus   | `@SEC-16` passes; REC-19's holding steps pass; a hook handler fails open within its budget; NFR-01 and NFR-02 benchmarked                                           |
| 10    | opus   | `@REC-06`, `@REC-13` and `@PRV-14` pass against a live MCP server                                                                                                   |
| 11    | opus   | `@PRV-02`, `@PRV-06`, `@LANE-01`, `@LANE-02`, `@OWN-01`, `@MEM-01` and `@ADM-13` pass; PRV-13 and LANE-31's M1 steps bound and passing                              |
| 12    | sonnet | `@REC-11`, `@SEC-04` and `@RCL-02` pass; NFR-03 benchmarked                                                                                                         |
| 13    | opus   | `@RCL-01`, `@RCL-03` to `@RCL-05`, `@RCL-07`, `@RCL-08`, `@SEC-06` and `@SEC-07` pass; the envelope's golden files match                                            |
| 14    | opus   | `@SEC-12`, `@RCL-06` and `@REC-17` pass; a purged event leaves nothing confirming a guess                                                                           |
| 15    | sonnet | `@ADM-09`, `@ADM-11`, `@OPS-01`, `@SEC-15`, `@SEC-17`, `@SEC-19` and `@ENG-22` pass; SEC-01's core row passes                                                       |
| 16    | opus   | `@ENG-03`, `@ENG-06`, `@ENG-07`, `@ENG-09`, `@ENG-11`, `@ENG-15`, `@ENG-23` and `@ENG-29` pass; `cairn-eval` reaches 95% recall; the two-writer rebuild test passes |

## Phases

<?catalog
glob:
  - "phase-*.md"
  - "phase-*.result.md"
sort: numeric:n
header: |

  | # | Status | Phase |
  |---|--------|-------|
row-expr: |
  [if result {
    "|  | ↳ | \(summary) |"
  }, if !result {
    "| \(n) | \(status) | [\(title)](phase-\(n).md) |"
  }][0]
footer: |

?>

| #   | Status | Phase                                                                       |
| --- | ------ | --------------------------------------------------------------------------- |
| 1   | 🔲     | [Proving slice: one transcript recorded, recalled and verified](phase-1.md) |
<?/catalog?>

## Acceptance Criteria

- [ ] Every scenario of a row M1 lands in full passes, off `@pending`
- [ ] Each row M1 lands in part has its M1 part proven, and its
  scenario names what keeps it pending
- [ ] Recall of at least 95% after 1 to 3 compactions on the §11.1
  evaluation
- [ ] `cairn verify` proves completeness, and quarantine is effective
- [ ] Two writers' logs rebuild one search index identical in content
  in either order (S12)
- [ ] A purged event leaves no value that confirms a guess at it
- [ ] Every dependency M1 adds has an accepted ADR (ENG-18, ENG-26),
  within the budget the stakeholder rules
- [ ] ENG-11's coverage floors hold for every security-sensitive crate
- [ ] The fuzz targets of ENG-07 run nightly from committed corpora
- [ ] The six targets §12.3 assigns to M1 are frozen, each with its
  evidence
- [ ] @ENG-03 and @ENG-15 pass, closing plan 2609292011's phases 3
  and 4
- [ ] All tests pass: `cargo test --workspace`
- [ ] `cargo clippy --workspace --all-targets --locked -- -D warnings`
  and `cargo fmt --all --check` are clean
- [ ] `mdsmith check .` is clean

[ADR-2610112209]: ../../docs/adr/ADR-2610112209-invariants-after-s1-s3.md
[ADR-2610122234]: ../../docs/adr/ADR-2610122234-key-event-log.md
[ADR-2610112004]: ../../docs/adr/ADR-2610112004-derived-artifacts-by-content.md
[ADR-2610050528]: ../../docs/adr/ADR-2610050528-language-rust-typescript.md
[ADR-2610042341]: ../../docs/adr/ADR-2610042341-app-shell-tauri.md
[ADR-2610110553]: ../../docs/adr/ADR-2610110553-sqlite-rusqlite.md
[ADR-2610102211]: ../../docs/adr/ADR-2610102211-mcp-server-in-house.md
