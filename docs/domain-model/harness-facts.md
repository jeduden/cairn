---
title: "Harness facts"
order: "02"
summary: >-
  The harness's own facts that Cairn records and names: the harness, harness sessions and transcripts, hooks, turns and compaction.
---
# Harness facts

The harness's own facts, which Cairn records and names but never manages; of
them, Cairn changes only harness configuration, under I7.

- **Harness**: The external program that runs agents, such as Claude Code or the
  Agent SDK. It keeps their transcripts, compacts their context and reports to
  Cairn through hooks. Cairn records it, changes its configuration only under I7
  and never manages its transcripts or compaction; of Cairn's components, only
  the launcher starts and controls runs in it.
- **Harness session**: The harness's own unit, which yields one transcript, plus
  one per subagent where the harness writes them apart (ASM-03). Named only when
  describing the harness, never as a Cairn unit. One whose harness process
  descends from a run's tool call records no `user` event: its input, which
  that tool call supplied, is untrusted `harness_text` whatever the deployment
  mode, and its runs carry that run's recall taint, or recall taint from their
  first event where Cairn cannot tell that run (PRV-08, SEC-13).
- **Transcript**: The harness's file of what a harness session did. A
  **transcript source** is a transcript Cairn ingests (Ingest marker, `cairn
  ingest`); a shrunk transcript or a rewritten prefix starts a new **transcript
  generation** (REC-07). The **transcript roots** (`transcript.roots`) are the
  directories ingest reads transcripts from by default (REC-01, SEC-18). As a
  Cairn term, "source" is a transcript source or a trusted source; the harness's
  `source` field and §6.1's threat sources keep their own sense, as does a
  document's single source.
- **Hook**: The harness's callback into Cairn, carrying its **hook input**, a
  JSON object (§9.1). The core's hook handlers answer it. Git's hooks are always
  qualified, as the commit hook.
- **Turn**: One exchange between the harness and the model, from an input to the
  model reply that ends it. A **user turn** is the turn that text typed at the
  harness's own input starts (its prompt, or the terminal the launcher hosts): a
  person's message in `interactive` deployment mode, a pipeline's in
  `automation`. A line the launcher carried in is no user turn: the launcher
  records the carried text's commitment as its **turn trigger** (what started a
  turn, LANE-14), and ingest records the matching line as untrusted
  `harness_text` (LANE-14). In interactive deployment mode its `user` event,
  never the model reply, is a trusted source on the node whose hook handlers
  recorded it (I2, PRV-02). Only that typed text is `user`. Other free text the
  harness writes or adds to the model's input, such as an `isMeta` line,
  command output it writes as a `user` line, a compaction summary, context a
  hook adds, a launcher-carried line or a subagent's delegated task,
  is `harness_text`, never `user` and never trusted, unless it is a turn's
  model reply, a tool result, a file's or an MCP server's text or a subagent's
  report (PRV-08, PRV-03).
- **Compaction**: The harness replacing earlier context with a compaction
  summary when the context window fills or when asked. Cairn neither performs
  nor controls it (NG1); it records it and restores pins after it (I3).
  **Compaction guidance** is fixed text Cairn ships that a `PreCompact` hook
  handler returns to the harness for its compaction, never record content
  (PIN-07, I2).
