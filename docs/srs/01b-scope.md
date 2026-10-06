---
title: "1.4 Scope"
summary: >-
  Scope by context layer: Cairn implements the record, pin, recall and
  kernel layers; long-term memory is out of scope.
  Claude first, room summaries from the facilitator, and a web
  service as a target interface, paced in four stages.
---
# 1.4 Scope

Cairn implements layers 1–4 of the agent context architecture:

| Layer | Name             | In Cairn v1                                                 |
| ----- | ---------------- | ----------------------------------------------------------- |
| 1     | Record           | Append-only, provenance-tagged event log of every agent run |
| 2     | Pins             | Qualifying pins, restored verbatim after every compaction   |
| 3     | Recall           | Pull-only recall, landmark index, restore blocks            |
| 4     | Kernel           | Hermetic kernel with read-only access to the record         |
| 5     | Long-term memory | **Not in scope.** Export interface only (§5.9)              |

Cairn serves Claude first. Version 1 supports Claude Code and the Claude
Agent SDK; other harnesses come later, and the design MUST NOT preclude
them (ADR-08, OQ-16).

Room summaries are in scope. When a run joins a room with much
content, its agent makes a summary request for a room summary of the
size it wants (LANE-33). The room's one facilitator, the service
account the owner appoints, writes it through its own node's CLI
(`cairn room-summary write`), signed with its device seat there; the
program that writes it is not a Cairn component, and the core makes no
model calls (I4). A room summary is untrusted and never replaces the
record (I1, I2).

A web service is a target interface, paced in four stages. Each stage
stays a client of the record under VIEW-03, and every capability it
offers also exists in the CLI or MCP.

1. The person's own browser on loopback (B1), as the room view does
   today; a paired phone may reach it through a tunnel the principal
   runs outside Cairn, which Cairn sees as loopback (§6.3).
2. The person's other devices, as enrolled, mutually authenticated
   peers and paired phones (B2).
3. Browsers on other machines with a login.
4. A service on a host the principal names.

Stages 1 and 2 fit I4 as written. Stages 3 and 4 need new I4 wording,
approved by ADR with a security review, before any requirement asks
for them (OQ-38).
