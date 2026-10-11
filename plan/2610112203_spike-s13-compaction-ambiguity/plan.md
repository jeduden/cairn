---
id: 2610112203
title: "Spike S13: can a hook tell which run compacted"
status: "✅"
summary: >-
  Record background-subagent and main-agent compactions with a
  snapshot of every transcript at each hook, to ground INJ-05's
  ambiguity rule in what Claude Code shows at hook time.
model: sonnet
depends-on: [2609292003, 2609292005]
---
# Spike S13: can a hook tell which run compacted

## Goal

Ground INJ-05's ambiguity rule in what Claude Code shows a hook
handler, not in a guess.

## Context

The stakeholder's ruling R2 has INJ-05 restore only the pins every
candidate run qualifies for when a compaction is ambiguous. The
reviews of writing it in asked whether a background subagent or ingest
lag can hide a subagent's compaction. On 11 October 2026 the
stakeholder asked for a spike instead of a guess. Builds on S1's
subagent compaction (ASM-06).

## Tasks

1. [x] Record a background subagent compacting, with a snapshot of
   every transcript of its harness session at each hook
2. [x] Record the main agent compacting while a background subagent
   runs, with the same snapshots
3. [x] Find where each `SessionStart` answer lands
4. [x] Check whether a transcript marks a turn's end without a `Stop`
   hook, and how it marks a prompt's origin
5. [x] Commit the recordings as fixtures and replay them in @ASM-06

## Outcome

The [report](report.md) has the findings. In short: no hook input and
no transcript names the compacting run while its hooks run, so INJ-05
must treat a compaction as ambiguous whenever a subagent of the
harness session may be running, background ones included.

## Acceptance Criteria

- [x] Both flows are recorded, redacted and committed under
  `testdata/claude-code/2.1.296`
- [x] @ASM-06 replays the foreground, background and main-agent cases
- [x] INJ-05's rule follows the findings
