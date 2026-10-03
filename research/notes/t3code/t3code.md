# T3 Code: a local GUI over coding agents

Scope: what T3 Code is and how it compares with Cairn's lane view, as
read from secondary sources on 3 October 2026. The project's own
repository was not read; facts marked "per sources" need checking
against it before they shape a decision.

## What it is

- A lightweight web and desktop GUI for running coding agents, from
  Theo's ping.gg team, launched in March 2026 and moving fast (v0.0.11
  within a week, 1,000+ pull requests) — [Stork](https://www.stork.ai/de/t3-code).
- It wraps the official CLIs and harnesses users already pay for:
  Codex first, with Claude Code, OpenCode, Cursor and Grok support.
  Codex is driven over JSON-RPC on stdio, Claude through the Anthropic
  Agent SDK — [PyShine](https://pyshine.com/T3Code-Minimal-Web-GUI-Coding-Agents/).
- Features: one thread per agent session, per project; plan mode; git
  worktrees for parallel agents in one repository; a live timeline of
  tool calls and file diffs; inline diff review; one-click pull
  request creation; multi-project session management.
- Runs as a local Node.js or Bun server with WebSockets and a React
  front end, started with `npx t3`, or as an Electron app for macOS,
  Windows and Linux.

## How it keeps state

- Event sourced, per sources: clients dispatch commands, commands
  produce immutable domain events in an event store, events update
  read-model projections (threads, messages, activities), and the
  server pushes events to clients. WebSocket methods include
  `orchestration.getSnapshot`, `orchestration.dispatchCommand`,
  `orchestration.getTurnDiff`, `orchestration.getFullThreadDiff` and
  `orchestration.replayEvents` — [architecture](https://www.mintlify.com/pingdotgg/t3code/concepts/architecture).
- Session data, thread history and configuration persist in SQLite, so
  conversations survive restarts; built-in git management with
  checkpoints and diff viewing.
- Not found in the sources: authentication on the local server,
  network exposure, multi-user use, searchable history, signing or
  tamper evidence, redaction, trust labels on recalled content.

## Against Cairn

| Question          | T3 Code                                        | Cairn (2.0-draft)                                                 |
| ----------------- | ---------------------------------------------- | ----------------------------------------------------------------- |
| Unit              | Thread per agent session, per project          | Lane: branch, worktrees, agents, people, edits, results           |
| Where it sits     | A GUI that drives the harnesses                | A record under the harnesses, plus an optional view               |
| State             | Event store and projections in SQLite          | One signed, hash-chained log per writer; rebuildable index        |
| Integrity         | None found                                     | Seals, receipts, `cairn verify`                                   |
| Untrusted content | None found                                     | Pull-only recall in an envelope (I2)                              |
| Constraints       | None found                                     | Pins restored verbatim after compaction (I3)                      |
| Network           | Local server; auth and exposure not documented | Core opens no socket; loopback view with a per-launch token       |
| Multiplayer       | None found                                     | Peer to peer, opt-in, no central service (P2)                     |
| Results           | Diffs and tool-call timeline                   | Evidence classes: claim, own run, witness run, CI attested        |
| Landing           | One-click pull request                         | Derived landed link with proof classes; forge stays authoritative |

## Lessons for Cairn

- The closest product to plan 2610022338's standalone view: several
  harnesses, worktrees, diffs and a timeline in one local GUI. It
  shows the shape users already adopt, and that a fast GUI over
  existing harnesses is enough to win attention.
- It drives the harnesses itself, which is where Cairn's `cairn-run`
  and adapters would sit; Cairn records beneath whatever drives them,
  so the two can coexist: T3 Code as one more harness surface whose
  sessions Cairn records.
- Its event-sourced store and `replayEvents` resemble Cairn's record
  and replay, without the trust, integrity or erasure guarantees.
- Its gaps are Cairn's pitch: no documented guard on the local server,
  no provenance on what reaches the model, no verifiable record.
