# T3 Code: a local GUI over coding agents

Scope: what T3 Code is and how it compares with Cairn, read from its
source at [pingdotgg/t3code](https://github.com/pingdotgg/t3code)
commit 00eb8f6 (MIT, T3 Tools Inc.) on 3 October 2026, after a first
pass over secondary sources. Paths are relative to that repository.

## What it is

- A web, desktop (Electron) and React Native mobile GUI that drives the
  coding agents a user already pays for, from Theo's ping.gg team,
  launched in March 2026 — [Stork](https://www.stork.ai/de/t3-code),
  [PyShine](https://pyshine.com/T3Code-Minimal-Web-GUI-Coding-Agents/).
- Threads per agent session, plan mode, git worktrees for parallel
  agents, a live timeline of tool calls and diffs, checkpoints with
  rollback, stacked commit, push and pull request actions, and merge.
- A local Node.js or Bun server with a WebSocket RPC to the clients.

## Local server and remote access

- Binds `127.0.0.1` by default on port 3773 (`server.ts:248`,
  `config.ts:23`); `--host` or `T3CODE_HOST` opens it, desktop
  "Network access" binds `0.0.0.0`, and the WSL backend always does.
- Every route needs a scoped session: an HMAC-signed `httpOnly`,
  `SameSite=Lax` cookie, a bearer token or a DPoP-bound token, from a
  bootstrap token or a one-time pairing token shown with a QR code.
  WebSocket upgrades take a short-lived ticket, and each RPC passes a
  per-method scope check (`auth/RpcAuthorization.ts`).
- No Origin or Host check on HTTP or WebSocket, no CSRF token, no
  explicit DNS-rebinding defence; packaged builds answer CORS with a
  wildcard origin, without credentials. A page on another localhost
  port is same-site and receives the Lax cookie.
- Remote access is a core feature: LAN pairing, Tailscale Serve, SSH
  with a port forward, and T3 Connect, which uses Clerk identity, a
  Cloudflare relay Worker and managed cloudflared tunnels
  (`docs/user/remote-access.md`, `docs/internals/t3-connect.md`).
- Telemetry goes to PostHog by default, off only with
  `T3CODE_TELEMETRY_ENABLED=false` (`docs/user/telemetry.md`). Other
  hosts: the T3 relay, `clerk.t3.codes`, `app.t3.codes`, OAuth
  providers, forge APIs and update feeds.

## Event store

- SQLite at `~/.t3/userdata/statev2.sqlite`, table
  `orchestration_events`: sequence, event id, aggregate, stream and
  stream version, type, time, command, causation and correlation ids,
  actor kind, payload and metadata JSON. Most payloads are full-state
  snapshots rather than deltas.
- One transaction commits the events, projections, command receipt and
  outbox effects (`orchestration-v2/ProjectionMaintenance.ts`).
  `rebuild` re-applies the log to empty projections; `verify` checks
  schema version, last sequence and thread set at start.
- No hash chain, signature or immutability guard. A compaction path
  deletes superseded snapshot events, called only from tests today.
  Deletion is a soft `thread.deleted` tombstone. No redaction of
  secrets in payloads was found.

## Harness control

- Claude through the Agent SDK's `query()` with a streaming prompt
  queue, no Claude Code hooks: steer by a mid-turn message, interrupt,
  switch permission mode, plan mode with `ExitPlanMode` captured as a
  plan. Approvals arrive through `canUseTool`, become events, and wait
  for the UI's accept, accept-for-session, decline or cancel. New
  threads default to full access (`docs/user/permission-modes.md`).
- Codex through `codex app-server` over JSON-RPC on stdio: start,
  steer, interrupt, fork, compact; per-turn approval and sandbox
  policy; command, file-change, permission and MCP elicitation
  approvals.

## Git

- Checkpoints as hidden refs `refs/t3/checkpoints/<thread>/turn/<n>`,
  built through a temporary index; worktrees under `~/.t3/worktrees`.
- Pull requests through the host CLIs (`gh`, `glab`, `fj`, `tea`,
  `az`) or Bitbucket's API; the GitHub token comes from `gh auth token`,
  pinned to its verified host. Merge and auto-merge via `gh pr merge`.

## Untrusted content and history

- Chat markdown is sanitized (`rehype-sanitize`, DOMPurify for
  Mermaid); agent-produced HTML and SVG are served under a sandbox CSP.
- Prompt-injection handling is ad hoc: attachments are wrapped as
  untrusted data, and handoffs say history is context. No trust mark
  on tool or web output in events.
- The agent can search (plain `LIKE`) and read past threads through
  MCP tools scoped to its project, and the text returns raw, with no
  untrusted envelope. An MCP respond tool can answer questions but,
  by its schema, cannot approve a permission.
- It imports existing Claude Code and other agents' sessions.

## Against Cairn

| Question          | T3 Code                                                     | Cairn (2.0-draft)                                                   |
| ----------------- | ----------------------------------------------------------- | ------------------------------------------------------------------- |
| Where it sits     | A GUI that drives the harnesses                             | A record under the harnesses, with an optional view                 |
| Record            | Event store in SQLite; no chain; snapshots may be compacted | One signed, hash-chained log per writer; nothing deleted silently   |
| Local guard       | Scoped tokens, no Origin or Host check                      | Per-launch token, Host and Origin checks, origin-scoped storage     |
| Network           | Remote access, relay and telemetry built in                 | Core opens no socket; peers opt-in; no central service or telemetry |
| Untrusted content | Sanitized rendering; history recalled raw                   | Pull-only recall in an untrusted envelope (I2)                      |
| Constraints       | None found                                                  | Pins restored verbatim after compaction (I3)                        |
| Erasure           | Soft tombstone; no redaction                                | Redaction before storage; purge that leaves no confirming value     |
| Harness control   | Agent SDK and Codex app-server, rich steering               | Through the harness's input interfaces (OWN-03), in `cairn-run`     |

## Lessons for Cairn

- Worth reusing:
  - committing event, projection and receipt in one transaction;
  - `rebuild` and `verify` from the log alone (I10);
  - short-lived WebSocket tickets, and a scope check on every RPC;
  - pairing that narrows scope and never widens it;
  - pairing secrets in URL fragments;
  - a sandbox CSP on agent-produced HTML;
  - a forge token pinned to its verified host;
  - an MCP tool whose schema cannot approve a permission, which is
    OWN-04's rule expressed in a type.
- Its harness control is the shape Cairn's adapters need (ASM-19): the
  Agent SDK's streaming queue for steer, `canUseTool` for approvals,
  and Codex's app-server requests.
- It and Cairn can coexist: T3 Code drives the harnesses, and Cairn
  records beneath it, if it is installed as one more harness surface.
- Its gaps are Cairn's pitch: no tamper evidence, no Origin or Host
  check on a port it also exposes remotely, telemetry and cloud
  services by default, no redaction, raw history recall, and full
  access as the default permission mode.
