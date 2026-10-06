---
id: ADR-2610062300
title: "Security review of the quarantine invariant wording"
status: accepted
summary: >-
  The security reviewer's record for rewording I5's title after the
  eighth round of blind domain-model reviews: bad data leaves
  circulation without destroying the record, since "evidence" names
  what a result rests on. Accepted by the stakeholder on 6 October
  2026.
---
# ADR-2610062300: Security review of the quarantine invariant wording

## Context

The eighth round of blind domain-model reviews ([merged
note][blind8]) found I5's title using "evidence" in a forensic sense.
The model gives the word one meaning: the checks, attestations or text
a result rests on. I5's body already says the record stays intact for
forensics.

The stakeholder asked for rounds to repeat until none finds anything
to address, with the recommended option on every question (the note's
section 1). CLAUDE.md treats an invariant change as a design change
that needs a security review and a new major version.
[ENG-29](../srs/10-engineering-quality.md#104-process) makes the review
a gate. The reviewer is the stakeholder, @jeduden.

## Decision

The reviewer approves or declines each change below. The status moves to
accepted only when every change reads approved or withdrawn.

| #   | Change                     | Review   |
| --- | -------------------------- | -------- |
| 1   | I5, the record stays whole | approved |

### 1. I5, the record stays whole

- **Before:** "Bad data can be removed from circulation without
  destroying evidence."
- **After:** "Bad data can be removed from circulation without
  destroying the record."
- **What changes:** only the words, to the model's terms. Quarantine
  still removes nothing from the record.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

## Alternatives

- Give "evidence" a second, forensic meaning in the model; declined,
  since every Cairn word has one meaning.

## Consequences

The reviewer approved the change, so the record is accepted (6 October
2026) and the new wording binds. CLAUDE.md, AGENTS.md and README carry
it through their includes of [invariants.md](../srs/invariants.md). No
requirement's traces change.

[blind8]:
  ../../plan/2610012322_cairn-for-agent-fleets/domain-model-review-blind-8.md
