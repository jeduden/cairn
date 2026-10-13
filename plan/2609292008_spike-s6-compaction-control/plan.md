---
id: 2609292008
title: "Spike S6: can compaction be deferred"
status: "✅"
summary: >-
  Test whether a PreCompact hook can defer native compaction, and
  what happens at a full context window.
model: sonnet
depends-on: [2609292003]
---
# Spike S6: can compaction be deferred

## Goal

Settle whether v1 coexists with native compaction or can steer it, so
NG1 and OQ-01 stop being open.

## Context

Resolves ASM-09 and OQ-01. Builds on the hook fixtures from S1.

## Tasks

1. [x] Exit a PreCompact hook with code 2 and observe whether compaction is
   blocked
2. [x] Drive a session to a full context window with compaction
   deferred and record the outcome
3. [x] Write the recommendation for OQ-01

## Outcome

The [report](report.md) holds the findings. The recordings come from
Claude Code 2.1.296 in an isolated home, made by `record/defer.sh` and
exported by `record/export-defer.sh`, under
[testdata/claude-code](../../testdata/claude-code/2.1.296/manifest.json).

- ASM-09 is verified: a `PreCompact` hook exiting 2 blocked a manual
  compaction and every automatic one, the harness's last attempt at the
  full context window included. On a manual compaction the model reads the
  handler's command and stderr; at the full context window the harness refuses
  every prompt with "Prompt is too long", before any `Stop` hook.
- OQ-01 is closed by the stakeholder on S6's findings: v1 coexists
  with native compaction, option (a), and NG1 stands. Blocking only
  moves native compaction to the full context window and blocks the
  agent there (I9). A bounded veto, compaction the launcher asks the
  harness for, and the proxy (c) are kept for later as UC-006, with the
  prior art the report lists.

Deviations from the tasks:

- Task 2 lowered the full context window to 60k tokens with
  `CLAUDE_CODE_BLOCKING_LIMIT_OVERRIDE`: the models run with a 1M
  window here, too large to record. The report says how the binary sets
  that limit without the override.

Handed on:

- On a harness resume, earlier `Read` results did not reach the model
  again, without a compaction or any hook. The report notes it as seen,
  not recorded; M2's restore assumes content leaves context only
  through a compaction.

Scenarios taken off `@pending`: @ASM-09.

## Acceptance Criteria

- [x] ASM-09 marked verified or re-planned; scenario @ASM-09 passes on
  recorded evidence
- [x] OQ-01 has a documented recommendation
