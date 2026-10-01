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

Spike S2 chose `github.com/ncruces/go-sqlite3`
([report](2609292004_spike-s2-sqlite-driver/report.md)). It leaves
four things for M1:

- Task 1 adds the module to `go.mod` and flips
  [ADR-2609302341](../docs/adr/ADR-2609302341-sqlite-driver.md) to
  `accepted` in the same change.
- That change also allow-lists `net/url` and `net/netip`, the
  driver's URI parsers, in the SEC-01 import test. FTS5 is registered
  on every connection.
- Task 6 bounds BM25 ranking. Full ranking of a common term at 10M
  events takes seconds, far over NFR-03's 200 ms. Ranking only the
  newest 2,000 matches still reaches 268–299 ms p95 on emulated
  reference hardware.
- Task 2's ingestion commits a hook's events together. 100 events per commit
  met NFR-04 over 1M events (10–13k events per second), while one
  event per commit ranged from 0.7k to 4.2k.

## Tasks

1. Store, payload CAS and hash chain: REC-06, REC-09, REC-10, REC-12,
   ADM-05
2. Ingestion through the harness adapter: REC-01 to REC-05, REC-07,
   REC-08, REC-11
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
