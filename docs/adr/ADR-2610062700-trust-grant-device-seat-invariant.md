---
id: ADR-2610062700
title: "Security review of the trust-grant device-seat wording"
status: accepted
summary: >-
  The security reviewer's record for rewording I2 after the twelfth
  round of blind domain-model reviews: a trust grant restores only the
  pins the trusted principal wrote from device seats one of its device
  keys certified, never a token-key-only node's. Accepted by the
  stakeholder on 7 October 2026.
---
# ADR-2610062700: Security review of the trust-grant device-seat wording

## Context

The twelfth round of blind domain-model reviews ([merged
note][blind12]) found I2's trust-grant sentence wider than the rest of
the specification. I2 said the pins a trusted principal "wrote from its
device seats" restore to the trusting principal's agents. A
token-key-only node's device seat is also that principal's, but PIN-10,
PRV-10 and the model restore its pins only once stamped, even for its
own principal's agents. Read as written, a trust grant would give
another principal more than the node's own principal gets.

The stakeholder asked for rounds to repeat until none finds anything
to address, with the recommended option on every question (the note's
section 1). CLAUDE.md treats an invariant change as a design change
that needs a security review and a new major version.
[ENG-29](../srs/10-engineering-quality.md#104-process) makes the review
a gate. The reviewer is the stakeholder, @jeduden.

## Decision

The reviewer approves or declines each change below. The status moves to
accepted only when every change reads approved or withdrawn.

| #   | Change                         | Review   |
| --- | ------------------------------ | -------- |
| 1   | I2, trust-granted device seats | approved |

### 1. I2, trust-granted device seats

- **Before:** "the pins that principal wrote from its device seats then
  restore to those agents".
- **After:** "the pins that principal wrote from device seats a device
  key of its certified then restore to those agents".
- **What changes:** a token-key-only node's pins stay unstamped-only for
  every principal. This narrows trust.
- **Review:** approved by the stakeholder, @jeduden (7 October 2026).

## Alternatives

- Let a trust grant cover token-key-only seats too; declined, since it
  would trust what the trusted principal itself does not.

## Consequences

The reviewer approved the change, so the record is accepted (7 October
2026) and the new wording binds. CLAUDE.md, AGENTS.md and README carry
it through their includes of [invariants.md](../srs/invariants.md). No
requirement's traces change.

[blind12]:
  ../../plan/2610012322_cairn-for-agent-fleets/domain-model-review-blind-12.md
