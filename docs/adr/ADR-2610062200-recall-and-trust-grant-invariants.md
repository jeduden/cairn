---
id: ADR-2610062200
title: "Security review of the recall and trust-grant invariant wording"
status: accepted
summary: >-
  The security reviewer's record for rewording I2 and I4 after the
  seventh round of blind domain-model reviews: I2's untrusted list
  excepts what a trust grant covers and speaks of the model's replies
  and the agent's recall, and I4 names recalled content among what
  leaves the machine. Accepted by the stakeholder on 6 October 2026.
---
# ADR-2610062200: Security review of the recall and trust-grant invariant wording

## Context

The seventh round of blind domain-model reviews ([merged
note][blind7]) found I2 and I4 out of step with the model:

- **I2's untrusted list** counted everything another principal produced
  as untrusted, excepting only what the agent's principal signed, while
  its own trusted sources (since ADR-2610062100) include the posts and
  pins a trust grant covers. It also said "assistant text", which the
  model calls the model's replies, and "Claude explicitly calls a
  recall tool", while the model makes recall the agent's tool call and
  Claude the model behind an agent.
- **I4** said data leaves the machine only as what Cairn writes through
  I2's closed paths, which I2 keeps apart from recall, so recalled
  content was left out (§2.2 names both).

The stakeholder asked for rounds to repeat until none finds anything
to address, with the recommended option on every question (the note's
section 1). CLAUDE.md treats an invariant change as a design change
that needs a security review and a new major version.
[ENG-29](../srs/10-engineering-quality.md#104-process) makes the review
a gate. The reviewer is the stakeholder, @jeduden.

## Decision

The reviewer approves or declines each change below. The status moves to
accepted only when every change reads approved or withdrawn.

| #   | Change                                  | Review   |
| --- | --------------------------------------- | -------- |
| 1   | I2, the trust-grant exception           | approved |
| 2   | I2, the model's replies and the agent   | approved |
| 3   | I4, recalled content leaves the machine | approved |

### 1. I2, the trust-grant exception

- **Before:** "except what the agent's principal signed through a
  device key it certified)".
- **After:** "except what the agent's principal signed through a device
  key it certified or a trust grant of that principal covers)".
- **What changes:** the first sentence now agrees with I2's own list of
  trusted sources. No content gains trust it did not have.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 2. I2, the model's replies and the agent

- **Before:** "assistant text"; "when Claude explicitly calls a recall
  tool".
- **After:** "the model's replies"; "when the agent explicitly calls a
  recall tool".
- **What changes:** only the words, to the model's terms.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 3. I4, recalled content leaves the machine

- **Before:** "Data leaves the machine only as what Cairn writes to an
  agent through I2's closed paths, which the harness sends to its
  model".
- **After:** "Data leaves the machine only as recalled content an agent
  receives through a tool call, or as what Cairn writes to an agent
  through I2's closed paths, both of which the harness sends to its
  model".
- **What changes:** the invariant names both ways data reaches the model
  provider, as §2.2 and §6.3 already do. No new path opens.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

## Alternatives

- Count recall among I2's closed paths instead; declined, since recall
  is pull-only and enveloped, and I2 keeps it apart on purpose.

## Consequences

The reviewer approved all three changes, so the record is accepted
(6 October 2026) and the new wording binds. CLAUDE.md, AGENTS.md and
README carry it through their includes of
[invariants.md](../srs/invariants.md). No requirement's traces change.

[blind7]:
  ../../plan/2610012322_cairn-for-agent-fleets/domain-model-review-blind-7.md
