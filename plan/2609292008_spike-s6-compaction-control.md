---
id: 2609292008
title: "Spike S6: can compaction be deferred"
status: "🔲"
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

1. Exit a PreCompact hook with code 2 and observe whether compaction is
   blocked
2. Drive a session to a full window with compaction deferred and record
   the outcome
3. Write the recommendation for OQ-01

## Acceptance Criteria

- [ ] ASM-09 marked verified or re-planned; scenario @ASM-09 passes on
  recorded evidence
- [ ] OQ-01 has a documented recommendation
