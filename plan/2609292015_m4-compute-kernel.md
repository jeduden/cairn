---
id: 2609292015
title: "M4: the hermetic compute kernel"
status: "🔲"
summary: >-
  A per-run SQL kernel in a resource-limited worker with
  read-only, quarantine-aware access to the record.
milestone-order: 5
model: opus
depends-on: [2609292012, 2609292007]
---
# M4: the hermetic compute kernel

## Goal

Let Claude run code over very long histories without growing the
prompt. The code gets no path to files, the network or processes.

## Context

Milestone M4. Scope: CMP-01 to CMP-07, all P1. The kernel speaks SQL,
by the S5 decision (ADR-04), on the core's SQLite.

## Tasks

1. Kernel worker with step, wall-clock and memory limits: CMP-05
2. Persistent namespace and the MCP tools: CMP-01, CMP-02
3. Read-only table functions honoring quarantine and scope: CMP-03, CMP-07
4. Hermeticity: CMP-04
5. Capped, enveloped, tainted rows of the last row-returning statement:
   CMP-06
6. M4's initial targets (§12.3): freeze CMP-05's limits and CMP-06's
   output cap, each with its evidence, removing its † and its §12.3
   row

## Acceptance Criteria

- [ ] Every scenario in scope passes; none of them is still @pending
- [ ] Aggregation over a 10M-token history works without prompt growth
- [ ] The worker is the one os/exec user, allow-listed in depguard and
  the import test
