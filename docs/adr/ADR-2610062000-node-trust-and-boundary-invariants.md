---
id: ADR-2610062000
title: "Security review of the node-trust and boundary invariants"
status: accepted
summary: >-
  The security reviewer's record for rewording I1, I2, I4, I7 and I8
  after the fourth round of blind domain-model reviews: unsigned
  trusted sources count only on the node that recorded them while what
  the principal signed crosses nodes, managed policy may set retention,
  only the core builds what reaches the model, B1 allows a same-user
  local endpoint, B2 reaches paired phones, and I7 names harness
  configuration. Accepted by the stakeholder on 6 October 2026.
---
# ADR-2610062000: Security review of the node-trust and boundary invariants

## Context

The fourth round of blind domain-model reviews ([merged
note][blind4]) found the invariants out of step with the requirements
and the model in five places:

- **I2 and I8** distrusted anything "another node" produced, while
  PRV-10 and PIN-11 trust the principal's acts and device-seat pins
  from its other devices, and the model said trust levels are the same
  on every node of a principal.
- **I1** let only the node's principal remove data under a retention
  policy, while SEC-22 lets managed policy set one.
- **I4** said only the core reaches the model, while the launcher
  carries steers and delegated tasks into the harness's input; it
  confined B1 to loopback, while SEC-01, SEC-29 and §6.3 allow a local
  endpoint only the same OS user can reach; and it let B2 reach only
  nodes, while a paired phone reaches its node over B2.
- **I7** spoke of "agent configuration", undefined, and of managed
  policy alone, while ADM-03 also protects the harness's managed
  settings.

The stakeholder decided the trust question on 6 October 2026 (the
note's section 5, decision 1) and asked for the recommended option on
the rest. CLAUDE.md treats an invariant change as a design change that
needs a security review and a new major version.
[ENG-29](../srs/10-engineering-quality.md#104-process) makes the review
a gate. The reviewer is the stakeholder, @jeduden.

## Decision

The reviewer approves or declines each change below. The status moves to
accepted only when every change reads approved or withdrawn.

| #   | Change                                         | Review   |
| --- | ---------------------------------------------- | -------- |
| 1   | I2, what crosses nodes                         | approved |
| 2   | I8, the principal's other nodes                | approved |
| 3   | I1, retention set by managed policy            | approved |
| 4   | I4, the core builds what reaches the model     | approved |
| 5   | I4, B1's same-user endpoint and B2's phones    | approved |
| 6   | I7, harness configuration and managed settings | approved |

### 1. I2, what crosses nodes

- **Before:** "anything another writer, node or principal produced";
  "principal acts signed by a device key the agent's principal
  certified".
- **After:** "anything another node or principal produced, except what
  the agent's principal signed through a device key it certified";
  "principal acts, posts and pins signed through a device key the
  agent's principal certified".
- **What changes:**
  - Unsigned trusted sources (this node's `operator`, `harness_meta`
    and structural events, its interactive `user` turns) stay trusted
    only on the node that recorded them. Another node's typed turn is
    no more trusted than any other text.
  - What the principal signed through a certified device key, within
    that key's scope, is trusted alike on all its nodes: its principal
    acts, as before, and its device-seat posts and pins. A trust level
    is thus derived per event, per principal and per reading node,
    still a deterministic function of the logs and the node's own key
    set (I10).
  - "Another writer" goes: a run spans several writers on its own
    node, and the line between trusted and untrusted is the node and
    the signature, not the writer.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 2. I8, the principal's other nodes

- **Before:** "Content another principal or node wrote is held as
  theirs".
- **After:** "Content another principal wrote, or another node wrote
  outside what this principal signed through a device key it
  certified, is held as theirs".
- **What changes:** the same exception as change 1. A sandbox node
  holding only a token key signs nothing through a device key, so its
  content stays untrusted on every other node.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 3. I1, retention set by managed policy

- **Before:** "data the node's principal explicitly purges or removes
  under a retention policy".
- **After:** "data the node's principal explicitly purges, or removes
  under a retention policy that principal or managed policy sets".
- **What changes:** the invariant now states what SEC-22 already
  allows. Every removal is still recorded, and a purge receipt says
  what it erased.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 4. I4, the core builds what reaches the model

- **Before:** "only the core reaches the model".
- **After:** "only the core builds what reaches the model", and "the
  launcher carries into the harness's input only text the core built".
- **What changes:** the launcher (B1) delivers steers, corrections and
  delegated tasks into the harness, as OWN-03 and §9.5 already say, but
  never composes them: the core builds every such text in its fixed
  templates. No new path to the model opens.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 5. I4, B1's same-user endpoint and B2's phones

- **Before:** "a component may listen on loopback only"; "connections
  to nodes the node's principal enrolled by key".
- **After:** "a component may listen only on loopback or on a local
  endpoint only the same OS user can reach"; "connections to nodes and
  paired phones the node's principal enrolled by key".
- **What changes:**
  - A local endpoint only the same OS user can reach, such as a
    socket file only that user can open, is narrower than loopback,
    which every local user can reach. SEC-01, SEC-29 and §6.3 already
    allow it.
  - A paired phone is enrolled by key like a peer and reaches its node
    over the same encrypted, mutually authenticated channel, limited
    to reading and to held permission requests (PRV-10).
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 6. I7, harness configuration and managed settings

- **Before:** "Cairn changes agent configuration … and never
  overrides managed policy."
- **After:** "Cairn changes harness configuration … and never
  overrides managed policy or the harness's managed settings."
- **What changes:** the invariant names what Cairn configures, the
  harness's hooks and MCP registration, and the harness's own managed
  settings that ADM-03 already protects.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

## Alternatives

- Trust unsigned sources from every node of the principal; declined,
  since one compromised node could then plant trusted text in every
  other node's agents without a signature or a scope.
- Confine B1 to loopback and cut the same-user endpoint from SEC-01;
  declined, since the endpoint is the narrower of the two.

## Consequences

The reviewer approved all six changes, so the record is accepted
(6 October 2026) and the new wording binds. CLAUDE.md, AGENTS.md and
README carry it through their includes of
[invariants.md](../srs/invariants.md). No requirement's traces change.

[blind4]:
  ../../plan/2610012322_cairn-for-agent-fleets/domain-model-review-blind-4.md
