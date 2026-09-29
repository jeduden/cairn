---
id: 2609292005
title: "Spike S3: catalogue the transcript formats"
status: "🔲"
summary: >-
  Collect main, subagent, rewritten and cleaned-up transcripts and
  turn every observed variant into parser fixtures.
model: sonnet
depends-on: []
---
# Spike S3: catalogue the transcript formats

## Goal

Know every transcript shape the parser must read before REC-01 to REC-07
are built on it.

## Context

Resolves ASM-03, ASM-04 and ASM-05.

## Tasks

1. Collect transcripts from a main session and from subagents
2. Reproduce a rewind or resume that truncates or rewrites a transcript
3. Confirm deletion after `cleanupPeriodDays`
4. Commit a redacted fixture for every observed line type and variant

## Acceptance Criteria

- [ ] ASM-03, ASM-04 and ASM-05 each marked verified or re-planned in
  the SRS
- [ ] Scenarios @ASM-03, @ASM-04 and @ASM-05 replay the fixtures and
  pass
- [ ] The fixtures are the seed corpus for the transcript-parser fuzz
  target (ENG-07)
