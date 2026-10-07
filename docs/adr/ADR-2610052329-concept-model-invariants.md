---
id: ADR-2610052329
title: "Security review of the concept-model invariant changes"
status: accepted
summary: >-
  The security reviewer's record for rewording I1, I5 and I10 to the
  domain model of 5 October 2026: a session belongs to the harness, so
  the invariants speak of agent runs, and the removed approval gate
  leaves I10. Accepted by the stakeholder on 6 October 2026.
---
# ADR-2610052329: Security review of the concept-model invariant changes

## Context

On 5 October 2026 the stakeholder fixed Cairn's domain model
([docs/domain-model.md](../domain-model/index.md)): session and project are
not Cairn concepts. A session belongs to the harness, and Cairn speaks
of runs instead: one agent run as the harness reports it, on one node,
holding a seat in its person's personal room and one per room it
joined. SRS 2.3-draft reworks every requirement to that model. Two
invariants still say "session": I1 counts the compactions and sessions
an event survives, and I5 lists a session among what can be
quarantined. No invariant says "project". I10 still lists "gate
verdicts", though the stakeholder removed Cairn's approval gate
(LANE-07 retired; approval belongs to the forge).

CLAUDE.md treats an invariant change as a design change that needs a
security review and a new major version.
[ENG-29](../srs/10-engineering-quality.md#104-process) makes the review
a gate: no invariant change is accepted until this record holds a named
reviewer's approval. The reviewer is the stakeholder, @jeduden. The
stakeholder decided to change these invariants in the same change as
the SRS rework, and approved all three on 6 October 2026.

## Decision

The reviewer approves or declines each change below. The status moves to
accepted only when every change reads approved or withdrawn. Nothing
else in the invariants changes; I4 and I8 keep the word "tenant", which
the glossary maps to person.

| #   | Change                                                  | Review   |
| --- | ------------------------------------------------------- | -------- |
| 1   | I1, Nothing is lost                                     | approved |
| 2   | I5, Bad data can be removed without destroying evidence | approved |
| 3   | I10, Everything derived is rebuildable                  | approved |

### 1. I1, Nothing is lost

- **Before (2.2):** Every event stays recoverable by its address (writer,
  seq) "across any number of compactions and sessions".
- **After (2.3):** The same, "across any number of compactions and agent
  runs".
- **What changes:**
  - Only the word: a harness session and a Cairn run cover the same
    agent's work. The guarantee neither widens nor narrows.
  - A run's history spans the writers of its seats, so the address
    stays (writer, seq), unchanged.
- **Where:** [wording](../srs/invariants.md); requirements
  [REC-02][REC], [REC-04][REC], [LANE-01][LANE].
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 2. I5, Bad data can be removed without destroying evidence

- **Before (2.2):** Any "event, span, session, writer or derived
  artifact" can be quarantined from recall.
- **After (2.3):** Any "event, span, run, writer or derived artifact".
- **What changes:**
  - Quarantine by run takes the place of quarantine by session; it
    covers every writer of the run's seats this node holds.
  - Quarantine also accepts a room, a seat or writer, an actor, an
    address range, a time window and a provenance class (SEC-12); the
    invariant names only the minimum.
- **Where:** [wording](../srs/invariants.md); requirements
  [SEC-12][SEC], [PEER-11][PEER].
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 3. I10, Everything derived is rebuildable

- **Before (2.2):** Derived state includes "evidence, proof and gate
  verdicts".
- **After (2.3):** "evidence and proof classes".
- **What changes:**
  - Cairn records no gate verdict any more; approval belongs to the
    forge (LANE-08), and a person's verdict is a pin, already covered
    by "active pins".
  - The rebuild guarantee neither widens nor narrows for what remains.
- **Where:** [wording](../srs/invariants.md); requirements
  [LANE-06][LANE], [LANE-08][LANE], [OWN-27][OWN].
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

## Alternatives

- Keep "session" in the invariants as the harness's word; declined,
  since the invariants would then name a concept Cairn does not hold,
  and the domain-model guard reports it on every review.
- Reword I4 and I8's "tenant" to "person" in the same record; left out,
  since the stakeholder limited this change to session and project.

## Consequences

The reviewer approved all three changes, so the record is accepted
(6 October 2026) and the 2.3 wording of I1, I5 and I10 binds. The SRS,
CLAUDE.md and AGENTS.md carry it through their includes of
[invariants.md](../srs/invariants.md). No requirement's traces change.

- I4 and I8 still say "tenant"; a later record may reword them.

[REC]: ../srs/05-functional-requirements.md#51-record-rec
[LANE]: ../srs/05b-lane-requirements.md#511-room-lane
[PEER]: ../srs/05c-principal-and-peer-requirements.md#514-peer-network-peer
[OWN]: ../srs/05c-principal-and-peer-requirements.md#513-principal-acts-own
[SEC]: ../srs/06-security.md#62-security-requirements-sec
