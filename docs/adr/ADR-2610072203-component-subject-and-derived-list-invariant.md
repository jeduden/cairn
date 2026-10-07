---
id: ADR-2610072203
title: "Security review of I4's subject and I10's derived list"
status: accepted
summary: >-
  The security reviewer's record for two invariant wordings the blind
  domain-model reviews left to the stakeholder: I4's last sentence binds
  what Cairn's components send, not where data ends up, and I10 names
  room state, trust levels and run and integrity statuses. Approved by
  the stakeholder on 7 October 2026.
---
# ADR-2610072203: Security review of I4's subject and I10's derived list

## Context

Three questions about invariant wording stayed open through the blind
domain-model reviews ([round 23][blind23], [round 27][blind27],
[round 30][blind30]). A [walkthrough note][walk] records how the
stakeholder took them one by one.

- I4's last sentence said "Data leaves the machine only as" three
  channels. §6.3 row 11 lets a browser on the principal's phone reach
  the loopback room view through the principal's own tunnel, LANE-28's
  room trailers leave with the principal's own `git push`, and VIEW-10
  asks the principal to carry a head receipt off the machine (residual
  risk R3). Each read as a break of the literal sentence, which §1.4
  covered only by a reading.
- I10 listed "statuses" among derived artifacts. Room status and the
  time-relative freshness marks are computed where shown and read a
  clock; only run and integrity statuses derive from the record alone.
  Room state and trust levels, also derived, went unnamed.

CLAUDE.md treats an invariant change as a design change that needs a
security review and a new major version.
[ENG-29](../srs/10-engineering-quality.md#104-process) makes the review
a gate. The reviewer is the stakeholder, @jeduden, who chose these
wordings in this session.

## Decision

The reviewer approves or declines each change below. The status moves to
accepted only when every change reads approved or withdrawn.

| #   | Change                          | Review   |
| --- | ------------------------------- | -------- |
| 1   | I4, the last sentence's subject | approved |
| 2   | I10, the derived list           | approved |

### 1. I4, the last sentence's subject

- **Before:** "Data leaves the machine only as recalled content …".
- **After:** "Cairn's components send data off the machine only as
  recalled content …"; the rest of the sentence stands.
- **What changes:** I4 promises what Cairn sends, which Cairn can
  enforce, not what the principal's own tools carry: its tunnel, its
  `git push`, its copies of backups, bundles or a head receipt. §1.4's
  reading now follows from I4. The SRS adds a guard: a Cairn feature
  whose purpose is to reach off the machine through a carrier the
  principal runs has its own §6.3 row, is off by default and is turned
  on by a widening principal act. Room trailers are provenance in the
  repository, stay always on (LANE-28) and get a §6.3 row and a line in
  Setup, so the contract still names what a push carries.
- **Review:** approved by the stakeholder, @jeduden (7 October 2026).

### 2. I10, the derived list

- **Before:** "quarantine set, statuses, queues".
- **After:** "quarantine set, room state, trust levels, run and
  integrity statuses, queues".
- **What changes:** the list matches what the model and §4 already
  derive. Room status, a check's state and the time-relative freshness
  marks stay computed where shown. The `refused` integrity status rests
  on a recorded structural event, not only on an audit entry.
- **Review:** approved by the stakeholder, @jeduden (7 October 2026).

## Alternatives

- I4: keep "Data" and name the tunnel and `git push` as further
  channels; declined, since every other copy the principal makes would
  need its own invariant change.
- I4: drop §6.3 row 11; declined, since people could still tunnel, with
  the full browser secret instead of the phone-scoped one, and the
  trailers and head receipt would still break the sentence.
- I10: copy §4's "run statuses but for their time-relative freshness
  marks"; declined, as a second copy of the model's rule.

## Consequences

The record is accepted (7 October 2026) and the new wording binds.
CLAUDE.md, AGENTS.md and README carry it through their includes of
[invariants.md](../srs/invariants.md). LANE-28 traces I4. The paired
phone under I8 (OQ-39) is not part of this record.

[blind23]:
  ../../plan/2610012322_cairn-for-agent-fleets/domain-model-review-blind-23.md
[blind27]:
  ../../plan/2610012322_cairn-for-agent-fleets/domain-model-review-blind-27.md
[blind30]:
  ../../plan/2610012322_cairn-for-agent-fleets/domain-model-review-blind-30.md
[walk]:
  ../../plan/2610012322_cairn-for-agent-fleets/stakeholder-walkthrough.md
