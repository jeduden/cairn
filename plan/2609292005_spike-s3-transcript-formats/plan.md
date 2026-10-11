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

Resolves ASM-03, ASM-04, ASM-05 and ASM-11 to ASM-16, and informs
OQ-12. ASM-11 to ASM-16 come from a review of sottochat's transcript
parser (SRS §14, reference 17).

## Tasks

1. Collect transcripts from a main session and from subagents
2. Reproduce a rewind or resume that truncates or rewrites a transcript
3. Confirm deletion after `cleanupPeriodDays`
4. Record attachment records, multi-line assistant replies, tool
   results with `toolUseResult`, lines without `uuid` or `timestamp`,
   and thinking blocks
5. Record how three working directories that differ only in `_`, `.`
   and `-` map to project directories
6. Commit a redacted fixture for every observed line type and variant

## Acceptance Criteria

- [ ] ASM-03, ASM-04, ASM-05 and ASM-11 to ASM-16 each marked
  verified or re-planned in the SRS
- [ ] Scenarios @ASM-03, @ASM-04, @ASM-05 and @ASM-11 to @ASM-16
  replay the fixtures and pass
- [ ] OQ-12 has a recommendation
- [ ] The fixtures are the seed corpus for the transcript-parser fuzz
  target (ENG-07)
