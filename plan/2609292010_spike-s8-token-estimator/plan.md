---
id: 2609292010
title: "Spike S8: calibrate the token estimator and freeze the targets"
status: "✅"
summary: >-
  Calibrate a conservative token estimator against Claude's counts
  and measure baselines for every dagger-marked target.
model: sonnet
depends-on: [2609292004]
---
# Spike S8: calibrate the token estimator and freeze the targets

## Goal

Freeze every initial target the SRS marks with a dagger, and give the
budgets a token estimator that never undercounts in practice.

## Context

Resolves §8.4 and the dagger-marked targets in §7. Needs the store from
S2 for the performance baselines.

## Tasks

1. [x] Collect a representative sample of record content
2. [x] Fit an estimator that is at or above the true count for at least 99%
   of it
3. [x] Measure baselines for the NFR-01 to NFR-05 targets on the S2 store
4. [x] Update each dagger-marked number in the SRS to its frozen value

## Outcome

The [report](report.md) has the numbers, the
[sample](sample.jsonl) holds 646 texts with Claude's counts, and
[s8bench](bench/src/main.rs) holds the estimator and the hook
timings. In short:

- The estimator weighs each character by one of eleven classes and
  each switch between lowercase, uppercase and digit. It is at or
  above Claude's count for all 646 samples, and for 99.85% under
  five-fold cross validation, overcounting 1.40× overall. Haiku,
  Sonnet and Opus 5.5 and Fable 5.1 share one tokenizer.
- Claude's tokenizer runs at 1.3 to 3.0 bytes per model token, so
  "bytes divided by four" undercounts 97.4% of the sample.
- On emulated reference hardware with a 1M-event store under write
  load, `UserPromptSubmit` takes 3.2 ms and `SessionStart` 4.2 ms at
  p95, each under 11 MiB RSS.
- 9 † targets are frozen at their values: those S2 and S8 measured
  and M1 needs from its start. The 44 others are listed in the SRS's
  new §12.3 with the spike or milestone that freezes each, and §1's
  convention now points there.

Deviations from the tasks:

- Task 4 froze no number at a new value: each frozen target held. On
  the stakeholder's ruling (13 October 2026) a target freezes as late
  as it can, so the spike froze only what it measured and M1 needs from
  its start. Every other target was re-planned, as the acceptance
  criteria allow, to the spike or milestone that first uses or
  measures it.
- A 1,000-character pin (PIN-03) in Korean, Chinese or emoji estimates
  above PIN-08's whole pin budget. Rather than rewrite PIN-03 here,
  OQ-56 asks M2 to bound pins in estimated model tokens too.
- Fable 5.1 reported thousands of extra input tokens on two random
  samples where the other models agree. The fit uses the shared
  tokenizer's count; the report explains why those are not text
  tokens.

Handed on:

- M1 ports the estimator into the core with its tests and moves the
  sample into `testdata/` (its task 9). Each milestone plan has a task
  to freeze the targets §12.3 gives it.
- M2 answers OQ-56.
- M1 sets NFR-09's store overhead on its schema.
- M3 decides LMK-03's field length.

No requirement closes and no scenario leaves `@pending`. The spike
gathers evidence for §8.4, NFR-01, NFR-03, NFR-04 and NFR-09.

## Acceptance Criteria

- [x] The estimator meets the §8.4 bound on the committed sample
- [x] Every dagger-marked target in the SRS is frozen or explicitly
  re-planned
