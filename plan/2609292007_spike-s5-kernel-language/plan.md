---
id: 2609292007
title: "Spike S5: Lua or SQL for the kernel"
status: "✅"
summary: >-
  Measure Claude's success writing Lua versus SQL on 30 aggregation
  tasks over a fixture record and decide the kernel default.
model: opus
depends-on: []
---
# Spike S5: Lua or SQL for the kernel

## Goal

Decide on evidence which language the hermetic compute kernel (CMP-01
to CMP-07) speaks by default, before M4 builds it.

## Context

Resolves ADR-04 and OQ-10. ADR-04 chose a hermetic Starlark kernel
and named its risk: a dialect Claude writes less well than the
languages it knows. The stakeholder narrowed the candidates to two
before the spike began:

- **Lua**, embedded through `mlua`: a general-purpose language whose
  globals the host builds, with an instruction hook for the step
  budget and an allocator limit.
- **SQL**, on a read-only SQLite connection over the record: the store
  is SQLite already (ADR-2610110553), aggregation is SQL's purpose,
  and the progress handler and heap limit give the budgets.

Starlark and Python are out of scope. The question is whether Claude
writes correct aggregations more often in Lua or in SQL, and whether
SQL's weakness, step-by-step logic, shows in the numbers.

## Tasks

1. [x] Generate a fixture record and write 30 aggregation tasks over
   it, each with a reference answer, among them step-by-step tasks
   (streaks, sessions, state)
2. [x] Build one runner that executes Lua and SQL under the kernel's
   limits and returns the answer as JSON
3. [x] Have Claude solve each task in Lua and in SQL, three
   repetitions each, without running the code
4. [x] Give each failed run its error once, as `kernel_exec` would,
   and score the repair
5. [x] Report success rates with confidence intervals, overall and per
   task category

## Outcome

The [spike report](report.md) holds the numbers. Claude solved all 90
tasks in each language on the first run (95% interval 95.9% to 100%),
and its SQL was half the size of its Lua. The stakeholder chose SQL on
13 October 2026. ADR-04 records it in
[§4](../../docs/srs/04-reference-architecture.md), OQ-10 is answered in
[§13](../../docs/srs/13-open-questions-and-risks.md), and CMP-01,
CMP-02, CMP-03 and CMP-06 are restated for SQL with their scenarios,
still `@pending` until M4.

Deviations from the tasks:

- No solution failed with an error, so task 4's repair round had
  nothing to score.
- The fixture record is not committed; its generator is, with a fixed
  seed.
- One model wrote every solution. The stakeholder decided without the
  smaller-model run the report suggests.

## Acceptance Criteria

- [x] A go/no-go on the kernel default is recorded in ADR-04
- [x] OQ-10 is answered
