---
id: 2609292015
title: "M4: the hermetic compute kernel"
status: "🔲"
summary: >-
  A per-session Starlark kernel in a resource-limited worker with
  read-only, quarantine-aware access to the record.
model: opus
depends-on: [2609292012, 2609292007]
---
# M4: the hermetic compute kernel

## Goal

Let Claude run code over very long histories without growing the
prompt. The code gets no path to files, the network or processes.

## Context

Milestone M4. Scope: CMP-01 to CMP-07, all P1. The language follows the
S5 decision.

## Tasks

1. Kernel worker with step, wall-clock and memory limits: CMP-05
2. Persistent namespace and the MCP tools: CMP-01, CMP-02
3. Read-only built-ins honoring quarantine and scope: CMP-03, CMP-07
4. Hermeticity: CMP-04
5. Capped, enveloped, tainted print output: CMP-06

## Acceptance Criteria

- [ ] Every scenario in scope passes; none of them is still @pending
- [ ] Aggregation over a 10M-token history works without prompt growth
- [ ] The worker is the one os/exec user, allow-listed in depguard and
  the import test
