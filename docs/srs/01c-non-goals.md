---
title: "1.5 Non-goals for v1"
summary: >-
  What Cairn v1 deliberately does not do, NG1 onward, each with its
  reason.
---
# 1.5 Non-goals for v1

NG6, NG7 and NG8 were retired by the stakeholder on 6 October 2026, and
their ids are not reused: Cairn serves Claude first rather than ruling
other harnesses out ([§1.4](01b-scope.md)), room summaries are in scope
(LANE-33), and whether Cairn may run, push or merge code is open
(OQ-37).

| #   | Non-goal                                                                     | Reason                                                                                                                                          |
| --- | ---------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| NG1 | Replacing or disabling Claude Code's native compaction                       | Behaviour of the harness is outside our control; Cairn makes compaction recoverable instead. Revisited via OQ-01.                               |
| NG2 | Long-term semantic memory, fact extraction, or automatic promotion           | Largest poisoning surface; belongs to a separate layer-5 system with its own policy                                                             |
| NG3 | Embedding or vector search                                                   | BM25 is deterministic, needs no model calls at ingest, and is sufficient for v1 (Scroll uses BM25)                                              |
| NG4 | Peering in v1.0, and a shared mutable store at any version                   | Signed per-writer logs replicate between enrolled peers as P2 (PEER) after their own security review. OQ-07 closes with PEER.                   |
| NG5 | A graphical interface that is required, or reachable from off the machine    | The room view is an optional machine-local client; every capability stays in the CLI or MCP (VIEW-03)                                           |
| NG9 | Live co-editing of one worktree by several writers (nodes or people) in v1.0 | A run and its subagents sharing a worktree are in scope (LANE-19); a conflict-free method for several writers is chosen by ADR after PEER ships |
