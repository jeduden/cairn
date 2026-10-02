---
id: 2609292004
title: "Spike S2: choose the pure-Go SQLite driver"
status: "✅"
summary: >-
  Benchmark modernc.org/sqlite against ncruces/go-sqlite3 for FTS5,
  cancellation, WAL concurrency, encryption and 10M-event
  performance.
model: opus
depends-on: []
---
# Spike S2: choose the pure-Go SQLite driver

## Goal

Choose the SQLite driver that ADR-07 leaves open, on measured evidence
at the scale NFR-03 and NFR-05 demand.

## Context

Resolves ADR-07, OQ-03 and OQ-04. The choice must keep CON-02 (static,
`CGO_ENABLED=0`) and SEC-01 (no network imports).

## Tasks

1. Build a synthetic 10M-event store with both drivers
2. Measure FTS5 BM25 search p95, ingestion throughput, and store
   overhead against NFR-03, NFR-04 and NFR-09
3. Check query cancellation by context deadline and WAL correctness
   under 50 concurrent writers
4. Survey an encrypting VFS for each driver (OQ-04)
5. Check each driver's import closure with the existing import test
6. Write the report and record the decision in ADR-07, whose Decision
   table names the driver module (ENG-18, ENG-26)

## Outcome

The [benchmark report](report.md) holds the numbers. Both drivers
passed the FTS5, cancellation and 50-writer WAL checks. On a large
host ncruces takes 11–21% longer at full-ranked search and ingests
21–31% fewer events per second in batches. On emulated reference
hardware the gaps are about 10% and 21%, and no target's outcome
turns on the driver. It wins:
modernc links `os/exec` and `net` into the binary, and only ncruces
offers an encrypting VFS.
[ADR-2609302341](../../docs/adr/ADR-2609302341-sqlite-driver.md)
records the choice.

Deviations from the tasks:

- Task 5 used a minimal binary per driver rather than the cairn
  import test, which bans the whole `net/` prefix. ncruces needs
  `net/url` and `net/netip` allow-listed there when M1 first links
  the store.
- The ADR is `proposed`, not `accepted`. ENG-18 fails on an accepted
  ADR whose module `go.mod` does not require. The M1 change that adds
  the module flips the status.

Handed on:

- M1 must bound BM25 ranking for NFR-03. Full ranking of a common
  term takes seconds with either driver. Ranking only the newest
  2,000 matches still reaches 268–299 ms p95 on emulated reference
  hardware.
- M1's hooks should commit their events together. Both drivers meet
  NFR-04 over 1M events at 100 per commit (10–13k events per second),
  while one event per commit ranges from 0.7k to 4.2k.
- Spike S8 freezes NFR-09's 1.5× store overhead. This schema measures
  1.96× before payload files count as stored text.
- The OQ-04 default and key management go to the security review.

No requirement closes and no scenario leaves `@pending`. The spike
gathers evidence for NFR-03, NFR-04, NFR-05 and NFR-09.

## Acceptance Criteria

- [x] A benchmark report is committed beside this plan
- [x] ADR-07 names the chosen driver; OQ-03 and OQ-04 are answered or
  re-planned
- [x] The chosen driver is justified by an ADR, proposed until the M1
  change that adds it to `go.mod` accepts it, and the ENG-18 scenario
  still passes
