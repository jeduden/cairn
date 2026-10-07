---
title: "4. Reference architecture (non-normative)"
summary: >-
  Non-normative reference architecture: components, event-sourced
  state, the compaction and aggregation scenarios, and design
  decisions ADR-01..10.
---
# 4. Reference architecture (non-normative)

This section describes the intended design so that requirements can be read in
context. It is not binding; §5–§10 are.

## 4.1 Overview

```text
 ┌──────────────────────────── Claude Code / Agent SDK harness ────────────────────────────┐
 │                                                                                           │
 │   hooks (command, stdin JSON)                          MCP client (stdio)                 │
 │        │                                                     │                            │
 └────────┼─────────────────────────────────────────────────────┼────────────────────────────┘
          ▼                                                     ▼
   cairn hook <hook>                                      cairn mcp
   (short-lived process)                                  (process per run)
          │                                                     │        spawns
          │                                                     ├──────────────► cairn kernel-worker
          ▼                                                     ▼                (rlimits, hermetic,
 ┌────────────────────────────────────────────────────────────────────────┐     read-only store)
 │                        cairn core (Rust library)                       │
 │                                                                        │
 │  adapter ─► parse ─► provenance ─► redact ─► flag ─► append ─► index   │
 │  (harness)                                          (seq, hash chain)  │
 │                                                                        │
 │  query compiler · recall · envelope · landmarks · pins · quarantine    │
 │  restore builder (TrustedText only) · audit · counters                 │
 └───────────────┬───────────────────────────────────┬────────────────────┘
                 ▼                                   ▼
 ┌─ store ────────────────────────────────────────────────────────────────┐
 │ ┌──────────────────────────────┐    ┌──────────────────────────────┐   │
 │ │ SQLite, WAL                  │    │ payload store (REC-09 names) │   │
 │ │ events · events_fts · spans  │    │ large tool outputs, files    │   │
 │ │ landmarks · pins · quarantine│    └──────────────────────────────┘   │
 │ │ counters · audit · meta      │                                       │
 │ └──────────────────────────────┘                                       │
 └────────────────────────────────────────────────────────────────────────┘
          CAIRN_HOME/                         (0700 dirs, 0600 files, one principal's)
```

## 4.2 Components

The components are the closed set of the
[domain model](../domain-model.md#concepts) and §6.3, each inside one
network boundary (I4).

| Component                | Responsibility                                                                                                                                                                                                                                                                                                                                                                     |
| ------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Core (B0)                | The hook handlers and the CLI, each harness adapter's transcript and hook part among them, the MCP server, the kernel worker, the TUI, and everything that builds what reaches the model; opens no socket and starts no program but its kernel worker.                                                                                                                             |
| Room-view component (B1) | `cairn ui`: serves the room view on loopback, or on a local endpoint only the same OS user can reach; off until the principal starts it.                                                                                                                                                                                                                                           |
| Launcher (B1)            | `cairn launch`: through each harness adapter's run part, starts, hosts and controls runs, pauses them at the harness prompt and records sandbox state (OWN-22); carries into the harness input only text the core built; executes witness checks; the only component but the core, for its kernel worker, that starts another program (SEC-29); off until the principal starts it. |
| Peer component (B2)      | `cairn peer-component on\|off`: replicates segments with peers the node's principal enrolled by key and serves paired phones; off until turned on.                                                                                                                                                                                                                                 |
| Publish component (B3)   | Read-only publishing and the git carrier, which the node's principal enables per remote and the room's owner per room (PEER-08); off until turned on.                                                                                                                                                                                                                              |
| Bridge component (B3)    | Outbound exchange with hosts the node's principal names: the forge bridge, the CI carrier and the notification bridge (SEC-28); off until turned on.                                                                                                                                                                                                                               |

The core's parts:

| Part of the core      | Responsibility                                                                                                                                                                                                                                                                                                                       |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `cairn hook <hook>`   | The hook handlers: read hook input from stdin, perform bounded processing within the hook budget, write hook output to stdout, always fail open (I9).                                                                                                                                                                                |
| `cairn mcp`           | The MCP server, one per run over stdio, exposing recall, landmarks, pins, room and kernel tools. Read-mostly.                                                                                                                                                                                                                        |
| `cairn kernel-worker` | The kernel worker, a child process of `cairn mcp` running the hermetic kernel under OS resource limits with a read-only store handle.                                                                                                                                                                                                |
| CLI                   | Install, verify, rebuild, quarantine, purge, backup creation and backup restore, migrate, counters, audit, doctor, and principal acts at a terminal (OWN-12).                                                                                                                                                                        |
| TUI                   | A reduced client of the room view on a terminal (VIEW-14) and, like the CLI, a principal surface there (OWN-02).                                                                                                                                                                                                                     |
| Core library          | All logic; no global state; every blocking operation takes a deadline, directly or through a cancellation signal a deadline fires.                                                                                                                                                                                                   |
| Harness adapter       | Not a component: Cairn's code for one harness, split between the core and the launcher. Its transcript and hook part, among the core's hook handlers and CLI, parses transcript formats and hook I/O for Claude Code (and later other harnesses), opening no socket and starting no process; its run part sits in the launcher (B1). |

## 4.3 Event-sourced state

The record is authoritative: everything else derives from it. Acts are
themselves events in the record. Principal acts, such as adding, editing or
unpinning a pin of a type that restores written from a device seat a device key
certified, a stamp, a quarantine, its release and a purge, are `operator` events
on the device seat of the device that signs them, in the room they act on, which
that node first joins without admission when its principal has a member seat
there. One that acts on no room or on a room its principal has no member seat
in, such as rejecting a foreign room, or that a paired phone signs, goes to that
device seat in the personal room, naming the room, and that room shows it by
address as it shows a cross-room post (OWN-02, LANE-29). Expire acts are
`operator` events on the device seat of the node that signs them. Room acts sit
in the writer of the seat that signs them: a post with provenance `post`, a room
summary `summary`, never trusted (LANE-33), every other room act with that
seat's pin class, `operator` for a device seat and `assistant` for a run seat
(LANE-31, PRV-01). A purge under a retention policy is not an act: the node
records it naming the policy, whose setting was the act. Every tombstone is a
`structural` event (PRV-01). Every other table (FTS index, spans, landmarks,
active pins, quarantine set, run and integrity statuses, queues, stats) is a
derived artifact that `cairn rebuild` reproduces exactly from the writer logs
the node holds and the node's own key set (I10). Purge, the only way content is
destroyed, removes content but leaves a tombstone event carrying the removed
addresses, counts, reason, the principal or retention policy that purged, and
the commitments of the removed events (REC-17, ADM-07), never a hash of the
removed content, so rebuilds stay deterministic, purges stay auditable and
nothing retained confirms a guess at what was purged.

## 4.4 Key scenarios

### Compaction (the core loop)

1. Context approaches its limit; Claude Code fires `PreCompact`.
2. The hook handler, `cairn hook PreCompact`, ingests the transcript up to
   now (bounded), closes the current span, updates landmarks, and returns
   compaction guidance if supported: fixed text Cairn ships, never record
   content (PIN-07, I2).
3. Claude Code compacts. `PostCompact` fires; Cairn records `compact_summary` as
   a `harness_text` event.
4. Claude Code fires `SessionStart` with `source = compact`. Cairn returns the
   **restore block**: the qualifying pins (PIN-10, verbatim), the landmark
   index, and the recall hint.
5. Later, the agent needs a detail that compaction dropped. It calls
   `event_search`, then `event_expand` on the returned address
   range, and receives the exact original inside an untrusted-data envelope.

### Aggregation over large history

1. The agent calls `kernel_exec` with a script such as
   `hits = cairn.event_search("timeout", kind="tool_result")` followed by a
   loop that extracts and counts error codes.
2. The worker binds results to the run's kernel variables; only the script's
   explicit `print` output (capped) returns to the agent, tainted by the events
   it derives from.

## 4.5 Design decisions

| ADR    | Decision                                                                                                                                                                                                                                                                                          | Alternatives considered                                                                      | Rationale                                                                                                                                                                                                                                                                                         |
| ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ADR-01 | **Daemonless core.** Hook handlers and per-run MCP servers open the store directly. The room-view component, the launcher, and the peer, publish and bridge components are optional, started by the principal, never on a hook or recall path, and their failure never touches an agent run (I9). | Long-running daemon on loopback (lcm)                                                        | Removes the port, daemon-credential, stale-daemon, and wedged-daemon failure classes observed in lcm; zero idle footprint; simpler isolation (I8, I9).                                                                                                                                            |
| ADR-02 | **Event sourcing.** Every derived artifact, including the effect of every act, is rebuilt from the writer logs a node holds.                                                                                                                                                                      | Mutable tables                                                                               | Makes verification, rollback, and forensic replay possible (I5, I10).                                                                                                                                                                                                                             |
| ADR-03 | **Pull-only recall.** Automatic injection accepts only the `TrustedText` type, which can be constructed only from qualifying pins, sanitized structural fields and fixed text Cairn ships.                                                                                                        | Per-prompt memory hints (lcm, claude-mem)                                                    | Poisoning evidence (§3, item 3). Enforced by the type system, not convention (I2).                                                                                                                                                                                                                |
| ADR-04 | **Hermetic Starlark kernel by default**; external Python kernel as optional P2.                                                                                                                                                                                                                   | Python/Jupyter kernel as default                                                             | Starlark (Python dialect, implemented in Rust) has no filesystem or network unless the host provides it, is deterministic, supports step limits, and needs no Python runtime (CON-03). Risk: less capable than Python for Claude. Gate: spike S5 measures Claude's task success with each.        |
| ADR-05 | **BM25 over SQLite FTS5; no embeddings in v1.**                                                                                                                                                                                                                                                   | Vector search, hybrid                                                                        | Deterministic, no model calls at ingest, no extra data leaving the machine; Scroll reached strong results with BM25.                                                                                                                                                                              |
| ADR-06 | **Distribution as a Claude Code plugin** (hooks + MCP registration) plus the core executable (CON-02). `cairn install` edits settings only as a fallback.                                                                                                                                         | Tool rewrites `settings.json` (lcm)                                                          | Uses the harness's own extension mechanism; no harness-configuration change in the default path (I7). Depends on ASM-07.                                                                                                                                                                          |
| ADR-07 | **SQLite compiled into the core**: through `rusqlite` with SQLite and FTS5 bundled (proposed in [ADR-2610050528](../adr/ADR-2610050528-language-rust-typescript.md); the successor to ADR-2609302341 records the driver); spike S2's FTS5 measurements carry over.                                | Pure-Go drivers ([ADR-2609302341](../adr/ADR-2609302341-sqlite-driver.md), to be superseded) | CON-02: bundled SQLite links into the core executable. In S2 both pure-Go drivers passed the FTS5, cancellation and WAL checks with 50 concurrent connections and were 10–31% apart at 10M events (ncruces slower); modernc links `os/exec` and `net` (SEC-01) and has no encrypting VFS (OQ-04). |
| ADR-08 | **Harness adapter interface** for transcript parsing and hook I/O in the core, and for run control in the launcher.                                                                                                                                                                               | Claude Code specifics throughout                                                             | Contains format churn; enables future harnesses (§1.4).                                                                                                                                                                                                                                           |
| ADR-09 | **Redact before storage.** The record is lossless with respect to post-redaction content.                                                                                                                                                                                                         | Store raw, redact on read                                                                    | A secret that is never written cannot leak from backups, indexes, or recall. Trade-off recorded as OQ-05.                                                                                                                                                                                         |
| ADR-10 | **Deterministic structural landmarks** in v1.                                                                                                                                                                                                                                                     | LLM-written headlines                                                                        | Headlines from untrusted free text would reopen the injection path; determinism supports I10. LLM headlines for trusted spans are P2.                                                                                                                                                             |
