---
id: 2609292012
title: "M1: record and recall"
status: "🔲"
summary: >-
  The append-only record, provenance and trust, pull-only recall
  over MCP, and the security and operations floor under them.
model: opus
depends-on: [2609292003, 2609292004, 2609292005, 2609292009, 2609292010, 2609292011, 2610012322]
---
# M1: record and recall

## Goal

Ingest every transcript into a hash-chained record that tags each
event with its source. Let Claude recall exact history through MCP
tools that wrap every result. Lose nothing, and fail loudly.

## Context

Milestone M1 of [docs/srs/12-delivery-plan.md](../docs/srs/12-delivery-
plan.md). Scope: REC P0, PRV P0, RCL P0, SEC-01 to SEC-08, SEC-12,
SEC-16, SEC-18, OPS-01 to OPS-03, and ADM-04, ADM-05, ADM-09 and ADM-11.
Split into phases when started.

Spike S2, rerun for the Rust core, measured `rusqlite` with SQLite
bundled ([report](2609292004_spike-s2-sqlite-driver/report.md),
[ADR-2610110553](../docs/adr/ADR-2610110553-sqlite-rusqlite.md),
proposed). It leaves five things for M1:

- Task 1 adds `rusqlite` without its default features and flips
  ADR-2610110553 to `accepted` in the same change. The reach test
  allows its six crates.
- That change builds SQLite with extension loading compiled out, so
  no `dlopen` links into the core.
- It also pins a C compiler and both musl targets' headers on the
  Linux builder, since bundled SQLite does not build with `rust-lld`
  alone.
- Task 6 bounds BM25 ranking. Full ranking takes 1.45 s p95 on
  emulated reference hardware. Ranking only the newest 2,000 matches
  meets NFR-03 at 172 ms p95, with a 548 ms p99 to watch.
- Task 2's ingestion commits a hook handler's events together. 100
  events per commit met NFR-04 over 1M events at 26.1k events per
  second, while one event per commit ran at 4.2k.

Spike S7 chose an in-house stdio MCP server on `serde_json`
([report](2609292009_spike-s7-mcp-sdk/report.md),
[ADR-2610102211](../docs/adr/ADR-2610102211-mcp-server-in-house.md)).
Task 6 links it into `cairn`: the reach test allows `serde_json` and
its four crates, and the ADR that lists `serde_json` says it links
into the core.

Spikes S1 and S3 recorded the hook inputs and transcripts of Claude
Code 2.1.296 under `testdata/claude-code`. The `@ASM-*` scenarios replay
them; the [S1 report](2609292003_spike-s1-hook-contract/report.md) and
the [S3 report](2609292005_spike-s3-transcript-formats/report.md)
explain them. They leave
two things for M1:

- Task 2's transcript parser gets its cargo-fuzz target in the same
  change (ENG-07), seeded under `fuzz/corpus` from
  `testdata/claude-code/*/transcripts`. That change adds the `fuzz/`
  crate with its `libfuzzer-sys` dependency ADR (ENG-18, ENG-26), pins
  the dated nightly ENG-01 requires for it, and makes the nightly job
  run it. S3 handed this over so that M1 does not wait on a spike that
  waits on M1's parser.
- A fork repeats a transcript's lines with their `uuid`s in a new
  transcript. Task 2 decides how ingest records them, after the
  stakeholder's ruling on a harness fork.

## Tasks

1. Store, payload CAS and hash chain: REC-06, REC-09, REC-10, REC-12,
   ADM-05
2. Ingestion through the harness adapter: REC-01 to REC-05, REC-07,
   REC-08, REC-11; the transcript parser's fuzz target (ENG-07)
3. Provenance, trust policy and taint: PRV-01 to PRV-06
4. Redaction before storage, input limits and path validation: SEC-05,
   SEC-08, SEC-16, SEC-18
5. Home permissions and tenant binding: SEC-02, SEC-03
6. Query compiler, recall tools and envelope: RCL-01 to RCL-07, SEC-04,
   SEC-06
7. Quarantine: SEC-12
8. Audit log, counters, configuration, verify and status: OPS-01 to
   OPS-03, ADM-04, ADM-09, ADM-11

## Acceptance Criteria

- [ ] Every scenario in scope passes; none of them is still @pending
- [ ] Recall of at least 95% after 1 to 3 compactions on the §11.1
  evaluation
- [ ] `cairn verify` proves completeness, and quarantine is effective
- [ ] ENG-11 coverage floors hold for every security-sensitive package
  landed
- [ ] The transcript parser's fuzz target runs nightly from the corpus
  S3 recorded, and its row of @ENG-07 passes
