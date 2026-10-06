---
id: ADR-2610062500
title: "Security review of the device-seat and peer wording"
status: accepted
summary: >-
  The security reviewer's record for rewording I2, I4 and I8 after
  the tenth round of blind domain-model reviews: trust follows acts a
  device key signs and posts and pins written from a device seat such
  a key certified, restore blocks carry qualifying pins and trusted
  structural fields, and what a peer brings in is trusted only as I2
  allows. Accepted by the stakeholder on 6 October 2026.
---
# ADR-2610062500: Security review of the device-seat and peer wording

## Context

The tenth round of blind domain-model reviews ([merged note][blind10])
found three invariants out of step with the model:

- **I2 and I8** said "signed through a device key". A token key and
  every run seat key also chain to a device key, so the words covered
  content the requirements keep untrusted (PRV-02, 09b, the provenance
  scenarios). The model trusts principal acts a device key signs and
  posts and pins written from a device seat such a key certified.
- **I2** listed "restore blocks of pins …, each built only from trusted
  structural fields and ids", which reads as if restore blocks carried
  no pin text.
- **I4** called whatever a B2 or B3 component brings in untrusted,
  while PRV-02 trusts what the principal signed on all its nodes, so a
  principal's own principal act received from its other node.

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
| 1   | I2 and I8, device-key-signed acts | approved |
| 2   | I2, what restore blocks carry     | approved |
| 3   | I4, what a peer brings in         | approved |

### 1. I2 and I8, device-key-signed acts

- **Before (I2):** "except what the agent's principal signed through a
  device key it certified, …"; "principal acts, posts and pins signed
  through a device key the agent's principal certified, within its
  scope".
- **After (I2):** "except the agent's principal's acts signed by a
  device key it certified, posts and pins written from a device seat
  such a key certified, …"; "principal acts signed by a device key the
  agent's principal certified and posts and pins written from a device
  seat such a key certified, within that key's scope".
- **Before (I8):** "or another node wrote outside what this principal
  signed through a device key it certified".
- **After (I8):** "or another node wrote other than this principal's
  acts signed by a device key it certified and its posts and pins
  written from a device seat such a key certified".
- **What changes:** run seats and token keys no longer read as covered.
  This narrows trust; nothing gains trust it did not have.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 2. I2, what restore blocks carry

- **Before:** "restore blocks of pins (INJ-01, INJ-02), opt-in notices
  (INJ-10), and the fixed templates of OWN-04 and OWN-07, each built
  only from trusted structural fields and ids."
- **After:** "restore blocks (INJ-01, INJ-02), built only from
  qualifying pins and trusted structural fields; and opt-in notices
  (INJ-10) and the fixed templates of OWN-04 and OWN-07, each built
  only from trusted structural fields and ids."
- **What changes:** the sentence states what INJ-03 already enforces.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 3. I4, what a peer brings in

- **Before:** "whatever such a component brings in is untrusted (I2)."
- **After:** "whatever such a component brings in is trusted only as I2
  allows."
- **What changes:** I4 defers to I2, which already decides trust; a
  peer connection itself still trusts nothing.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

## Alternatives

- Keep "untrusted" in I4 and refuse a principal's own signed acts from
  its other nodes; declined, since PRV-10 makes them derive alike on
  every node of that principal.

## Consequences

The reviewer approved all three changes, so the record is accepted
(6 October 2026) and the new wording binds. CLAUDE.md, AGENTS.md and
README carry it through their includes of
[invariants.md](../srs/invariants.md). No requirement's traces change.

[blind10]:
  ../../plan/2610012322_cairn-for-agent-fleets/domain-model-review-blind-10.md
