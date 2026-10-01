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
 ┌──────────────────────────── Claude Code / Agent SDK session ────────────────────────────┐
 │                                                                                           │
 │   hooks (command, stdin JSON)                          MCP client (stdio)                 │
 │        │                                                     │                            │
 └────────┼─────────────────────────────────────────────────────┼────────────────────────────┘
          ▼                                                     ▼
   cairn hook <event>                                     cairn mcp
   (short-lived process)                                  (process per session)
          │                                                     │        spawns
          │                                                     ├──────────────► cairn kernel-worker
          ▼                                                     ▼                (rlimits, hermetic,
 ┌────────────────────────────────────────────────────────────────────────┐     read-only store)
 │                         cairn core (Go library)                        │
 │                                                                        │
 │  adapter ─► parse ─► provenance ─► redact ─► flag ─► append ─► index   │
 │  (harness)                                          (seq, hash chain)  │
 │                                                                        │
 │  query compiler · recall · envelope · landmarks · pins · quarantine    │
 │  restore builder (TrustedText only) · audit · counters                 │
 └───────────────┬───────────────────────────────────┬────────────────────┘
                 ▼                                   ▼
   ┌──────────────────────────────┐    ┌──────────────────────────────┐
   │ project store (SQLite, WAL)  │    │ payload store (CAS, sha256)  │
   │ events · events_fts · spans  │    │ large tool outputs, files    │
   │ landmarks · pins · quarantine│    └──────────────────────────────┘
   │ counters · audit · meta      │
   └──────────────────────────────┘
          CAIRN_HOME/<project-id>/            (0700 dirs, 0600 files, tenant-bound)
```

## 4.2 Components

| Component             | Responsibility                                                                                                                                    |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `cairn hook <event>`  | Reads hook input from stdin, performs bounded work within the hook budget, writes hook output to stdout, always exits in a fail-open manner (I9). |
| `cairn mcp`           | Per-session MCP server over stdio exposing recall, landmarks, pins, and kernel tools. Read-mostly.                                                |
| `cairn kernel-worker` | Child process of `cairn mcp` running the hermetic kernel under OS resource limits with a read-only store handle.                                  |
| Admin CLI             | Install, verify, rebuild, quarantine, purge, backup, migrate, stats, audit, doctor.                                                               |
| Core library          | All logic; no global state; every operation takes a `context.Context` with a deadline.                                                            |
| Harness adapter       | Isolates transcript formats and hook I/O for Claude Code (and later other harnesses).                                                             |

## 4.3 Event-sourced state

The record is the only source of truth. Administrative actions — pin creation,
pin removal, quarantine, release, purge tombstones, retention runs — are
themselves appended as events with provenance `operator`. Every other table (FTS
index, spans, landmarks, active pins, quarantine set, counters) is a projection
that `cairn rebuild` reproduces exactly from the record (I10). Purge removes
content but leaves a tombstone event carrying the removed range, reason,
operator, and a hash of the removed content, so rebuilds stay deterministic and
purges stay auditable.

## 4.4 Key scenarios

### Compaction (the core loop)

1. Context approaches its limit; Claude Code fires `PreCompact`.
2. `cairn hook PreCompact` ingests the transcript up to now (bounded), closes
   the current span, updates landmarks, and returns compaction guidance if
   supported (PIN-07).
3. Claude Code compacts. `PostCompact` fires; Cairn records `compact_summary` as
   a `harness` event.
4. Claude Code fires `SessionStart` with `source = compact`. Cairn returns the
   **restore block**: active pins (verbatim), the landmark index, and a one-line
   hint that recall tools exist.
5. Later, Claude needs a detail that compaction dropped. It calls
   `cairn.search`, then `cairn.expand` on the returned `seq` range, and receives
   the exact original inside an untrusted-data envelope.

### Aggregation over large history

1. Claude calls `kernel.exec` with a script such as `hits =
   cairn.search("timeout", kind="tool_result")` followed by a loop that extracts
   and counts error codes.
2. The worker binds results to variables in the session namespace; only the
   script's explicit `print` output (capped) returns to Claude, tainted per its
   sources.

## 4.5 Design decisions

| ADR    | Decision                                                                                                                                                | Alternatives considered                      | Rationale                                                                                                                                                                                                                                                                      |
| ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| ADR-01 | **Daemonless.** Short-lived hook processes and per-session MCP processes open the store directly (SQLite WAL, `BEGIN IMMEDIATE`, busy timeout).         | Long-running daemon on loopback (lcm)        | Removes the port, token, stale-daemon, and wedged-daemon failure classes observed in lcm; zero idle footprint; simpler isolation (I8, I9).                                                                                                                                     |
| ADR-02 | **Event sourcing.** All derived state, including admin actions, is rebuildable from the record.                                                         | Mutable tables                               | Makes verification, rollback, and forensic replay possible (I5, I10).                                                                                                                                                                                                          |
| ADR-03 | **Pull-only recall.** Automatic injection accepts only the `TrustedText` type, which can be constructed only from pins and sanitized structural fields. | Per-prompt memory hints (lcm, claude-mem)    | Poisoning evidence (§3, item 3). Enforced by the type system, not convention (I2).                                                                                                                                                                                             |
| ADR-04 | **Hermetic Starlark kernel by default**; external Python kernel as optional P2.                                                                         | Python/Jupyter kernel as default             | Starlark (Python dialect, pure Go) has no filesystem or network unless the host provides it, is deterministic, supports step limits, and needs no Python runtime (CON-03). Risk: less capable than Python for Claude. Gate: spike S5 measures Claude's task success with each. |
| ADR-05 | **BM25 over SQLite FTS5; no embeddings in v1.**                                                                                                         | Vector search, hybrid                        | Deterministic, no model calls at ingest, no extra data leaving the machine; Scroll reached strong results with BM25.                                                                                                                                                           |
| ADR-06 | **Distribution as a Claude Code plugin** (hooks + MCP registration) plus the static binary. `cairn install` edits settings only as a fallback.          | Tool rewrites `settings.json` (lcm)          | Uses the harness's own extension mechanism; no configuration mutation in the default path (I7). Depends on ASM-07.                                                                                                                                                             |
| ADR-07 | **Pure-Go SQLite driver**: spike S2 selects `github.com/ncruces/go-sqlite3` ([ADR-2609302341](../adr/ADR-2609302341-sqlite-driver.md)).                 | `modernc.org/sqlite`; CGO `mattn/go-sqlite3` | CON-02. Both pure-Go drivers passed the FTS5, cancellation and 50-writer WAL checks and scored within 10–25% of each other at 10M events; modernc links `os/exec` and `net` (SEC-01) and has no encrypting VFS (OQ-04).                                                        |
| ADR-08 | **Harness adapter interface** for transcript parsing and hook I/O.                                                                                      | Claude Code specifics throughout             | Contains format churn; enables future harnesses (NG6).                                                                                                                                                                                                                         |
| ADR-09 | **Redact before storage.** The record is lossless with respect to post-redaction content.                                                               | Store raw, redact on read                    | A secret that is never written cannot leak from backups, indexes, or recall. Trade-off recorded as OQ-05.                                                                                                                                                                      |
| ADR-10 | **Deterministic structural landmarks** in v1.                                                                                                           | LLM-written headlines                        | Headlines from untrusted free text would reopen the injection path; determinism supports I10. LLM headlines for trusted spans are P2.                                                                                                                                          |
