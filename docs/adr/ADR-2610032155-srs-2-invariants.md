---
id: ADR-2610032155
title: "Security review of the SRS 2.0 invariant changes"
status: proposed
summary: >-
  The named security reviewer's record for the SRS 2.0 invariant and
  constraint changes: I4 restated by network boundary, one signed log
  per writer, owner acts resting on the sandbox, and delegation.
  Accepting it lets B0-crossing components leave the prototype stage.
---
# ADR-2610032155: Security review of the SRS 2.0 invariant changes

## Context

SRS 2.0-draft rewords six invariants and two constraints and adds three
trust rules, carried from plan 2610012322's proposal after seven
persona-review rounds and the stakeholder's decisions of 3 October 2026.
CLAUDE.md treats an invariant change as a design change that needs a
security review and a new major version.
[ENG-29](../srs/10-engineering-quality.md#104-process) makes the review a
gate: no invariant change is accepted, and no component that crosses B0
is built past a prototype, until this record holds a named reviewer's
approval. The reviewer is the stakeholder, @jeduden.

## Decision

The reviewer approves or declines each change below. The status moves to
accepted only when every change reads approved. The threat model
([§6.1](../srs/06-security.md#61-threat-model)) and the boundary register
([§6.3](../srs/06-security.md#63-boundary-register)) are reviewed with the
changes they control.

| #   | Change                                                    | Review                          |
| --- | --------------------------------------------------------- | ------------------------------- |
| 1   | I1, Nothing is lost                                       | approved, 3 October 2026        |
| 2   | I2, No automatic path from untrusted content to the model | approved, 3 October 2026        |
| 3   | I4, Cairn never talks to the network                      | pending, revised for one binary |
| 4   | I5, Bad data can be removed without destroying evidence   | approved, 3 October 2026        |
| 5   | I8, Isolation follows the tenant                          | approved, 3 October 2026        |
| 6   | I10, Everything derived is rebuildable                    | approved, 3 October 2026        |
| 7   | CON-04, No network in the core                            | pending, revised for one binary |
| 8   | CON-06, No central service (new)                          | approved, 3 October 2026        |
| 9   | Owner acts rest on the sandbox                            | approved, 3 October 2026        |
| 10  | Owner key certifies device keys                           | approved, 3 October 2026        |
| 11  | Delegation joins I2's closed list                         | approved, 3 October 2026        |
| 12  | Room pins reach every member agent (new)                  | pending                         |

### 1. I1, Nothing is lost

- **Before (1.4):** Every event stays recoverable by a stable address: its
  `seq`, unique within one project.
- **After (2.0):** Every event stays recoverable by its address (writer, seq),
  on every node that holds that writer's log. Redaction on import joins
  redaction before storage as a recorded exception.
- **What changes:**
  - A `seq` was one counter per project, so two machines could never append to
    the same project. Each writer now has its own log, and an address names
    both.
  - Importing someone else's lane may redact secrets first; that removal is
    recorded like any other.
- **Where:** [wording](../srs/01-introduction.md#13-invariants); requirements
  [REC-06][REC], [REC-22][REC], [REC-23][REC].
- **Review:** approved, 3 October 2026.

### 2. I2, No automatic path from untrusted content to the model

- **Before (1.4):** Untrusted content (tool output, web, MCP, files, assistant
  text) reaches the model only when Claude calls a recall tool, inside an
  untrusted envelope.
- **After (2.0):** The same rule, with every other writer, node and person
  named as untrusted, and the trusted boundary bounded by key. Cairn may write
  to an agent only through a closed list: restore blocks, opt-in notices and
  fixed templates without an owner act; owner-typed text, fixed templates and
  endorsed posts on an owner act; delegated tasks under a grant.
- **What changes:**
  - Other people's posts never reach your agent unless you endorse them,
    sending exactly what you saw.
  - Every automatic write Cairn makes is listed, so nothing outside the list
    can reach a model.
- **Where:** [wording](../srs/01-introduction.md#13-invariants); requirements
  [PRV-02][PRV], [PRV-10][PRV], [OWN-03][OWN], [OWN-08][OWN], [OWN-09][OWN].
- **Review:** approved, 3 October 2026.

### 3. I4, Cairn never talks to the network

- **Before (1.4):** "Cairn never talks to the network": no listening sockets,
  no outbound connections, no telemetry.
- **After (2.0):** "Each component stays inside one declared network boundary,
  and only the core reaches the model." B0, the core, still opens no socket.
  B1 may listen on loopback only. B2 (peers you enrol by key) and B3
  (publishing to hosts you name) are off until you turn them on, and managed
  policy can lock each off. Boundaries hold per component and per process, so
  every component can be a subcommand of the one `cairn` binary.
- **What changes:**
  - The core's guarantee is unchanged: a hook, MCP or CLI process opens no
    socket.
  - `cairn ui`, `cairn peer` and the rest are subcommands of the same binary,
    not separate binaries. A process runs exactly one component, fixed when it
    starts, and a core process never starts a network component.
  - CI checks each component's import closure, runs each under its boundary's
    sandbox, and fails when any linked package touches the network or starts a
    program while initialising, since Go initialises every linked package in
    every process.
  - What you trade: the shipped file contains network code, so the guarantee
    is a property of each process, not of the binary's symbol table.
  - The browser lane view on localhost and opt-in peer to peer become possible
    inside one product.
- **Where:** [wording](../srs/01-introduction.md#13-invariants); requirements
  [SEC-01][SEC], [SEC-19][SEC], [SEC-22][SEC], [ENG-12][ENG], [ENG-16][ENG].
- **Review:** pending, revised for one binary.

### 4. I5, Bad data can be removed without destroying evidence

- **Before (1.4):** Any event, span, session or derived artefact can be
  quarantined from recall immediately.
- **After (2.0):** The same, adding writers, with the quarantine binding the
  node that records it; another node receives it as a request its operator
  applies.
- **What changes:**
  - A quarantine on your laptop cannot silently change a co-author's copy;
    their node is asked, and the answer is shown.
- **Where:** [wording](../srs/01-introduction.md#13-invariants); requirements
  [SEC-12][SEC], [PEER-11][PEER].
- **Review:** approved, 3 October 2026.

### 5. I8, Isolation follows the tenant

- **Before (1.4):** All state is bound to one tenant's home; Cairn refuses
  state it does not own.
- **After (2.0):** All local state stays bound to one home. Content another
  tenant or node wrote may be held, as theirs: attributed to its writer's key,
  untrusted, and never widening what this tenant's agents trust or recall.
- **What changes:**
  - Holding a co-author's log in a shared lane no longer breaks the invariant,
    and cannot raise its trust.
- **Where:** [wording](../srs/01-introduction.md#13-invariants); requirements
  [PRV-02][PRV], [PRV-09][PRV].
- **Review:** approved, 3 October 2026.

### 6. I10, Everything derived is rebuildable

- **Before (1.4):** Derived state is a deterministic function of the
  append-only record.
- **After (2.0):** Derived state, statuses, queues and verdicts included, is a
  deterministic function of the set of writer logs a node holds and its own
  keys, whatever order the logs arrived in.
- **What changes:**
  - Two peers holding the same logs show the same lane, whatever they synced
    first.
- **Where:** [wording](../srs/01-introduction.md#13-invariants); requirements
  [ADM-08][ADM], [PEER-03][PEER].
- **Review:** approved, 3 October 2026.

### 7. CON-04, No network in the core

- **Before (1.4):** No network access in any binary that ships P0 features.
- **After (2.0):** No network access in the core component; every other
  component is confined to the one boundary the register assigns it, whether
  or not it shares an executable with the core, checked at build level. CON-02
  now ships Cairn as one binary whose components are its entry points.
- **What changes:**
  - Each component's reach is a row in the [boundary
    register](../srs/06-security.md#63-boundary-register), and CI fails on
    more.
- **Where:** [wording](../srs/02-context.md#24-constraints); requirements
  [SEC-01][SEC], [SEC-19][SEC], [ENG-16][ENG].
- **Review:** pending, revised for one binary.

### 8. CON-06, No central service (new)

- **Before (1.4):** No such constraint.
- **After (2.0):** No feature at any boundary may depend on a central or
  third-party service; every node can serve what it holds, and self-hosting is
  the normal case.
- **What changes:**
  - Peering needs no rendezvous or relay host; a hosted peer is optional and
    may be blind (PEER-12).
- **Where:** [wording](../srs/02-context.md#24-constraints); requirements
  [PEER-02][PEER].
- **Review:** approved, 3 October 2026.

### 9. Owner acts rest on the sandbox

- **Before (1.4):** No owner acts existed; a process of the same OS user was
  out of scope.
- **After (2.0):** Owner acts fall in three classes. Cut and neutral acts need
  an authenticated surface or a terminal. Widening acts need the same while
  every session runs in a sandbox that blocks the residual risks, and your
  recorded acceptance of the open risks otherwise; managed policy may forbid
  acceptance. Per-act WebAuthn is optional hardening.
- **What changes:**
  - Against an unsandboxed agent, the seven [residual
    risks](../srs/06-security.md#61-threat-model) are accepted openly, shown
    beside each such session, rather than fought with checks an agent can
    pass.
- **Where:**
  [wording](../srs/05c-owner-and-peer-requirements.md#513-owner-acts-own);
  requirements [OWN-11][OWN], [OWN-12][OWN], [OWN-22][OWN].
- **Review:** approved, 3 October 2026.

### 10. Owner key certifies device keys

- **Before (1.4):** Every imported event untrusted, so your phone and second
  machine could answer nothing.
- **After (2.0):** An owner key certifies each device key with a scope and a
  rule ceiling; owner acts from your other devices count within that scope. A
  phone may only allow once or deny.
- **What changes:**
  - You can answer a held request from your phone; a stolen device is cut off
    by revocation, including any keys it issued.
- **Where:**
  [wording](../srs/05-functional-requirements.md#52-provenance-and-trust-prv);
  requirements [PRV-10][PRV], [OWN-17][OWN], [PEER-07][PEER].
- **Review:** approved, 3 October 2026.

### 11. Delegation joins I2's closed list

- **Before (1.4):** No agent-to-agent path existed outside the harness's own
  subagents.
- **After (2.0):** An agent may delegate beyond its session only under a grant
  its principal recorded, naming targets, rule ceiling, budget and expiry. The
  task arrives in a sourced template; the delegate inherits the ceiling and
  taint; results return only when pulled. Another person's agent takes work
  only under their acceptance grant.
- **What changes:**
  - Orchestrators can fan work out to other sessions and machines without
    turning an injected instruction into an unbounded agent.
- **Where:**
  [wording](../srs/05c-owner-and-peer-requirements.md#513-owner-acts-own);
  requirements [OWN-23][OWN], [OWN-24][OWN], [OWN-25][OWN], [OWN-26][OWN].
- **Review:** approved, 3 October 2026.

### 12. Room pins reach every member agent (new)

- **Before (1.4):** Only an agent's own person's words reach it automatically,
  and pins restore only to the person's own agents.
- **After (2.0):** A room's pins, its intent first, reach every agent that is
  a member of the room, whoever wrote them. Only an agent's own person can add
  it to a room, and that act accepts the room's pins for that agent.
- **What changes:**
  - Teammates' agents in a shared room work from the same intent and pins as
    yours, by design (stakeholder decision Q6, 4 October 2026).
  - Another person's pin text reaches your agent automatically once you admit
    it to their room; admission is the bound. See the [concepts
    note](../../plan/2610012322_cairn-for-agent-fleets/concepts.md).
- **Where:** [wording](../srs/01-introduction.md#13-invariants); requirements
  [OWN-01][OWN], [PIN-01][PIN], [PIN-10][PIN].
- **Review:** pending.

## Alternatives

- Keep I4 as written and ship the lane view and the network side as a
  separate product with its own SRS; declined by the stakeholder (D1).
- One binary per boundary (`cairn-ui`, `cairn-peer` and so on), so the
  core's file links no network code; declined by the reviewer, who wants
  a single binary. The boundary moves from the file to the process.
- Keep one project chain; it rules out peering and the multi-machine
  persona.
- A per-act WebAuthn assertion as the mandatory guard against an
  unsandboxed agent; the persona reviews showed no software-only check
  holds, and the stakeholder chose the sandbox with recorded risk
  acceptance (OQ-29).

## Consequences

Until this record is accepted, the 2.0 wording stands as a draft, and
plan 2610022338's lane view and every B1 to B3 component stay prototypes
behind a build tag that release builds exclude. A declined change goes
back to the proposal.

[PIN]: ../srs/05-functional-requirements.md#53-pins-pin
[REC]: ../srs/05-functional-requirements.md#51-record-rec
[PRV]: ../srs/05-functional-requirements.md#52-provenance-and-trust-prv
[ADM]: ../srs/05-functional-requirements.md#58-administration-and-lifecycle-adm
[LANE]: ../srs/05b-lane-requirements.md#511-lane-lane
[OWN]: ../srs/05c-owner-and-peer-requirements.md#513-owner-acts-own
[PEER]: ../srs/05c-owner-and-peer-requirements.md#514-peer-network-peer
[SEC]: ../srs/06-security.md#62-security-requirements-sec
[ENG]: ../srs/10-engineering-quality.md#104-process
