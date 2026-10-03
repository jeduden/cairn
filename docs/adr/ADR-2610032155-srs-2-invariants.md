---
id: ADR-2610032155
title: "Security review of the SRS 2.0 invariant changes"
status: proposed
summary: >-
  The named security reviewer's record for the SRS 2.0 invariant and
  constraint changes: I4 restated by network boundary, one signed log
  per writer, and owner acts resting on the sandbox. Accepting it lets
  B0-crossing components leave the prototype stage (ENG-29).
---
# ADR-2610032155: Security review of the SRS 2.0 invariant changes

## Context

SRS 2.0-draft rewords six invariants and four constraints, carried
from plan 2610012322's proposal after seven persona-review rounds and
the stakeholder's decisions of 3 October 2026. CLAUDE.md treats an
invariant change as a design change that needs security review and a
new major version. ENG-29 makes the review a gate: no invariant change
is accepted, and no component that crosses B0 is built past a
prototype, until an ADR records a named human security reviewer's
approval. The stakeholder, @jeduden, is that reviewer.

## Decision

The reviewer approves, or declines, each change below. The status
moves to accepted only when every row reads approved.

| Change                                                                                                         | Requirements                           | Review  |
| -------------------------------------------------------------------------------------------------------------- | -------------------------------------- | ------- |
| I1: stable address (writer, seq); redaction on import is an exception                                          | REC-06, REC-22, REC-23                 | pending |
| I2: other writers, nodes and people are untrusted; the closed set of automatic writes; the principal's endorse | PRV-02, PRV-10, OWN-03, OWN-08, OWN-09 | pending |
| I4: each component inside one boundary, B0 to B3; only the core reaches the model                              | SEC-01, SEC-19, SEC-22, ENG-12, ENG-16 | pending |
| I5: a quarantine binds the node that records it; peers get a request                                           | SEC-12, PEER-11                        | pending |
| I8: content another tenant or node wrote is held as theirs                                                     | PRV-02, PRV-09                         | pending |
| I10: derived state is a function of the set of writer logs, in any order                                       | ADM-08, PEER-03                        | pending |
| CON-04: no network in the core; every other binary confined to its boundary                                    | SEC-19                                 | pending |
| CON-06: no dependence on a central or third-party service                                                      | PEER-02                                | pending |
| Owner acts rest on the sandbox; risk acceptance for unsandboxed sessions                                       | OWN-11, OWN-12, OWN-22                 | pending |
| Owner key certifies device keys (the I2 review PRV-10 requires)                                                | PRV-10, OWN-17, PEER-07                | pending |

The threat model of §6.1 (actors, T14 to T24, residual risks R1 to R7)
and the boundary register of §6.3 are reviewed with the rows they
control.

## Alternatives

- Keep I4 as written and ship the lane view and the network side as a
  separate product with its own SRS; declined by the stakeholder (D1).
- Keep one project chain; it rules out peering and the multi-machine
  persona.
- A per-act WebAuthn assertion as the mandatory guard against an
  unsandboxed agent; the persona reviews showed no software-only check
  holds, and the stakeholder chose the sandbox with recorded risk
  acceptance (OQ-29).

## Consequences

Until this ADR is accepted, the 2.0 wording stands as a draft, and
plan 2610022338's lane view and every B1 to B3 component stay
prototypes behind a build tag that release builds exclude. A declined
row sends its change back to the proposal.
