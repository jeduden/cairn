---
id: 2609292007
title: "Spike S5: Starlark or Python for the kernel"
status: "🔲"
summary: >-
  Measure Claude's success writing Starlark versus Python on 30
  aggregation tasks and decide the kernel default.
model: opus
depends-on: []
---
# Spike S5: Starlark or Python for the kernel

## Goal

Decide on evidence whether the hermetic Starlark kernel of ADR-04 is
good enough for Claude, before M4 builds it.

## Context

Resolves ADR-04 and OQ-10.

## Tasks

1. Write 30 aggregation tasks over a fixture record, with reference
   answers
2. Run each with Claude in Starlark and in Python, three repetitions
   each
3. Report success rates with confidence intervals

## Acceptance Criteria

- [ ] A go/no-go on the Starlark default is recorded in ADR-04
- [ ] OQ-10 is answered
