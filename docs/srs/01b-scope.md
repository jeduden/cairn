---
title: "1.4 Scope"
summary: >-
  Scope by context layer: Cairn implements the record, pin, recall and
  kernel layers; long-term memory is out of scope.
  Claude first, room summaries from the facilitator, and the room
  view on loopback (B1), then on enrolled devices (B2), with any
  wider reach an open question (OQ-38).
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
(`cairn room-summary write`), signed with its device seat's key there;
the program that writes it is neither a Cairn component nor an agent,
and the core makes no model calls (I4). A room summary is untrusted
and never replaces the record (I1, I2).

The room view reaches the principal in two steps. It stays a client of
the record under VIEW-03, and every capability it offers also exists
in the CLI or MCP.

1. The browser room view on loopback (B1). Before B2, a phone reaches
   it only as a principal surface, through the principal's own tunnel
   outside Cairn, which Cairn sees as loopback; it is not a paired
   phone (§6.3).
2. The principal's other devices, as enrolled, mutually authenticated
   peers and paired phones (B2).

I4 binds Cairn's components, not the principal's own tunnel: the
room-view component listens only on loopback (B1), and the peer
component connects only to enrolled peers and paired phones (B2).
Whether the room view may reach browsers on other machines, or be
served from a host the principal names, is an open question: each
would need new I4 wording, approved by ADR with a security review,
before any requirement asks for it (OQ-38).
