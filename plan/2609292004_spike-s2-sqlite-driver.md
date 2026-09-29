---
id: 2609292004
title: "Spike S2: choose the pure-Go SQLite driver"
status: "🔲"
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

## Acceptance Criteria

- [ ] A benchmark report is committed beside this plan
- [ ] ADR-07 names the chosen driver; OQ-03 and OQ-04 are answered or
  re-planned
- [ ] The chosen driver is justified by an accepted ADR and the ENG-18
  scenario still passes
