---
id: 2609292003
title: "Spike S1: record the hook contract"
status: "🔲"
summary: >-
  Record real hook payloads, output handling and budgets for every
  hook Cairn uses, including subagent compaction.
model: sonnet
depends-on: []
---
# Spike S1: record the hook contract

## Goal

Pin down what Claude Code actually sends to and accepts from each hook,
so §9.1 is built on recorded fixtures rather than documentation.

## Context

Resolves ASM-01, ASM-02, ASM-06, ASM-10 and ASM-11, and informs OQ-02. The
recorded payloads seed the ENG-17 contract tests.

## Tasks

1. Record stdin payloads for SessionStart (all four sources),
   PreCompact, PostCompact, PreToolUse, PostToolUse (a success and a
   failure), Stop, UserPromptSubmit and SessionEnd
2. Record a subagent compaction and note which identity its hooks carry
3. Test whether `additionalContext` is honored per hook, and whether
   PreToolUse and PostToolUse context reaches Claude before its next
   step (the cue moments of SRS §5.11)
4. Measure the effective timeout of each hook, SessionEnd included
5. Commit the fixtures under testdata and write the contract into
   [docs/srs/09-interfaces.md](../docs/srs/09-interfaces.md) if it
   differs

## Acceptance Criteria

- [ ] ASM-01, ASM-02, ASM-06, ASM-10 and ASM-11 each marked verified
  or re-planned in the SRS
- [ ] Scenarios @ASM-01, @ASM-02, @ASM-06, @ASM-10 and @ASM-11 replay
  the recorded fixtures and pass
- [ ] OQ-02 has an answer or a stated follow-up
