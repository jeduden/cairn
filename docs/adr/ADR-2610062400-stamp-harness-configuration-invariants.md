---
id: ADR-2610062400
title: "Security review of the stamp and configuration wording"
status: accepted
summary: >-
  The security reviewer's record for rewording I2, I7 and I10 after
  the ninth round of blind domain-model reviews: I2's untrusted list
  excepts a pin version the agent's principal stamped, I7's title
  names harness configuration, and I10 says stats. Accepted by the
  stakeholder on 6 October 2026.
---
# ADR-2610062400: Security review of the stamp and configuration wording

## Context

The ninth round of blind domain-model reviews ([merged note][blind9])
found three invariants out of step with the model:

- **I2's untrusted list** excepted what the agent's principal signed
  and what its trust grant covers, but not a pin version it stamped,
  which I2's own trusted sources, I8, PRV-02 and PIN-10 include.
- **I7's title** said "Configuration changes", while its body and the
  model speak of harness configuration; the model's unqualified
  configuration is the principal's settings.
- **I10** said "statistics", which the model calls stats.

The stakeholder asked for rounds to repeat until none finds anything
to address, with the recommended option on every question (the note's
section 1). CLAUDE.md treats an invariant change as a design change
that needs a security review and a new major version.
[ENG-29](../srs/10-engineering-quality.md#104-process) makes the review
a gate. The reviewer is the stakeholder, @jeduden.

## Decision

The reviewer approves or declines each change below. The status moves to
accepted only when every change reads approved or withdrawn.

| #   | Change                    | Review   |
| --- | ------------------------- | -------- |
| 1   | I2, the stamp exception   | approved |
| 2   | I7, harness configuration | approved |
| 3   | I10, stats                | approved |

### 1. I2, the stamp exception

- **Before:** "except what the agent's principal signed through a
  device key it certified or a trust grant of that principal covers)".
- **After:** "except what the agent's principal signed through a device
  key it certified, a pin version it stamped, or what a trust grant of
  that principal covers)".
- **What changes:** the first sentence now agrees with I2's own trusted
  sources. No content gains trust it did not have.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 2. I7, harness configuration

- **Before:** "Configuration changes only on explicit instruction."
- **After:** "Harness configuration changes only on explicit
  instruction."
- **What changes:** only the title, to match the body. Changes to the
  principal's configuration are principal acts under OWN-11 as before.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 3. I10, stats

- **Before:** "evidence and proof classes, statistics)".
- **After:** "evidence and proof classes, stats)".
- **What changes:** only the term, which the model defines.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

## Alternatives

- Leave I2's list as it was, reading a stamp as a signature through a
  device key; declined, since a stamp is a principal act on someone
  else's pin version, not the agent's principal's own content.

## Consequences

The reviewer approved all three changes, so the record is accepted
(6 October 2026) and the new wording binds. CLAUDE.md, AGENTS.md and
README carry it through their includes of
[invariants.md](../srs/invariants.md). No requirement's traces change.

[blind9]:
  ../../plan/2610012322_cairn-for-agent-fleets/domain-model-review-blind-9.md
