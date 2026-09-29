---
id: 2609292016
title: "M5: hardening and evaluation"
status: "🔲"
summary: >-
  The remaining P1 requirements, the full §11 evaluation, an
  external security review and a two-week pilot.
model: opus
depends-on: [2609292014, 2609292015, 2609292008]
---
# M5: hardening and evaluation

## Goal

Reach the v1.0 definition of done: every P0 verified, every P1 done or
signed off, the evaluation published and the pilot clean.

## Context

Milestone M5. Scope: PRV-07, PIN-05, PIN-07, PIN-09, REC-13 to REC-15,
SEC-09, SEC-13, OPS-04, OPS-05, ADM-12, ENG-13, ENG-25, the §11
evaluation, the external review and the pilot.

## Tasks

1. Instruction-like flagging: PRV-07
2. Pin candidates, compaction guidance, CLAUDE.md duplicate warning:
   PIN-05, PIN-07, PIN-09
3. Incremental hook ingestion, SDK transcripts, retention: REC-13 to
   REC-15
4. Encryption at rest and the recall-taint policy hook: SEC-09, SEC-13
5. Canary, structured logs, trusted-only export: OPS-04, OPS-05, ADM-12
6. Mutation testing and the maintainer runbook: ENG-13, ENG-25
7. Run the §11.1 evaluation and publish it
8. External security review and the two-week pilot

## Acceptance Criteria

- [ ] The §11.2 definition of done holds
- [ ] No scenario tagged P0 or P1 is still @pending, or each remaining
  one is deferred with signed-off rationale
