---
id: ADR-2610063300
title: "Security review of the trusted user events wording"
status: accepted
summary: >-
  The security reviewer's record for I2's trusted sources after the
  twentieth round of blind domain-model reviews: the user events the hook
  handlers recorded, not the user turns, since a turn runs to the model
  reply that ends it. Approved by the stakeholder on 7 October 2026.
---
# ADR-2610063300: Security review of the trusted user events wording

## Context

The twentieth round of blind domain-model reviews ([merged
note][blind20]) found a gap in I2. It trusted "the `user` turns" the
hook handlers recorded while the deployment mode is `interactive`. The
model defines a turn as running from an input to the model reply that
ends it. Read literally, the invariant also trusted the model's
replies, which its own list of untrusted content names.

CLAUDE.md treats an invariant change as a design change that needs a
security review and a new major version.
[ENG-29](../srs/10-engineering-quality.md#104-process) makes the review
a gate. The reviewer is the stakeholder, @jeduden, who answered "I2 ->
yes" to the proposed wording in this session.

## Decision

The reviewer approves or declines each change below. The status moves to
accepted only when every change reads approved or withdrawn.

| #   | Change                    | Review   |
| --- | ------------------------- | -------- |
| 1   | I2, trusted `user` events | approved |

### 1. I2, trusted `user` events

- **Before:** "the `user` turns they recorded while the deployment mode
  is `interactive`".
- **After:** "the `user` events they recorded while the deployment mode
  is `interactive`".
- **What changes:** trust narrows to the person's own input event. The
  model reply in the same turn stays untrusted, as I2's list of
  untrusted content already says.
- **Review:** approved by the stakeholder, @jeduden (7 October 2026).

## Alternatives

- Keep "turns" and define a trusted user turn in the model as only its
  `user` event; declined, since the invariant would still read wider
  than the rule it states.

## Consequences

The record is accepted (7 October 2026) and the new wording binds.
CLAUDE.md, AGENTS.md and README carry it through their includes of
[invariants.md](../srs/invariants.md). The model's Turn and Trusted
sources entries follow. No requirement's traces change.

[blind20]:
  ../../plan/2610012322_cairn-for-agent-fleets/domain-model-review-blind-20.md
