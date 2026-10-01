---
title: "1. Introduction"
summary: >-
  Purpose, intent, the ten invariants (I1–I10) every requirement
  serves, scope by context layer, non-goals, and the glossary.
---
# 1. Introduction

## 1.1 Purpose

This document specifies Cairn, a context layer that lets Claude agents work over
unbounded histories — long research sessions, long-running conversations, and
remote agents running for hours or days — without losing information to
compaction and without turning stored history into a prompt-injection channel.

## 1.2 Intent

Cairn gives long-running Claude agents **unbounded, exact recall without
creating a new attack surface**.

Existing tools treat memory as a convenience feature and add security
afterwards. Cairn treats the memory store as security-critical infrastructure
from the first line of code. Where usefulness and safety conflict, Cairn chooses
the design that keeps the agent safe and makes the convenience opt-in, never the
reverse.

Retrieval is the purpose. The record, provenance, pins, landmarks and the
kernel all exist so that Claude finds the right history at the right moment. A
record Claude cannot find its way back into is worth nothing, and context
delivered when it is not needed costs attention, tokens and cache. So Cairn
models retrieval need explicitly (§5.11). It offers pointers at the moments a
need is likely, holds them to measured precision (§11.1), and stays silent
otherwise.

## 1.3 Invariants

The invariants are Cairn's contract. Every requirement serves at least one of
them. A change that would break an invariant is a design change requiring
security review and a new major version, not a bug fix.

| #       | Invariant                                                                | Meaning                                                                                                                                                                                                                                                                                     |
| ------- | ------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **I1**  | **Nothing is lost**                                                      | Every event an agent saw or produced remains recoverable by a stable address across any number of compactions and sessions. The only exceptions are secrets removed by redaction before storage, and data an operator explicitly purges or expires by policy. Both exceptions are recorded. |
| **I2**  | **No automatic path from untrusted content to the model**                | Content that originates outside the trusted boundary (tool output, web, MCP servers, files, assistant text) reaches the model only when Claude explicitly calls a recall tool, and always inside an untrusted-data envelope.                                                                |
| **I3**  | **Constraints are never summarized**                                     | Pinned constraints are stored verbatim and re-injected verbatim after every compaction.                                                                                                                                                                                                     |
| **I4**  | **Cairn never talks to the network**                                     | No listening sockets, no outbound connections, no telemetry. Data leaves the machine only when Claude receives recalled content through a tool call, and then travels to the model provider like any other context.                                                                         |
| **I5**  | **Bad data can be removed from circulation without destroying evidence** | Any event, span, session, or derived artifact can be quarantined from recall immediately, while the record stays intact for forensics.                                                                                                                                                      |
| **I6**  | **No silent failures**                                                   | Every dropped, rejected, redacted, timed-out, or failed operation is counted, logged, and visible to operators.                                                                                                                                                                             |
| **I7**  | **Configuration changes only on explicit instruction**                   | Cairn changes agent configuration only through explicit install and uninstall operations, shows the change first, and never overrides managed policy.                                                                                                                                       |
| **I8**  | **Isolation follows the tenant**                                         | All state is bound to one tenant's home with strict permissions. Cairn refuses to operate on state it does not own.                                                                                                                                                                         |
| **I9**  | **Cairn never degrades the agent**                                       | A Cairn failure never blocks or slows the agent beyond defined budgets. Cairn fails open, except where continuing would violate I2, I4, or I8.                                                                                                                                              |
| **I10** | **Everything derived is rebuildable**                                    | All derived state (indexes, landmarks, active pins, quarantine set, statistics) is a deterministic function of the append-only record. Rebuilding from the record reproduces it exactly.                                                                                                    |

## 1.4 Scope

Cairn implements layers 1–4 of the agent context architecture:

| Layer | Name           | In Cairn v1                                                            |
| ----- | -------------- | ---------------------------------------------------------------------- |
| 1     | Record         | Append-only, provenance-tagged event log of every session              |
| 2     | Pinned context | Constraints that survive every compaction verbatim                     |
| 3     | Working view   | Pull-only recall, recall cues, landmark index, post-compaction restore |
| 4     | Compute        | Hermetic kernel with read-only access to the record                    |
| 5     | Durable memory | **Not in scope.** Export interface only (§5.9)                         |

## 1.5 Non-goals for v1

| #   | Non-goal                                                           | Reason                                                                                                            |
| --- | ------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------- |
| NG1 | Replacing or disabling Claude Code's native compaction             | Behaviour of the harness is outside our control; Cairn makes compaction recoverable instead. Revisited via OQ-01. |
| NG2 | Long-term semantic memory, fact extraction, or automatic promotion | Largest poisoning surface; belongs to a separate layer-5 system with its own policy                               |
| NG3 | Embedding or vector search                                         | BM25 is deterministic, needs no model calls at ingest, and is sufficient for v1 (Scroll uses BM25)                |
| NG4 | Multi-host shared stores                                           | Multiplies blast radius; requires provenance-gated sharing design first (OQ-07)                                   |
| NG5 | Graphical UI                                                       | CLI and MCP only                                                                                                  |
| NG6 | Harnesses other than Claude Code and the Claude Agent SDK          | Design MUST NOT preclude them (ADR-08)                                                                            |
| NG7 | Summarization in the core                                          | Summaries are lossy, cost model calls, and are a poisoning channel; optional P2 feature only                      |

## 1.6 Glossary

| Term              | Definition                                                                                                                         |
| ----------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| **Tenant**        | The security principal that owns a Cairn home: one OS user, optionally bound to a tenant ID supplied by the runner environment.    |
| **Home**          | Directory holding all of a tenant's Cairn state (`CAIRN_HOME`, default `~/.cairn`).                                                |
| **Project**       | A working directory as identified by Claude Code (one transcript directory). Each project has its own store.                       |
| **Session**       | One Claude Code or Agent SDK session, identified by `session_id`. A subagent run is a child session.                               |
| **Source**        | A transcript file that Cairn ingests.                                                                                              |
| **Record**        | The append-only log of events for a project. The source of truth.                                                                  |
| **Event**         | One immutable entry in the record: a message, tool call, tool result, hook observation, or administrative action.                  |
| **seq**           | An event's address: a strictly increasing integer, unique within a project, never reused.                                          |
| **Payload**       | The full content of a large event, stored content-addressed outside the event row.                                                 |
| **Provenance**    | Where an event's content came from (§5.2).                                                                                         |
| **Trust level**   | `trusted` or `untrusted`, derived from provenance by the trust policy.                                                             |
| **Taint**         | The trust level inherited by a derived artifact: untrusted if any source event is untrusted.                                       |
| **Span**          | A contiguous range of events within a session, bounded by user turns, compactions, or subagent boundaries.                         |
| **Landmark**      | A structural headline describing a span, bound to its exact `seq` range.                                                           |
| **Pin**           | A constraint the tenant has explicitly marked as always-present, stored verbatim.                                                  |
| **Quarantine**    | Exclusion of events or artifacts from recall and injection, recorded as an event, without deletion.                                |
| **Envelope**      | The structured wrapper in which recalled content is returned to Claude, marking it as untrusted historical data.                   |
| **Restore block** | The deterministic text Cairn injects after compaction: pins, landmark index, and a recall hint.                                    |
| **In view**       | An event Claude can still see: the current session after its latest compaction boundary (§5.11). Every other event is out of view. |
| **Moment**        | A hook event at which Cairn can observe a sign that Claude needs out-of-view history, and still deliver a cue (§5.11).             |
| **Cue**           | A short `TrustedText` pointer to out-of-view events, delivered at a moment; Claude pulls the content itself (§5.11).               |
| **Working view**  | Whatever is currently in the model's context window. A projection; never the source of truth.                                      |
| **Kernel**        | The layer-4 compute environment in which Claude runs code over the record.                                                         |
