---
n: 4
title: "The benchmark gate (ENG-15)"
status: "🔲"
result: false
---
# Phase 4: the benchmark gate (ENG-15)

Requirement. ENG-15 (P0, Ver T): a benchmark suite covers NFR-01..05,
and a CI gate fails on a statistically significant regression greater
than 10%.

The hook handlers, the MCP tools and ingestion that NFR-01..05 measure
arrive with M1's other plans. This phase builds the harness and the
gate: how a benchmark registers against an NFR id, how base and head
are sampled, the significance test, and the CI job that fails naming
the benchmark. A benchmark for code that does not exist yet is not
written as a stand-in.

BDD coverage: `@ENG-15` comes off `@pending` once every NFR-01..05 has
a benchmark of real code. Until then the gate's decision is
unit-proven on synthetic samples, and the job runs in CI.

RED: unit tests of the decision on samples that regress by 9%, 11%
with noise, and 11% clean. GREEN: the harness, the gate, the job.

Gate: the decision's unit tests pass and the job runs on a pull
request.
