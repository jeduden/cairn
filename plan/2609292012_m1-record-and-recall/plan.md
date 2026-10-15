---
id: 2609292012
title: "M1: record and recall"
status: "🔳"
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
- MEM-01, SEC-02 to SEC-08, SEC-12, SEC-15 to SEC-19 and SEC-22
- OPS-01 to OPS-03, ADM-04, ADM-09 and ADM-11
- LANE-02 and OWN-01

These land in part, as §12.2 bounds them or as their scenarios reach past M1:

| Row            | M1's part                                                               | The rest                             |
| -------------- | ----------------------------------------------------------------------- | ------------------------------------ |
| REC-19         | ingest seats, holding a run's events, closing segments, verify's report | sealing, M7                          |
| LANE-16        | the owner's device seat, a run seat's contributor role                  | other roles, M7                      |
| LANE-23        | how a seat id derives                                                   | joins, M2 and M7                     |
| LANE-31        | the rule for two branch links naming one branch                         | room state, M7                       |
| SEC-01         | the core's row: no socket, no program start, one component per process  | the B1 components, M7                |
| SEC-27         | the new seat and writer a key minted in place starts                    | rotation, M7                         |
| OWN-11, OWN-12 | the act classes; widening acts only through the CLI at a terminal       | other surfaces, M7                   |
| OWN-22         | the precondition: every run counts as unsandboxed                       | sandbox state, M7                    |
| PRV-12         | the key event log, rotation and recovery                                | M3, M7, M8, PRV-10                   |
| PRV-13         | origin from the record and the key set                                  | arrival, M3, M8, M9                  |
| ENG-08         | every property but the restore delimiters'                              | restore delimiters, M2               |
| ENG-10         | the envelope's golden files                                             | the restore block's, M2              |
| ENG-17         | the contract replay, drift and residual-identifier checks               | the nightly run through restore, M2  |
| ADM-05         | versions, and the refusal of a newer one                                | migration and its backup, M3         |
| ADM-07         | `cairn purge --range`                                                   | other scopes, SEC-31, seals, M3      |
| SEC-10         | keys kept per ruling 2; no key in the store, a log or output            | backup M3, at-rest key M5, launch M7 |
| LANE-01        | routing to personal-room and device seats; branch links                 | joins, M2 and M7                     |
| ADM-13         | the core writes to no git repository                                    | install M2; commit hook, launcher M7 |

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

### The stakeholder's rulings of 14 October 2026

The stakeholder ruled on these in the session that re-planned M1, and
consents to them on [pull request #46][pr46]. Rulings 3 and 4 also
changed M1's row in §12.2.

1. **The dependency budget (ENG-18).** ENG-18's target of ten counts
   only direct dependencies linked into a shipped executable. Test,
   fuzz and tooling crates still need an accepted ADR and an allowed
   license, but do not count. ENG-18 and its gate are stakeholder
   paths, so their rewording, with a drift case, lands as its own pull
   request before phases 4, 5 and 7 add crates past ten. The
   core then holds about nine: `rusqlite`, `serde_json`, `tracing`,
   `toml`, `sha2`, `ed25519-dalek`, `getrandom` and one key-store crate
   per OS.
2. **Key stores (SEC-10).** macOS keeps keys in its Keychain through
   `security-framework`, whose `unsafe` stays inside that crate. Windows
   keeps them in a file DPAPI protects, through `windows-sys` in one
   listed `unsafe` shim that Miri checks (ENG-09). Linux's Secret
   Service needs D-Bus, a socket the core may not open (I4), and its
   kernel keyring does not survive a reboot. Linux therefore keeps
   keys in `0600` files, and `cairn status` says so. Phase 5 records
   this in an ADR. `@SEC-10`'s scenario also needs a backup (M3), the
   at-rest key (SEC-09, M5) and keys handed over at launch (M7), so
   SEC-10 lands in part.
3. **Migrations (ADM-05).** M1 ships format version 1, so nothing
   migrates yet. M1 versions every segment and the store, and refuses
   and names anything newer; `cairn status` then exits 3. `cairn
   migrate` and the ADM-06 backup it runs first come with the first
   real format change, in M3. ADM-05 lands in part.
4. **Purge (ADM-07).** M1 ships `cairn purge --range` as a widening
   principal act. Phase 14 builds the tombstone, the erasure of each
   purged event's commitment key and payload, SQLite's and FTS5's
   secure delete with a WAL checkpoint, the audit entry, and the
   refusal of a range that holds a pending ingest marker. ADM-07's
   other scopes, SEC-31's copies and seal rewrites stay in M3. ADM-07
   lands in part.

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
   redactor's fuzz target. It lifts phase 1's refusal of `cairn ingest`
   in a release build
5. Keys, seats and the personal room: PRV-09, PRV-11, PRV-12 in part,
   SEC-10, SEC-27, LANE-16 and LANE-23 in part, REC-24; the
   key-store ADR and the three key stores, with Miri over the Windows
   shim (ENG-09)
6. The store: writer logs, commitments, payloads, segments and format
   versions: REC-06, REC-09, REC-12, REC-17's commitments, ADM-05 in part,
   REC-19's segment closing; the gap-free `seq` property (ENG-08);
   crash consistency over ingest (ENG-06); freeze
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
    PRV-06, PRV-13, LANE-01 and LANE-31 in part, LANE-02, OWN-01,
    MEM-01, ADM-13 in part; the field sanitizer's fuzz target
12. Search: REC-11, SEC-04, RCL-02 with bounded BM25; the query
    parser's fuzz target; the rebuild-digest property (ENG-08); the
    NFR-03 benchmark; freeze REC-11's cap, RCL-02's hits and SEC-04's
    terms
13. Recall and the envelope: RCL-01, RCL-03 to RCL-05, RCL-07, RCL-08,
    SEC-06, SEC-07; the estimator ported from S8; the envelope's golden
    files (ENG-10), its round-trip property and its encoder's fuzz
    target; freeze RCL-03's cap
14. Quarantine and `cairn purge --range`: SEC-12, RCL-06, REC-17,
    ADM-07 in part; crash consistency over purge (ENG-06)
15. Operations: ADM-09, ADM-11, OPS-01, SEC-01's core row, SEC-15,
    SEC-17, SEC-19; every privacy statement tied to its test (ENG-22)
16. Exit: the NFR-05 benchmark; ENG-11's coverage floors; ENG-23's
    migration tests; the `cairn-eval` module of §11.1 and its recall
    evaluation; the two-writer rebuild test (S12)

## Execution

Phase 1 runs first and fixes the test approach the others copy.
ENG-18's rewording (ruling 1) lands as its own pull request before
phases 4, 5 and 7. Then four tracks run in parallel: 2 then 3; 4; 5
then 6; 7. Phases 8 to 16 follow in order, but 12 may start once 8
lands. Each later phase's spec is written once the phase before it
hands off.

| Phase | Model  | Gate                                                                                                                                                    |
| ----- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1     | opus   | The slice's end-to-end test in an isolated home; `@REC-10` passes                                                                                       |
| 2     | sonnet | `@SEC-02`, `@SEC-03`, `@OPS-02`, `@OPS-03` and `@ENG-04` pass                                                                                           |
| 3     | opus   | Its end-to-end tests pass; so do ADM-04's rows 1 to 4 and the M1 steps of OWN-11, -12 and -22                                                           |
| 4     | sonnet | `@SEC-05`, `@SEC-08`, `@SEC-18` and `@ENG-05` pass; no fixture secret reaches the store or a log                                                        |
| 5     | opus   | `@PRV-09`, `@PRV-11` and `@REC-24` pass; PRV-12, SEC-10, SEC-27, LANE-16 and LANE-23's M1 steps bound and passing                                       |
| 6     | opus   | `@REC-09` and `@REC-12` pass; REC-19's segment steps pass; a newer segment or store is refused; the crash-consistency job runs nightly                  |
| 7     | opus   | `@PRV-01`, `@PRV-03`, `@PRV-08`, `@REC-05` and `@REC-26` pass; the parser's fuzz target and the residual-identifier drift case run                      |
| 8     | opus   | `@REC-01` to `@REC-04`, `@REC-07`, `@REC-08`, `@REC-22` and PRV-05's transcript rows pass; both properties hold; NFR-04 benchmarked                     |
| 9     | opus   | `@SEC-16` and `@PRV-04` pass; REC-19's holding steps pass; a hook handler fails open within its budget; NFR-01 and NFR-02 benchmarked                   |
| 10    | opus   | `@REC-06`, `@REC-13` and `@PRV-14` pass against a live MCP server                                                                                       |
| 11    | opus   | `@PRV-02`, `@PRV-06`, `@LANE-02`, `@OWN-01` and `@MEM-01` pass; PRV-13, LANE-01, LANE-31 and ADM-13's M1 steps pass                                     |
| 12    | sonnet | `@REC-11`, `@SEC-04` and `@RCL-02` pass; NFR-03 benchmarked                                                                                             |
| 13    | opus   | `@RCL-01`, `@RCL-03` to `@RCL-05`, `@RCL-07`, `@RCL-08`, `@SEC-06` and `@SEC-07` pass; the envelope's golden files match                                |
| 14    | opus   | `@SEC-12`, `@RCL-06`, `@REC-17` and `@SEC-22` pass; a purged event leaves nothing confirming a guess                                                    |
| 15    | sonnet | `@ADM-09`, `@ADM-11`, `@OPS-01`, `@SEC-15`, `@SEC-17`, `@SEC-19` and `@ENG-22` pass; SEC-01's core row passes                                           |
| 16    | opus   | `@ENG-03`, `@ENG-06`, `@ENG-07`, `@ENG-09`, `@ENG-11`, `@ENG-15`, `@ENG-23` and `@ENG-29` pass; 95% recall in `cairn-eval`; the S12 rebuild test passes |

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

| #   | Status | Phase                                                                                                                                                                                                  |
| --- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1   | ✅     | [Proving slice: one transcript recorded, recalled and verified](phase-1.md)                                                                                                                            |
|     | ↳      | @REC-10 off @pending and passing. A recorded transcript is ingested, recalled through event_get in the envelope and verified, end to end and confined; a flipped header byte exits 3 naming the event. |
| 2   | ✅     | [The home, the audit log and counters](phase-2.md)                                                                                                                                                     |
|     | ↳      | @SEC-02, @SEC-03, @OPS-02, @OPS-03 and @ENG-04 off @pending and passing, confined. Every command checks the home first; the audit log chains across rotation; cairn hook fails open and audits.        |
| 3   | 🔳     | [Settings layers and principal acts at the terminal](phase-3.md)                                                                                                                                       |
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
[pr46]: https://github.com/jeduden/cairn/pull/46
