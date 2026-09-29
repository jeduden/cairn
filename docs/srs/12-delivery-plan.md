---
title: "12. Delivery plan"
summary: >-
  Milestone M0 spikes S1–S8 and milestones M1–M6 with their scope and
  exit criteria.
---
# 12. Delivery plan

## 12.1 Milestone M0 — spikes

| Spike | Question                                                                                                               | Resolves             | Exit criterion                                     |
| ----- | ---------------------------------------------------------------------------------------------------------------------- | -------------------- | -------------------------------------------------- |
| S1    | Exact hook payloads, output handling, and budgets, including subagent compaction                                       | ASM-01, 02, 06, 10   | Recorded fixtures for every hook; written contract |
| S2    | Which pure-Go SQLite driver meets FTS5, cancellation, WAL concurrency, encryption, and performance needs at 10M events | ADR-07, OQ-03, OQ-04 | Benchmark report and driver decision               |
| S3    | Transcript formats: main, subagent, rewrites, deletion after cleanup                                                   | ASM-03, 04, 05       | Parser fixtures covering all observed variants     |
| S4    | Plugin distribution on workstations, runner images, and the Agent SDK                                                  | ASM-07, 08           | Working install on each target                     |
| S5    | Claude's success rate writing Starlark versus Python on 30 aggregation tasks                                           | ADR-04, OQ-10        | Go/no-go on Starlark default                       |
| S6    | Can compaction be deferred via `PreCompact`, and what happens at a full window                                         | ASM-09, OQ-01        | Documented behaviour and recommendation            |
| S7    | Fit of the official MCP Go SDK (stdio, structured content, cancellation)                                               | §9.2                 | Decision: SDK or minimal in-house server           |
| S8    | Token-estimator calibration and baseline measurements for all † targets                                                | §8.4, §7             | Calibrated estimator; frozen targets               |

## 12.2 Milestones

| Milestone                         | Scope                                                                                                                                           | Exit criteria                                                                                       |
| --------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| **M1 — Record and recall**        | REC P0, PRV P0, RCL P0, SEC-01..08, SEC-12, SEC-16, SEC-18, OPS-01..03, ADM-04, 05, 09, 11, ENG P0 gates                                        | Recall ≥ 95% at 1–3 compactions; `verify` proves completeness; quarantine effective; CI gates green |
| **M2 — Pins and restore**         | PIN P0, INJ P0, ADM-01..03, ADM-10, SEC-11                                                                                                      | Constraint survival target met; golden tests stable; install/uninstall round-trip exact             |
| **M3 — Landmarks and lifecycle**  | LMK P0, ADM-06..08, SEC-14                                                                                                                      | Recall target met at 10 compactions; index bound verified; rebuild byte-identical                   |
| **M4 — Kernel**                   | CMP P1                                                                                                                                          | Aggregation over 10M-token histories without prompt growth; limits enforced                         |
| **M5 — Hardening and evaluation** | Remaining P1 (PRV-07, PIN-05, PIN-07, PIN-09, REC-13..15, SEC-09, SEC-13, OPS-04..05, ADM-12, ENG-13, ENG-25), full §11, external review, pilot | Definition of done (§11.2)                                                                          |
| **M6 — Extensions**               | P2 items, subject to separate proposals                                                                                                         | —                                                                                                   |
