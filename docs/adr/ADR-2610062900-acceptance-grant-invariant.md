---
id: ADR-2610062900
title: "Security review of the acceptance-grant wording"
status: accepted
summary: >-
  The security reviewer's record for rewording I2 after the fourteenth
  round of blind domain-model reviews: a delegated task reaches an agent
  of another principal only under an acceptance grant that agent's
  principal recorded, and the trust-grant sentence reads "one of its
  device keys". Accepted by the stakeholder on 7 October 2026.
---
# ADR-2610062900: Security review of the acceptance-grant wording

## Context

The fourteenth round of blind domain-model reviews ([merged
note][blind14]) found I2's delegation path narrower than the
requirements. I2 let a delegated task reach an agent only "under a
delegation grant its principal recorded (OWN-23)". OWN-26, OWN-01 and
the model also let a principal's agent delegate to another principal's
agent, which takes the task only under an acceptance grant its own
principal recorded. Read as written, I2 either forbids that path or
lets the delegating principal widen what reaches the other principal's
agent. Two reviews also found the trust-grant sentence ungrammatical:
"device seats a device key of its certified".

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
| 1   | I2, the acceptance grant     | approved |
| 2   | I2, "one of its device keys" | approved |

### 1. I2, the acceptance grant

- **Before:** "Under a delegation grant its principal recorded (OWN-23):
  a delegated task inside the fixed template of OWN-24."
- **After:** "Under a delegation grant its principal recorded (OWN-23),
  and, for an agent of another principal, an acceptance grant that
  agent's principal recorded (OWN-26): a delegated task inside the fixed
  template of OWN-24."
- **What changes:** the receiving agent's own principal must have
  recorded the acceptance grant, so only that principal widens what
  reaches its agent, as the model's Principal entry requires.
- **Review:** approved by the stakeholder, @jeduden (7 October 2026).

### 2. I2, "one of its device keys"

- **Before:** "the pins that principal wrote from device seats a device
  key of its certified".
- **After:** "the pins that principal wrote from device seats one of its
  device keys certified".
- **What changes:** only the grammar; the meaning stays as
  ADR-2610062700 set it.
- **Review:** approved by the stakeholder, @jeduden (7 October 2026).

## Alternatives

- Mark OWN-26 as outside I2 until its own review; declined, since the
  path already needs the receiving principal's grant, which I2 can name.

## Consequences

The reviewer approved both changes, so the record is accepted (7 October
2026) and the new wording binds. CLAUDE.md, AGENTS.md and README carry
it through their includes of [invariants.md](../srs/invariants.md). No
requirement's traces change.

[blind14]:
  ../../plan/2610012322_cairn-for-agent-fleets/domain-model-review-blind-14.md
