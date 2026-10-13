---
n: 4
title: "The benchmark gate (ENG-15)"
status: "🔳"
result: true
summary: >-
  The benchmark harness, its unit-proven Mann-Whitney gate and the PR
  job landed; ENG-15 stays pending until NFR-01..05 have benchmarks of
  real code.
---
# Phase 4 result: the benchmark gate (ENG-15)

## Handoff

What landed:

- The `benchmark` tooling crate. Its suite registers each benchmark
  against an NFR id, as a target or a floor, and declares NFR-01..05
  awaiting the hook handlers, the MCP tools, ingestion and the store
  (plan 2609292012). The one benchmark is the core executable's
  start-up, labelled a floor every hook handler pays.
- The gate's decision, unit-proven: the head fails a benchmark when its
  median is more than 10% above the base's and a one-sided
  Mann-Whitney U test (alpha 0.01) finds it slower. 9% slower and
  significant holds, 11% slower but noisy holds, 11% slower and clean
  fails naming the benchmark, faster holds; a benchmark missing on the
  head, or with fewer than ten samples a side, fails (I6).
- CI's `benchmark` job, joined to the `CI` aggregator. It builds the
  base (the head's first parent) and the head in release mode on one
  runner, samples them interleaved, and runs the head's gate.
- Two gate tests in `engineering`'s `benchmark_gate` target, with
  three drift cases (ENG-27), and the ENG-15 steps bound in
  `engineering_benchmark.rs`.

Scenario: `@ENG-15` stays `@pending`. Its Given fails naming
NFR-01..05 and what each awaits; its When and Then pass.

What remains:

- A benchmark of real code for each of NFR-01..05, as M1 builds the
  hook handlers, the MCP tools, ingestion and the store; then drop
  `@pending`.
- The job's first run on a pull request is unobserved: the phase gate
  flips to done once it is green there. Until the suite reaches main,
  the base has none, and the job warns and samples the head alone.
