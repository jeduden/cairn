---
title: "1.4 Scope"
summary: >-
  Scope by context layer: Cairn implements the record, pinned context,
  working view and compute layers; durable memory is out of scope.
  Claude first, and room summaries from the facilitator.
---
# 1.4 Scope

Cairn implements layers 1–4 of the agent context architecture:

| Layer | Name           | In Cairn v1                                                 |
| ----- | -------------- | ----------------------------------------------------------- |
| 1     | Record         | Append-only, provenance-tagged event log of every agent run |
| 2     | Pinned context | Constraints that survive every compaction verbatim          |
| 3     | Working view   | Pull-only recall, landmark index, post-compaction restore   |
| 4     | Compute        | Hermetic kernel with read-only access to the record         |
| 5     | Durable memory | **Not in scope.** Export interface only (§5.9)              |

Cairn serves Claude first. Version 1 supports Claude Code and the Claude
Agent SDK; other harnesses come later, and the design MUST NOT preclude
them (ADR-08, OQ-16).

Room summaries are in scope. An agent joining a room with much content
asks for a summary of the size it wants (LANE-33). The room's
facilitator writes it, outside the core, which makes no model calls
(I4). A summary is untrusted and never replaces the record (I1, I2).
