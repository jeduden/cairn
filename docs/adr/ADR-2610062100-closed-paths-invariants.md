---
id: ADR-2610062100
title: "Security review of the closed-paths invariant wording"
status: accepted
summary: >-
  The security reviewer's record for rewording I2, I4, I8 and I10
  after the sixth round of blind domain-model reviews: I2 names
  trust-granted posts and pins among the trusted sources and lets
  outside content reach the model only enveloped by recall or through
  a closed path a principal act names, I4 says what leaves the
  machine, I8 drops "held" and "widens", and I10 says derived
  artifacts. Accepted by the stakeholder on 6 October 2026.
---
# ADR-2610062100: Security review of the closed-paths invariant wording

## Context

The sixth round of blind domain-model reviews ([merged
note][blind6]) found four invariants out of step with the model and
the requirements:

- **I2** listed the trusted sources without the posts and pins a trust
  grant covers, though PRV-02, VIEW-07 and the model list them. Its
  first sentence let outside content reach the model only by recall,
  while its own later sentences add endorsements, delegated tasks and
  trust-granted posts. It also said trust-granted posts reach agents
  "as the trusting principal's own text would", while a principal's
  own posts reach its agents only by recall or endorsement (OWN-08).
- **I4** said data leaves the machine only as recalled content, while
  restore blocks and opt-in notices also reach the model through the
  harness (§6.3).
- **I8** said content "is held as theirs", while the model says
  "holds" only of nodes, and "never widens what … agents trust or
  recall", while "widening" belongs to acts.
- **I10** said "derived state", which the model calls derived
  artifacts.

The stakeholder asked for rounds to repeat until none finds anything
to address, with the recommended option on every question (the note's
section 1). CLAUDE.md treats an invariant change as a design change
that needs a security review and a new major version.
[ENG-29](../srs/10-engineering-quality.md#104-process) makes the review
a gate. The reviewer is the stakeholder, @jeduden.

## Decision

The reviewer approves or declines each change below. The status moves to
accepted only when every change reads approved or withdrawn.

| #   | Change                            | Review   |
| --- | --------------------------------- | -------- |
| 1   | I2, recall or a named closed path | approved |
| 2   | I2, trust-granted posts and pins  | approved |
| 3   | I4, what leaves the machine       | approved |
| 4   | I8, "kept" and "extends"          | approved |
| 5   | I10, derived artifacts            | approved |

### 1. I2, recall or a named closed path

- **Before:** "reaches the model only when Claude explicitly calls a
  recall tool, and always inside an untrusted-data envelope."
- **After:** "reaches the model only inside an untrusted-data envelope
  when Claude explicitly calls a recall tool, or through one of the
  closed paths below that a principal act names."
- **What changes:** the sentence now agrees with the closed list that
  follows it. Every non-recall path already needs a principal act: an
  endorsement, a delegation grant with its acceptance grant, or a trust
  grant. No path is added.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 2. I2, trust-granted posts and pins

- **Before:** the trusted sources ended at stamped pin versions; "the
  posts and pins that principal wrote … then reach those agents as the
  trusting principal's own text would."
- **After:** the trusted sources also list "the posts and pins a trust
  grant of the agent's principal covers"; "the pins that principal
  wrote from its device seats then restore to those agents, and its
  posts reach them inside the fixed template of OWN-29."
- **What changes:** the invariant states what OWN-29 already does, and
  no longer suggests that a trust grant delivers posts the way a
  principal's own text would.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 3. I4, what leaves the machine

- **Before:** "Data leaves the machine only when Claude receives
  recalled content through a tool call, or through a B2 or B3
  component".
- **After:** "Data leaves the machine only as what Cairn writes to an
  agent through I2's closed paths, which the harness sends to its
  model, or through a B2 or B3 component".
- **What changes:** restore blocks, opt-in notices and the fixed
  templates leave with the harness's own traffic too; the invariant
  now says so. Cairn still opens no connection from the core.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 4. I8, "kept" and "extends"

- **Before:** "is held as theirs"; "It never widens what this
  principal's agents trust or recall on its own."
- **After:** "is kept as theirs"; "On its own it never makes this
  principal's agents trust it, nor extends their recall."
- **What changes:** only the words, to the model's verb rules.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 5. I10, derived artifacts

- **Before:** "All derived state (…) is a deterministic function".
- **After:** "All derived artifacts (…) are a deterministic function".
- **What changes:** only the term, which the model defines.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

## Alternatives

- Push a principal's own device-seat posts to its agents like
  trust-granted ones; declined in round 5, since it would add a path.
- Keep I4's wording and treat restore blocks as part of recall;
  declined, since they reach the model without a tool call.

## Consequences

The reviewer approved all five changes, so the record is accepted
(6 October 2026) and the new wording binds. CLAUDE.md, AGENTS.md and
README carry it through their includes of
[invariants.md](../srs/invariants.md). No requirement's traces change.

[blind6]:
  ../../plan/2610012322_cairn-for-agent-fleets/domain-model-review-blind-6.md
