---
id: 2609292010
title: "Spike S8: calibrate the token estimator and freeze the targets"
status: "🔲"
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

1. Collect a representative sample of record content
2. Fit an estimator that is at or above the true count for at least 99%
   of it
3. Measure baselines for the NFR-01 to NFR-05 targets on the S2 store
4. Update each dagger-marked number in the SRS to its frozen value

## Acceptance Criteria

- [ ] The estimator meets the §8.4 bound on the committed sample
- [ ] Every dagger-marked target in the SRS is frozen or explicitly
  re-planned
