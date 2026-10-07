---
id: ADR-2610063000
title: "Security review of the compaction-guidance wording"
status: accepted
summary: >-
  The security reviewer's record for adding compaction guidance to I2's
  closed paths after the fifteenth round of blind domain-model reviews:
  PIN-07's fixed text, returned to the harness at PreCompact, is a write
  Cairn makes without a principal act, so I2 names it among the paths
  built only from fixed text Cairn ships. Accepted by the stakeholder on
  7 October 2026.
---
# ADR-2610063000: Security review of the compaction-guidance wording

## Context

The fifteenth round of blind domain-model reviews ([merged
note][blind15]) found that PIN-07 lets the `PreCompact` hook handler
return fixed compaction guidance to the harness, which reaches the
model through the harness's compaction. I2 lists every path by which
Cairn writes to an agent and ends "No other write to an agent exists";
compaction guidance was not on the list. PIN-07 already requires the
guidance to be static text with no record content.

The stakeholder asked for rounds to repeat until none finds anything
to address, with the recommended option on every question (the note's
section 1). CLAUDE.md treats an invariant change as a design change
that needs a security review and a new major version.
[ENG-29](../srs/10-engineering-quality.md#104-process) makes the review
a gate. The reviewer is the stakeholder, @jeduden.

## Decision

The reviewer approves or declines each change below. The status moves to
accepted only when every change reads approved or withdrawn.

| #   | Change                  | Review   |
| --- | ----------------------- | -------- |
| 1   | I2, compaction guidance | approved |

### 1. I2, compaction guidance

- **Before:** "and opt-in notices (INJ-10) and the fixed templates of
  OWN-04 and OWN-07, each built only from fixed text Cairn ships,
  trusted structural fields and ids."
- **After:** "and opt-in notices (INJ-10), compaction guidance (PIN-07)
  and the fixed templates of OWN-04 and OWN-07, each built only from
  fixed text Cairn ships, trusted structural fields and ids."
- **What changes:** the closed list names a write PIN-07 already makes,
  under the same limit as notices and templates. No record content
  gains a path.
- **Review:** approved by the stakeholder, @jeduden (7 October 2026).

## Alternatives

- Drop PIN-07; declined, since no requirement row may be deleted, and
  static guidance carries no content from outside Cairn.

## Consequences

The reviewer approved the change, so the record is accepted (7 October
2026) and the new wording binds. CLAUDE.md, AGENTS.md and README carry
it through their includes of [invariants.md](../srs/invariants.md). No
requirement's traces change.

[blind15]:
  ../../plan/2610012322_cairn-for-agent-fleets/domain-model-review-blind-15.md
