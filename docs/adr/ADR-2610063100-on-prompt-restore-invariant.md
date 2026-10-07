---
id: ADR-2610063100
title: "Security review of the on-prompt restore wording"
status: accepted
summary: >-
  The security reviewer's record for citing INJ-04 in I2 after the
  sixteenth round of blind domain-model reviews: a restore block injected
  on a prompt, while restore_block.on_prompt is on, is the same closed
  path as the restore blocks of INJ-01 and INJ-02. Accepted by the
  stakeholder on 7 October 2026.
---
# ADR-2610063100: Security review of the on-prompt restore wording

## Context

The sixteenth round of blind domain-model reviews ([merged
note][blind16]) found a gap in I2. INJ-04 lets Cairn inject a restore
block on a prompt when `restore_block.on_prompt` is on. I2's closed list
of writes cited only INJ-01 and INJ-02 for restore blocks. INJ-04 builds
the same `TrustedText` under the same rules (INJ-03), so the path is not
new, but the invariant did not name it.

The stakeholder asked for rounds to repeat until none finds anything
to address, with the recommended option on every question (the note's
section 1). CLAUDE.md treats an invariant change as a design change
that needs a security review and a new major version.
[ENG-29](../srs/10-engineering-quality.md#104-process) makes the review
a gate. The reviewer is the stakeholder, @jeduden.

## Decision

The reviewer approves or declines each change below. The status moves to
accepted only when every change reads approved or withdrawn.

| #   | Change                       | Review   |
| --- | ---------------------------- | -------- |
| 1   | I2, on-prompt restore blocks | approved |

### 1. I2, on-prompt restore blocks

- **Before:** "restore blocks (INJ-01, INJ-02)".
- **After:** "restore blocks (INJ-01, INJ-02, INJ-04)".
- **What changes:** the closed list cites every requirement that injects
  a restore block. The content limit is unchanged.
- **Review:** approved by the stakeholder, @jeduden (7 October 2026).

## Alternatives

- Treat on-prompt injection as outside I2 and drop INJ-04; declined,
  since no requirement row may be deleted and the block is the same.

## Consequences

The reviewer approved the change, so the record is accepted (7 October
2026) and the new wording binds. CLAUDE.md, AGENTS.md and README carry
it through their includes of [invariants.md](../srs/invariants.md). No
requirement's traces change.

[blind16]:
  ../../plan/2610012322_cairn-for-agent-fleets/domain-model-review-blind-16.md
