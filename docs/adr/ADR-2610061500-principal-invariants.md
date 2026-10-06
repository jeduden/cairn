---
id: ADR-2610061500
title: "Security review of the principal-model invariant changes"
status: accepted
summary: >-
  The security reviewer's record for rewording I1, I2, I4, I5, I6 and
  I8 to the domain model the stakeholder adopted on 6 October 2026:
  principal for owner, operator and tenant, trusted sources for the
  trusted boundary, and the TUI named in the core. Accepted by the
  stakeholder on 6 October 2026.
---
# ADR-2610061500: Security review of the principal-model invariant changes

## Context

On 6 October 2026 the stakeholder adopted the second version of
Cairn's domain model ([docs/domain-model.md](../domain-model.md)). It
came with the recommendations of its [proposal][v2] and the decisions
of the three [blind reviews][blind] that followed. The model changes
words the invariants use:

- **Owner** now means only a room's owner. A principal, a person or a
  service account, signs principal acts with a device key its
  principal key certified.
- **Operator** survives only as the `operator` provenance class; the
  room role is moderator, and the person running a node is the node's
  principal.
- **Tenant** is replaced by principal, and a writer is named by its
  seat key.
- **Trusted boundary** becomes trusted sources, so that "boundary"
  means only I4's network boundaries.
- **Core:** the model's core holds the TUI, which I4's list left out.

CLAUDE.md treats an invariant change as a design change that needs a
security review and a new major version.
[ENG-29](../srs/10-engineering-quality.md#104-process) makes the review
a gate: no invariant change is accepted until this record holds a named
reviewer's approval. The reviewer is the stakeholder, @jeduden, who
adopted the model with its invariant wording on 6 October 2026.

## Decision

The reviewer approves or declines each change below. The status moves to
accepted only when every change reads approved or withdrawn.

| #   | Change                                                  | Review   |
| --- | ------------------------------------------------------- | -------- |
| 1   | I1, Nothing is lost                                     | approved |
| 2   | I2, No automatic path from untrusted content            | approved |
| 3   | I4, One network boundary per component                  | approved |
| 4   | I5, Bad data can be removed without destroying evidence | approved |
| 5   | I6, No silent failures                                  | approved |
| 6   | I8, Isolation follows the principal                     | approved |

### 1. I1, Nothing is lost

- **Before:** "data an operator explicitly purges or expires by
  policy".
- **After:** "data the node's principal explicitly purges or expires
  by policy".
- **What changes:** only who is named. The person running a node was
  always its operator; the exception neither widens nor narrows.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 2. I2, No automatic path from untrusted content

- **Before:** "outside the trusted boundary", "another writer, node or
  person", "owner acts signed by a device key the owner certified",
  "Without an owner act", "On an owner act", "owner-typed text", and "a
  person may also trust a poster by key".
- **After:** "outside the trusted sources", "another writer, node or
  principal", "principal acts signed by a device key the agent's
  principal certified", "Without a principal act", "On a principal
  act", "principal-typed text", and "a principal may also trust another
  principal by key".
- **What changes:**
  - A service account is principal of its own agents exactly as a
    person is, so it may widen what reaches its own agents and no one
    else's. The trusted sources stay per agent: only the agent's own
    principal's acts count.
  - Room governance, now room acts signed with seat keys, stays
    outside I2: a room act never widens what reaches an agent.
  - The closed set of writes to an agent does not change.
- **Where:** [wording](../srs/invariants.md); requirements
  [OWN-02][OWN], [OWN-11][OWN], [OWN-29][OWN], [LANE-31][LANE].
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 3. I4, One network boundary per component

- **Before:** "nodes the owner enrolled by key, off until the tenant
  turns it on", "hosts the owner names", "the tenant turned on", and a
  core list without the TUI.
- **After:** "the node's principal" in each place, and the TUI named
  in the core.
- **What changes:**
  - Only who decides. The node's principal enrols peers and turns B2
    and B3 on, as the tenant did.
  - The TUI was always a core surface (it opens no socket); naming it
    closes a gap the reviews found, and widens no boundary.
- **Where:** [wording](../srs/invariants.md); [§6.3](../srs/06-security.md).
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 4. I5, Bad data can be removed without destroying evidence

- **Before:** "a request that node's operator applies".
- **After:** "a quarantine request that node's principal applies".
- **What changes:** only the names; "request" is now always qualified.
- **Where:** [wording](../srs/invariants.md); requirements
  [SEC-12][SEC], [PEER-11][PEER].
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 5. I6, No silent failures

- **Before:** "visible to operators".
- **After:** "visible to the node's principal".
- **What changes:** only who is named; the Health surface shows it.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 6. I8, Isolation follows the principal

- **Before:** "Isolation follows the tenant", "one tenant's home",
  "another tenant or node", "attributed to its writer's key", and "this
  tenant's agents".
- **After:** "principal" in each place, and "attributed to its seat
  key".
- **What changes:**
  - A node serves one principal, a person or a service account, as it
    served one tenant.
  - A writer is named by its seat key, the one key each seat has, so
    attribution is unchanged.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

## Alternatives

- Keep "owner" for the person behind owner keys and owner acts, and
  rename the room's owner instead; declined, since the room relation
  is what people mean by owner, and the stakeholder chose it.
- Keep "trusted boundary"; declined, since I2 and I4 would use
  "boundary" in two senses.

## Consequences

The reviewer approved all six changes, so the record is accepted
(6 October 2026) and the new wording binds. The SRS, CLAUDE.md and
AGENTS.md carry it through their includes of
[invariants.md](../srs/invariants.md). I3, I7, I9 and I10 do not
change, and no requirement's traces change.

[LANE]: ../srs/05b-lane-requirements.md#511-room-lane
[PEER]: ../srs/05c-owner-and-peer-requirements.md#514-peer-network-peer
[OWN]: ../srs/05c-owner-and-peer-requirements.md#513-owner-acts-own
[SEC]: ../srs/06-security.md#62-security-requirements-sec
[v2]: ../../plan/2610012322_cairn-for-agent-fleets/domain-model-v2.md
[blind]:
  ../../plan/2610012322_cairn-for-agent-fleets/domain-model-review-blind.md
