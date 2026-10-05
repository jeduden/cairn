---
id: ADR-2610050717
title: "Room automations and the inbound webhook endpoint"
status: proposed
summary: >-
  Proposed invariant changes for room automations, decided by the
  stakeholder as A1 to A4: I2's closed list gains automations under a
  person's recorded grant, and I4 gains an inbound webhook endpoint
  beyond loopback, off by default and signature-checked. Drafts the
  requirements that would follow; none is in the SRS until review.
---
# ADR-2610050717: Room automations and the inbound webhook endpoint

## Context

The stakeholder decided on 4 October 2026 how automations fit the room
model, in decisions A1 to A4 of the [automations note][auto]. A nightly
timer, a forge issue event or a merged pull request fires a rule a
person recorded in advance. Rooms need a running Cairn service, and the
timer runs in it. Forge events arrive at a webhook endpoint Cairn
provides, so a deployed node must be reachable from the forge.

Two accepted invariants stand in the way. I2's closed list of writes to
an agent has no entry for an automation, and OWN-09 lets only a current
owner act or a delegation grant start a turn. I4 lets B1 listen on
loopback only and B3 only send outbound, so no component may receive a
forge's webhook. CLAUDE.md makes each a design change needing security
review and a new major version, and
[ENG-29](../srs/10-engineering-quality.md#104-process) gates it on a
named reviewer's record. This ADR is that record, proposed.
[OQ-33](../srs/13-open-questions-and-risks.md#131-open-questions) points
here.

## Decision

The reviewer approves or declines each change. Until then no automation
requirement enters the SRS, and nothing below may be built past a
prototype.

| #   | Change                                            | Review  |
| --- | ------------------------------------------------- | ------- |
| 1   | I2 and OWN-09: automations under a recorded grant | pending |
| 2   | I4 and SEC-28: an inbound webhook endpoint        | pending |

### 1. I2 and OWN-09: automations under a recorded grant

- **Before:** Cairn writes to an agent only through I2's closed list:
  restore blocks, opt-in notices and fixed templates without an owner
  act; owner-typed text, fixed templates and endorsed posts on an owner
  act recorded at that time; delegated tasks under a delegation grant.
  Nothing else starts or resumes a turn (OWN-09).
- **After:** The list gains one entry: under an automation rule its
  person recorded as a widening owner act, a fixed template that names
  only ids and links, never an event's own text, delivered by the
  action the rule names.
- **What changes:**
  - A timer tick, a forge event or a landed commit can start a session
    on a new branch of a room, or post a notice, with no person present
    at that moment, because the person recorded the rule beforehand.
  - The started agent reads the triggering event itself, through a
    tool, inside the untrusted envelope, and its session counts as
    tainted (SEC-13), so its sensitive actions ask first.
  - No event text ever reaches a template, a pin or a notice, and no
    person may trust a bridge or a source (A2).
- **Where:** [wording](../srs/invariants.md); requirements
  [OWN-09][OWN], [OWN-23][OWN], [SEC-13][SEC].
- **Review:** pending.

### 2. I4 and SEC-28: an inbound webhook endpoint

- **Before:** B1 may listen on loopback only; B3 is outbound only
  (SEC-28), so forge events arrive only when the bridge polls.
- **After:** A new boundary-register row: a webhook endpoint on a
  deployed Cairn node that receives a forge's issue and pull request
  events. It is off by default, enabled by a person's act, lockable by
  managed policy, listens only on the addresses its configuration
  names, and verifies each delivery against the forge's webhook
  signature before recording it. Payloads are recorded as untrusted
  events under the endpoint's own participant id.
- **What changes:**
  - A node the forge can reach accepts inbound connections from it; the
    core still opens no socket.
  - A delivery that fails its signature check is refused, counted and
    audited, never recorded as an event.
- **Where:** [wording](../srs/invariants.md); requirements
  [SEC-19][SEC], [SEC-22][SEC], [SEC-28][SEC]; the
  [boundary register](../srs/06-security.md#63-boundary-register).
- **Review:** pending.

### Requirements that would follow

Drafted for review; each would land with its scenario, `@pending`, once
its change is approved.

- **Sources.** Every source of automation events (timer, webhook
  endpoint, local git, CI carrier) MUST be a room participant of kind
  bot with its own key, and its events MUST be untrusted posts stamped
  with its participant id.
- **The event kind.** A source MUST post messages of kind `event`,
  whose structural fields (kind, label, branch, schedule, link to a
  snapshot) are sanitized under LMK-03, and whose text no rule reads.
- **Rules.** An automation rule MUST be its person's widening owner act
  naming a trigger (a room, a source and an event kind, matched on
  structural fields only, never on text), one action, a fixed template,
  a budget and an expiry by count or revocation, never by clock in
  projections.
- **Closed action set.** An action MUST be one of: post a notice, open
  a room from a template, start a session on a new branch of the room,
  run a recorded command (SEC-29), close a branch or a room. No action
  MAY act outside its person's own agents and rooms.
- **Fixed templates.** A template MUST name only ids and links, and
  MUST NOT carry any field of the triggering event.
- **Budgets.** Each rule MUST carry token, session and firing budgets
  and a cap on concurrent runs; a firing beyond any MUST be refused,
  counted and raised as a Needs you item.
- **Firing keyed to the event.** Matching MUST be a projection of the
  record; a firing MUST be recorded as an event keyed by the triggering
  event's address, so replaying the record never fires twice. A merge
  MUST fire only when the landed commit matches the room's branch with
  a proven class (LANE-06), and MUST raise a Needs you item otherwise
  (A4).
- **Per-room off switch.** The room's owner MUST be able to turn every
  automation of a room off as a cut act.
- **Managed policy.** Managed policy MUST be able to disable
  automations and the webhook endpoint per host (SEC-22).
- **Silent failures.** Every firing, skip, refusal and failure MUST be
  recorded and counted (I6).
- **The boundary row.** The webhook endpoint MUST have its own row in
  the register, as change 2 states.

## Alternatives

- **Poll the forge from the bridge instead of a webhook.** Keeps I4 as
  it is, but adds latency and spends the forge's rate limits; declined
  by the stakeholder (A1), who chose a deployed endpoint.
- **Let the timer live in the core.** The core runs no resident process
  (NFR-09) and reads no clock in projections (I10); declined (A3).
- **Allow trust grants to a bridge.** Anyone who can open an issue would
  then instruct the person's agents; declined (A2).
- **Treat an automation as a delegation.** A delegation starts from an
  agent's act; an automation starts from an outside event, so it needs
  its own entry and its own review.

## Consequences

If both changes are approved, the ADR is accepted, I2 and I4 are
reworded in their single source, and the requirements above enter the
SRS with their scenarios, `@pending`. If either is declined, automations
wait, and forge events reach rooms only through the outbound bridge. In
either case, an automation never makes event text trusted: the started
agent reads it as untrusted data.

[OWN]: ../srs/05c-owner-and-peer-requirements.md#513-owner-acts-own
[SEC]: ../srs/06-security.md#62-security-requirements-sec
[auto]: ../../plan/2610012322_cairn-for-agent-fleets/automations.md
