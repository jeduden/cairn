# Trace forward: Cairn's pitch to the SRS

This file traces the pitch of 3 October 2026 and the stakeholder's ten
decisions in [pitch.md](pitch.md) forward into the SRS
([index](../../docs/srs/index.md), version 1.4-draft). The plans behind
the pitch fill in detail: [plan 2610012322](plan.md),
[plan 2610022338](../2610022338_cairn-network-side/plan.md) and its
[phase 1](../2610022338_cairn-network-side/phase-1.md). It is evidence
for task 2 of this plan, the SRS change. Nothing here is normative.

Legend:

- **Status.** *covered*: a requirement makes the claim testable.
  *partial*: some of it is testable; the gap is named. *contradicted*: an
  existing id, invariant or non-goal says otherwise. *missing*: no
  requirement exists.
- **Action.** (a) new requirement, (b) change existing text, (c) change
  the pitch. Each claim that is not covered gets exactly one action.
  Knock-on edits that the action forces are listed with it.
- **Post-v1** marks a claim whose answer belongs to a later milestone in
  §12, not to v1.0.

Ids follow the families on disk, next free number. Not used, because
other open pull requests hold them: ASM-10..16, PRV-08, OQ-10..13,
T10..13 and the CUE family. New ids proposed here: REC-17, REC-18,
RCL-08, ADM-13, SEC-19..23, LANE-01..06, UI-01..04, PEER-01..04,
ASM-17, OQ-14, T14..T18, spike S9 and milestones M7..M9. Every new or
changed id lands with its `@pending` scenario in the same change
(CLAUDE.md, "Requirements and Scenarios").

## Counts

| Status       | Claims |
| ------------ | ------ |
| covered      | 12     |
| partial      | 9      |
| contradicted | 8      |
| missing      | 22     |
| total        | 51     |

## Claim table

Sources: **pitch** is the quoted pitch, **lane** the paragraph after it,
**D1**–**D10** the decisions, **table** the Delta comparison, **open**
the list "Open before the SRS change", and **2610012322** or
**2610022338** the plan text.

| #   | Claim                                                                                                    | Source         | Status       | Supporting ids (short quote)                                                                                                                                                                                                                                                                        | Action                                                         |
| --- | -------------------------------------------------------------------------------------------------------- | -------------- | ------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| P1  | The record holds every message                                                                           | pitch          | covered      | REC-01 "MUST ingest Claude Code session transcripts"; PRV-01 "`user`, `assistant`"; REC-05 "MUST NOT be dropped"; REC-02 subagents                                                                                                                                                                  | —                                                              |
| P2  | It holds every edit an agent makes through its tools                                                     | pitch          | covered      | PRV-01 "`tool_call`"; REC-09 large inputs as payloads; RCL-03 "MUST return exact post-redaction content". Kept as raw tool input, not as a diff                                                                                                                                                     | —                                                              |
| P3  | It holds edits made outside the agent's tools (a person, a formatter, a shell)                           | pitch          | missing      | None. Transcripts carry only what the harness logged; no requirement reads the worktree                                                                                                                                                                                                             | (a) REC-17                                                     |
| P4  | It holds every tool run                                                                                  | pitch          | covered      | PRV-01 "`tool_call`"; REC-01; REC-02 "link each to its parent session"                                                                                                                                                                                                                              | —                                                              |
| P5  | It holds every result                                                                                    | pitch          | covered      | PRV-01 "`tool_result:<tool>`"; REC-09; RCL-03                                                                                                                                                                                                                                                       | —                                                              |
| P6  | The record is tamper-evident                                                                             | pitch          | partial      | REC-10 "Events MUST form a per-project hash chain"; ADM-09 "hash-chain integrity"; OPS-02 "with its own hash chain". Gap: the chain is unkeyed, so whoever can write the store can rebuild it; nothing anchors or signs a head; SEC-10 "The v1 core MUST handle no credentials." bars a signing key | (a) REC-18                                                     |
| P7  | The record stays on your own machines                                                                    | pitch          | covered      | I4 "No listening sockets, no outbound connections, no telemetry."; SEC-01; SEC-15; OPS-05 "Cairn itself MUST NOT ship logs anywhere."                                                                                                                                                               | —                                                              |
| P8  | One record spans agents on several machines                                                              | pitch          | contradicted | NG4 "Multi-host shared stores", "requires provenance-gated sharing design first (OQ-07)"; RCL-05 "Cross-project recall MUST NOT be possible in v1."                                                                                                                                                 | (b) NG4. Post-v1                                               |
| P9  | Agents working in parallel are all recorded                                                              | pitch          | covered      | NFR-05 "≥ 50 concurrent sessions and subagents writing"; NFR-08 "Concurrent writers MUST never corrupt the store"; ENG-09 soak test                                                                                                                                                                 | —                                                              |
| P10 | There is a view (a graphical interface)                                                                  | pitch, D8      | contradicted | NG5 "Graphical UI", "CLI and MCP only"                                                                                                                                                                                                                                                              | (b) NG5                                                        |
| P11 | The view shows each agent live, as it works                                                              | pitch, D8      | missing      | None. REC-13 "SHOULD ingest the active transcript incrementally" bounds ingestion, not display                                                                                                                                                                                                      | (a) UI-02                                                      |
| P12 | The view shows what each agent produced                                                                  | pitch, D8      | partial      | LMK-02 "tool names with call counts, touched file paths, and an error indicator"; PRV-01 tool results. Gap: no per-agent results projection (files changed, commands run, outcomes)                                                                                                                 | (a) LANE-03                                                    |
| P13 | The view shows what actually verified each result                                                        | pitch          | missing      | None. LMK-02's error indicator is not verification                                                                                                                                                                                                                                                  | (a) LANE-04                                                    |
| P14 | Work with your agents live: steer them from the view                                                     | pitch headline | missing      | None. Steering through Cairn's injection path would contradict INJ-03 "MUST accept only values of type `TrustedText`" and INJ-04                                                                                                                                                                    | (a) UI-04                                                      |
| P15 | It runs standalone, with no service behind it                                                            | pitch, D3      | covered      | CON-03 "No runtime dependency on Python, Node.js, or a database server for any P0 feature."; NFR-12 "with zero required configuration"                                                                                                                                                              | —                                                              |
| P16 | Standalone needs no network                                                                              | pitch, D4      | covered      | SEC-01 "Shipped binaries MUST NOT open listening sockets or make outbound network connections."; ENG-12 "network-denied sandbox"; CON-04                                                                                                                                                            | —                                                              |
| P17 | Peer to peer can be turned on                                                                            | pitch, D4      | contradicted | I4 "No listening sockets, no outbound connections"; SEC-01; CON-04 "No network access, at build-verified level, in any binary that ships P0 features"; NG4                                                                                                                                          | (b) I4 — **invariant change**. Post-v1                         |
| P18 | Peering is built on standalone and off by default                                                        | D4, 2610022338 | missing      | None                                                                                                                                                                                                                                                                                                | (a) PEER-01. Post-v1                                           |
| P19 | Work a lane live with others                                                                             | pitch          | missing      | None                                                                                                                                                                                                                                                                                                | (a) PEER-04. Post-v1                                           |
| P20 | No central service: every node is a full peer, and self-hosting is the normal case                       | pitch, D3      | missing      | None for the network side (standalone is P15)                                                                                                                                                                                                                                                       | (a) PEER-02. Post-v1                                           |
| P21 | No break when the network splits: each side keeps writing                                                | pitch, D5      | contradicted | REC-06 "strictly increasing, gap-free, and never reused within its project"; REC-10 "per-project hash chain"; §8.2 "`seq` PRIMARY KEY, strictly increasing, gap-free". Two cut-off writers cannot extend one gap-free chain                                                                         | (b) REC-06, REC-10                                             |
| P22 | The sides merge on reconnect and converge                                                                | D5, table      | missing      | None                                                                                                                                                                                                                                                                                                | (a) PEER-03. Post-v1                                           |
| P23 | Derived lane state is the same whatever order logs arrive in                                             | D5, 2610022338 | partial      | I10 "a deterministic function of the append-only record"; ADM-08 "byte-identical to the state before rebuild". Gap: one record in one order is assumed; nothing requires order independence across logs                                                                                             | (b) ADM-08 and glossary "Record" — **touches invariant scope** |
| P24 | CRDTs merge state that several participants edit at once (lane metadata)                                 | D5             | missing      | None                                                                                                                                                                                                                                                                                                | (a) LANE-05. Post-v1                                           |
| P25 | Live co-editing of files in one worktree                                                                 | table          | missing      | None; the table marks it "Planned"                                                                                                                                                                                                                                                                  | (b) §12 milestone M9. Post-v1                                  |
| P26 | Only you can instruct your agents                                                                        | pitch, table   | partial      | PIN-01 "Active pins MUST be created only by trusted actions"; INJ-03; PRV-04 "in which `user` is untrusted". Gap: Cairn controls only its own channels; the harness reads CLAUDE.md, files, web pages and tool output written by others directly                                                    | (c) pitch                                                      |
| P27 | Everyone else's words reach your agents only as untrusted data                                           | pitch, table   | partial      | I2 "reaches the model only when Claude explicitly calls a recall tool"; RCL-04 "All recalled content MUST be returned inside the envelope". Gap: PRV-02 trusts by class, "and `user` when the deployment mode is `interactive`", so a co-author's prompt imported from a peer would be trusted      | (b) PRV-02                                                     |
| P28 | The lane is the unit Cairn holds                                                                         | lane, D2       | missing      | None. The record is per project (§1.4 "event log of every session")                                                                                                                                                                                                                                 | (a) LANE-01                                                    |
| P29 | A lane spans a branch and all its worktrees                                                              | lane, D2       | contradicted | §8.1 "project-id = hex(SHA-256(canonical project path))[:32]"; glossary Project "A working directory as identified by Claude Code"; RCL-05 forbids recall across the resulting projects                                                                                                             | (b) §8.1 and glossary "Project"                                |
| P30 | A lane holds its agents and humans, and who did what                                                     | lane           | partial      | REC-02 "recording agent identity when the transcript provides it"; PRV-01 `user`. Gap: no human identity, so two humans in a lane are one class                                                                                                                                                     | (a) LANE-02                                                    |
| P31 | A lane is a pull request in all but its interface                                                        | lane, D2       | missing      | None: no review, approval, required check or landing step                                                                                                                                                                                                                                           | (c) pitch                                                      |
| P32 | Signed approvals and required checks gate the lane                                                       | table          | missing      | None                                                                                                                                                                                                                                                                                                | (a) LANE-06. Post-v1                                           |
| P33 | Git keeps the code                                                                                       | lane           | partial      | ADM-02 "MUST display a diff of every configuration change"; SEC-18 paths for reading. Gap: nothing forbids Cairn writing to a repository                                                                                                                                                            | (a) ADM-13                                                     |
| P34 | Keep the name Cairn                                                                                      | D1             | covered      | SRS title page, "Cairn — lossless, security-first context layer". Not requirement-bearing                                                                                                                                                                                                           | —                                                              |
| P35 | The experience comes first; seamless sync is the second step                                             | D6             | missing      | §12 has no lane, view or peer milestone                                                                                                                                                                                                                                                             | (b) §12                                                        |
| P36 | Security is good enough, measured against Zed Delta                                                      | D7             | missing      | SEC-17 "A threat-model document MUST be maintained in the repository and reviewed at every minor release." The actors in §6.1 have no co-author, peer, network attacker or browser page                                                                                                             | (b) SEC-17 and §6.1                                            |
| P37 | One view shows the other harnesses on the machine beside the one being worked with                       | D8             | missing      | None                                                                                                                                                                                                                                                                                                | (a) UI-01                                                      |
| P38 | Harnesses connect directly and reach each other's lane work with no UI between                           | D8             | partial      | RCL-05 "Widening from the current session to all sessions of the project MUST require an explicit parameter"; RCL-01. Gap: worktrees are separate projects (P29), and no lane scope exists                                                                                                          | (a) RCL-08                                                     |
| P39 | The UI is one more client, never a required hop                                                          | D8             | missing      | None                                                                                                                                                                                                                                                                                                | (a) UI-03                                                      |
| P40 | The standalone UI is a browser page on a loopback-only port                                              | D9             | contradicted | SEC-01 "MUST NOT open listening sockets"; NFR-09 "No resident process between sessions."; ADR-01 (non-normative) "Removes the port, token, stale-daemon"                                                                                                                                            | (b) SEC-01                                                     |
| P41 | The UI guards against local attacks: token per launch, Host and Origin checks, no cross-origin           | 2610022338     | missing      | None                                                                                                                                                                                                                                                                                                | (a) SEC-20                                                     |
| P42 | The view shows others' words and tool output without them acting in the browser                          | pitch × D8     | missing      | SEC-06 "so no stored content can alter the envelope's structure" covers the envelope only, not rendering                                                                                                                                                                                            | (a) SEC-21                                                     |
| P43 | Standalone means nothing leaves the machine                                                              | D9             | partial      | SEC-15; I4 "Data leaves the machine only when Claude receives recalled content through a tool call". Gap: by I4's own carve-out, recalled content does leave, to the model provider                                                                                                                 | (c) pitch                                                      |
| P44 | Network boundaries are explicit (process, machine, peer, public), and a wider one is never on by default | D10            | missing      | None. SEC-01 is a single ban with no boundary model                                                                                                                                                                                                                                                 | (a) SEC-19                                                     |
| P45 | B0: the core that feeds the model has no network capability                                              | D10            | covered      | SEC-01; ENG-16 "forbidden imports (`net`, `net/http`, `os/exec` outside allow-listed packages)"; ENG-12                                                                                                                                                                                             | —                                                              |
| P46 | B2: only signed segments from peers enrolled by key, imported as untrusted                               | D10            | missing      | None                                                                                                                                                                                                                                                                                                | (a) SEC-22. Post-v1                                            |
| P47 | B3: public, read-only, reviewed and signed lane bundles                                                  | D10            | missing      | None. ADM-12 export is local and trusted-only                                                                                                                                                                                                                                                       | (a) SEC-23. Post-v1                                            |
| P48 | No telemetry and no third-party service at any boundary                                                  | D10            | covered      | SEC-15 "MUST NOT include telemetry, crash reporting to remote services, or update checks"; OPS-05                                                                                                                                                                                                   | —                                                              |
| P49 | Erasure works: purged content cannot be confirmed afterwards (keyed hashes)                              | table, open    | contradicted | REC-09 "keyed by SHA-256"; §8.2 tombstones "hash of removed content". An unkeyed hash lets anyone confirm a guessed purged text                                                                                                                                                                     | (b) REC-09 and §8.2                                            |
| P50 | Claude Code first; other harnesses later                                                                 | table          | covered      | NG6 "Design MUST NOT preclude them (ADR-08)"; NFR-13 "MUST require changes only in the harness adapter package"                                                                                                                                                                                     | —                                                              |
| P51 | Agents in reclaimed cloud sandboxes keep their record                                                    | 2610012322     | contradicted | §2.2 assumes `CAIRN_HOME` and the rest "live on durable volumes so sessions can pause and resume"                                                                                                                                                                                                   | (b) §2.2. Post-v1                                              |

Not traced: the lesson from OpenAI dots on rule levels per agent action
(act, act when told, ask first, hand off). The pitch states it as a
lesson, not a claim, and plan 2610012322 task 1 weighs it. SEC-13
already ships an example `PreToolUse` approval hook.

## Proposed requirement texts

### New families and why

- **LANE** (§5.11, Lane). The lane is a unit above the session and the
  project. No existing family owns its identity, its actors, its results
  or its merge gate. REC is about ingestion, and RCL about retrieval.
- **UI** (§5.12, User interface). NG5 kept a graphical client out, so no
  family covers one. Keeping it separate lets §12 schedule the view as
  its own milestone and keeps its security in §6.
- **PEER** (§5.13, Peer network). Replication between nodes is new
  behavior with its own priority (P2) and milestone. Its security
  controls go to SEC (SEC-22), its behavior here.
- No **NET** family: the network boundaries are security controls, so
  they extend SEC (SEC-19..23).

Priorities: the standalone lane and view are P1 because D6 puts the
experience first and plan 2610022338's phase 1 builds it before
peering. Everything that needs a network beyond loopback is P2. If the
stakeholder wants the view after v1.0, the UI and LANE P1 rows become
P2 with no other change.

### §5.1 Record (REC)

| ID     | Pri | Requirement                                                                                                                                                                                                                                                                                                                                                                                                                                                      | Ver | Traces |
| ------ | --- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --- | ------ |
| REC-17 | P2  | On `Stop` and `SessionEnd`, Cairn SHOULD record a worktree checkpoint as an event with provenance `file`: the checked-out commit id, the branch, and a redacted diff (SEC-08) of tracked and non-ignored untracked files against the previous checkpoint, stored as a payload. Edits made outside the agent's tool calls thereby enter the record. Reading the worktree MUST stay within SEC-18 and the hook budget, leaving the rest to a work marker (NFR-02). | T   | I1     |
| REC-18 | P1  | Each origin MUST sign the head of its hash chain (REC-10) with a per-origin Ed25519 key whenever it closes a segment, and `cairn verify` MUST check every signature against the origin's public key, exiting 3 on any mismatch. The private key MUST be generated on the node, held under SEC-10's secret-reference rules, and MUST NOT enter the record, an export, a backup or a segment.                                                                      | T   | I6     |

REC-17 needs a pure-Go reader for the git index and objects, since
`os/exec` is banned. That is either own code or a new direct dependency
with its ADR (ENG-18). REC-18 forces a knock-on to SEC-10: "The v1 core
MUST handle no credentials" becomes "The core MUST handle no
credential other than its own origin signing key (REC-18)". On one
node, an attacker running as the tenant's UID can still reach the key.
That actor is out of scope in §6.1. REC-18 makes the record
tamper-evident once it leaves the node: in a backup, an export or a
peer's copy.

### §5.4 Recall (RCL)

| ID     | Pri | Requirement                                                                                                                                                                                                                                                                                                  | Ver | Traces |
| ------ | --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --- | ------ |
| RCL-08 | P1  | `search`, `expand` and `landmarks` MUST accept the scope `session = lane`, covering every session of the current lane (LANE-01), including sessions in other worktrees of the project. The scope MUST be explicit, MUST be logged as the all-sessions scope is (RCL-05), and MUST NOT reach another project. | T   | I8     |

Knock-on: §9.2 adds `lane` to the `session` parameter. In §9.6,
`recall.default_session_scope` stays `current`.

### §5.8 Administration and lifecycle (ADM)

| ID     | Pri | Requirement                                                                                                                                                                                                                                                                                             | Ver | Traces |
| ------ | --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --- | ------ |
| ADM-13 | P0  | Cairn MUST NOT write to any git repository or working tree: objects, refs, notes, configuration, hooks or files. The only exception is the project-scope settings file that `cairn install --scope project` writes after confirmation (ADM-02). Cairn MAY read repositories within SEC-18's path rules. | T   | I7     |

Plan 2610022338 task 7 (git as a carrier) would later amend ADM-13 with
an explicit, opt-in export step.

### §5.11 Lane (LANE)

| ID      | Pri | Requirement                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       | Ver  | Traces      |
| ------- | --- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---- | ----------- |
| LANE-01 | P1  | Cairn MUST hold each lane as one record: a lane is identified by its project and a lane id, and created by an `operator` event or on first sight of a branch. Every session MUST belong to exactly one lane, chosen by a deterministic rule over the record: the lane of the branch recorded as a `harness_meta` event at the session's first ingested event, or the project's default lane. A reassignment MUST be recorded as an `operator` event. Branch names MUST be sanitized as LMK-03 requires before any structural use. | T    | I1, I2, I10 |
| LANE-02 | P1  | Every event MUST name its actor: an agent session, with subagent identity as REC-02 records it, or a human. A human is the local tenant for local events and the enrolled key's owner for imported ones. Cairn MUST derive the actor from origin and source only, never from content.                                                                                                                                                                                                                                             | T    | I2          |
| LANE-03 | P1  | Cairn MUST derive a results projection per session and per lane: each file changed (from tool calls and from REC-17 checkpoints when enabled), and each command run with its exit status. Each result MUST be bound to the `seq` range that produced it and built from structural fields only (LMK-03). The projection MUST be a deterministic function of the record.                                                                                                                                                            | T    | I2, I10     |
| LANE-04 | P1  | Each result in LANE-03 MUST carry one verification level from the closed set `none`, `claimed` (asserted only in assistant text), `local_run` (a command recorded in the lane with its exit status) and `external` (a check off the machine, known only from tool output). The level MUST be derived deterministically from the record, a level that rests on untrusted events MUST be marked untrusted, and Cairn MUST NOT fetch external check results itself.                                                                  | T    | I2, I4, I10 |
| LANE-05 | P2  | Lane metadata that several participants may change (title, status, labels, assignment) MUST be stored as events and merged by a documented, deterministic, conflict-free rule, a CRDT chosen by ADR, so that every node holding the same events derives the same metadata.                                                                                                                                                                                                                                                        | T    | I10         |
| LANE-06 | P2  | A lane MAY declare required checks and approvers. An approval MUST be an event signed by an enrolled approver's key over the lane heads it approves. Cairn MUST report a lane as landable only when every required check is at `local_run` or stronger (LANE-04) on those heads and every required approval is present. It MUST NOT push or merge in git itself (ADM-13).                                                                                                                                                         | T, D | I2, I10     |

LANE-04 is honest about I4: the core can never know canonical CI
results first-hand, only through tool output, which is untrusted.

### §5.12 User interface (UI)

| ID    | Pri | Requirement                                                                                                                                                                                                                                                                                                                                                                                                    | Ver  | Traces  |
| ----- | --- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---- | ------- |
| UI-01 | P1  | `cairn-ui` MUST show, in one view, every live and recorded session of the local tenant's projects, grouped by lane. The selected session's timeline MUST show prompts, tool calls, permission requests and output, each edit as a diff, each tool run with its result and verification level (LANE-04), and the actor of each event (LANE-02). Every other session MUST appear as a tile that opens full size. | D, T | I1      |
| UI-02 | P1  | `cairn-ui` MUST show each transcript line of a running session within 2 s† of the harness writing it (ASM-17). Reading by `cairn-ui` MUST NOT push any hook beyond its NFR-01 budget.                                                                                                                                                                                                                          | A    | I9      |
| UI-03 | P1  | `cairn-ui` MUST be an optional client. It MUST read the store only through the core library, read-only, and MUST hold no state that cannot be rebuilt from the record. Every capability it offers MUST also be available through the CLI or MCP, and no hook, ingestion or recall path MAY depend on `cairn-ui` running.                                                                                       | T    | I9, I10 |
| UI-04 | P2  | `cairn-ui` MAY send a prompt or a permission decision to a running local harness only on the local tenant's action in an authenticated UI session (SEC-20), only through that harness's own client interface and never through Cairn's injection path (INJ-03, INJ-04). It MUST NOT forward content received from a peer. Each such action MUST be recorded with provenance `user` and audited.                | T    | I2, I6  |

### §5.13 Peer network (PEER)

| ID      | Pri | Requirement                                                                                                                                                                                                                                                                                                                                | Ver  | Traces  |
| ------- | --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---- | ------- |
| PEER-01 | P2  | Peering MUST ship as a separate binary, `cairn-peer` (boundary B2), that is off by default, starts only on an explicit tenant action, and that the core neither links nor starts. With `cairn-peer` absent or stopped, every requirement in §5–§10 MUST hold exactly as in standalone.                                                     | T    | I4, I9  |
| PEER-02 | P2  | Every peer MUST be able to hold complete copies of the lanes it shares and serve them to any other enrolled peer. No peer MAY be required for two others to exchange segments, and no third-party service MAY be required for enrollment, discovery or relay.                                                                              | T, D | I4      |
| PEER-03 | P2  | Peers MUST exchange whole, signed origin segments (REC-18). Any two peers holding the same set of segments MUST derive byte-identical lane state and index, whatever order or partition the segments arrived through (ADM-08). A writer cut off from every peer MUST keep appending to its own log without blocking or slowing its agents. | T    | I9, I10 |
| PEER-04 | P2  | While connected, enrolled peers MUST receive a lane's newly closed segments within 5 s†. Every imported event MUST keep its origin, MUST be untrusted on the importing node (PRV-02), and MUST reach an agent only through pull-only recall.                                                                                               | A, T | I2      |

### §6.2 Security requirements (SEC)

| ID     | Pri | Requirement                                                                                                                                                                                                                                                                                                                                                                                                                     | Ver  | Traces     |
| ------ | --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---- | ---------- |
| SEC-19 | P0  | Each Cairn binary MUST be assigned to exactly one network boundary in a register kept in the repository: B0 process (the core; no socket), B1 machine (loopback only), B2 peer (enrolled peers only) or B3 public (read-only publishing). B2 and B3 MUST be off by default. A CI check MUST fail when a binary's import closure, or a test run under its boundary's network sandbox, shows more reach than the register grants. | T, I | I4         |
| SEC-20 | P1  | `cairn-ui` MUST bind only to a loopback address on an ephemeral port. It MUST require on every request a random token (at least 128 bits) minted per launch and shown only to the launching user. It MUST reject a request whose `Host` is not its loopback address and port, or whose `Origin` is not its own, MUST allow no cross-origin request, and MUST audit every rejection.                                             | T    | I4, I6, I8 |
| SEC-21 | P1  | `cairn-ui` MUST render all record content as inert text: it MUST NOT interpret HTML, script, or Markdown links and images taken from record content. A Content-Security-Policy MUST forbid loading any resource from outside its own origin, so that displayed content can neither run code nor make the browser reach the network. Untrusted content MUST be visibly marked.                                                   | T    | I2, I4     |
| SEC-22 | P2  | `cairn-peer` MUST accept segments only from peers the tenant enrolled by public key. It MUST verify each segment's signature (REC-18) and chain continuity before import, MUST refuse and audit a segment that fails either check, and MUST store imported events under the local tenant's home (SEC-02) as untrusted (PRV-02).                                                                                                 | T    | I2, I6, I8 |
| SEC-23 | P2  | Publishing a lane (B3) MUST require the tenant's explicit review of each bundle, MUST apply redaction at least as strict as SEC-08 plus a publish-only rule set, and MUST sign the bundle. The public host MUST serve bundles read-only, with no write path. An imported public bundle MUST be untrusted.                                                                                                                       | T, I | I2, I4     |

### Supporting register entries

§2.3 assumptions register:

| ID     | Assumption                                                                            | Evidence so far | Verified in | Affects |
| ------ | ------------------------------------------------------------------------------------- | --------------- | ----------- | ------- |
| ASM-17 | Claude Code appends each transcript line within 1 s of the event while a session runs | Unverified      | S9          | UI-02   |

§12.1 spike:

| Spike | Question                                                                                              | Resolves | Exit criterion                 |
| ----- | ----------------------------------------------------------------------------------------------------- | -------- | ------------------------------ |
| S9    | How soon does a running Claude Code session flush transcript lines, and can a read-only tail keep up? | ASM-17   | Measured lag; UI-02's † frozen |

§13.1 open question, the one pitch.md leaves open:

| ID    | Question                                                                                                                                                                                  | Owner / resolution path    |
| ----- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------- |
| OQ-14 | How does a co-author's message reach an agent without the owner relaying it: a signed endorsement by the owner, or a granted role? Until resolved, no imported event is trusted (PRV-02). | Security review, before M8 |

§6.1 actors (the SEC-17 change below names them):

| Actor                                | Capability                                               | In scope |
| ------------------------------------ | -------------------------------------------------------- | -------- |
| Co-author or enrolled peer           | Writes their own origin's events into a shared lane      | Yes      |
| Network attacker between peers       | Reads, drops, replays or alters traffic between peers    | Yes      |
| Web page in the tenant's own browser | Sends requests to loopback ports; attempts DNS rebinding | Yes      |

§6.1 threats:

| #   | Threat                       | Vector                                                                        | Controls                                       |
| --- | ---------------------------- | ----------------------------------------------------------------------------- | ---------------------------------------------- |
| T14 | Co-author injection          | A co-author's or peer's message written to instruct the owner's agents        | PRV-02 (origin-bound trust), PEER-04, OQ-14    |
| T15 | Local web attack on the UI   | DNS rebinding, CSRF or cross-origin reads from a page in the tenant's browser | SEC-20                                         |
| T16 | Rendered-content attack      | Record content that runs script or loads remote resources in the view         | SEC-21                                         |
| T17 | Forged or rewritten segments | A forged origin, or a rewritten chain, delivered by a peer or the network     | REC-18, SEC-22                                 |
| T18 | Erasure gap                  | Content purged on one node persists on peers and in published bundles         | SEC-23 review before publishing; documentation |

## Proposed changes to existing text

### C1 — I4, for P17 (invariant change)

**Flag:** this weakens an invariant. CLAUDE.md and §1.3 require a
security review and a new major version: the SRS becomes 2.0-draft.

Old (§1.3):

> **I4 — Cairn never talks to the network.** No listening sockets, no
> outbound connections, no telemetry. Data leaves the machine only when
> Claude receives recalled content through a tool call, and then travels
> to the model provider like any other context.

New:

> **I4 — The core never talks to the network, and nothing wider is on
> by default.** The core (record, hooks, recall, restore, and
> everything that can reach the model) opens no socket and makes no
> outbound connection: boundary B0. Every other component sits behind
> exactly one wider boundary (SEC-19): B1 loopback only, B2 enrolled
> peers, B3 public read-only. B2 and B3 are off unless the tenant turns
> them on. No component sends telemetry or calls a third-party service.
> Data leaves the machine only when Claude receives recalled content
> through a tool call, or, once the tenant enables B2 or B3, as signed
> segments to peers the tenant enrolled or bundles the tenant reviewed.
> Whatever a peer delivers is untrusted (I2).

Knock-on: the glossary gains **Core** ("the B0 binary `cairn` and the
library behind it"). Appendix B's I4 row title follows. CON-04 can stay
as written if `cairn-ui` and `cairn-peer` are separate binaries that
ship no P0 feature.

### C2 — SEC-01, for P40

Old:

> Shipped binaries MUST NOT open listening sockets or make outbound
> network connections. This MUST be enforced by an import allow-list
> check in CI and by a test suite run under a network-deny sandbox.

New:

> The core binary (`cairn`, boundary B0) MUST NOT open any socket or
> make outbound network connections. Components behind B1–B3 MUST ship
> as separate binaries that the core neither links nor starts.
> `cairn-ui` (B1) MUST listen only on a loopback address and MUST make
> no outbound connection. This MUST be enforced per binary by an import
> allow-list check in CI, and by test suites run under a sandbox that
> denies all network to the core and all but loopback to `cairn-ui`.

Knock-on:

- NFR-09: "No resident process between sessions." becomes "The core
  keeps no resident process between sessions; `cairn-ui` and
  `cairn-peer` run only while the tenant has started them."
- ENG-12 fails on any socket for the core suite, and on any
  non-loopback socket for the `cairn-ui` suite.
- ENG-16's forbidden-import analyzer runs per binary.
- A new ADR records the UI server without superseding ADR-01: the
  core stays daemonless.

### C3 — REC-06 and REC-10, for P21

Old REC-06:

> Every event MUST receive a `seq` that is strictly increasing,
> gap-free, and never reused within its project, assigned inside the
> append transaction.

New REC-06:

> Every event MUST receive an address (`origin`, `seq`). `origin`
> identifies the writing node and session; `seq` is strictly
> increasing, gap-free and never reused within that origin's log, and
> only that origin's writer assigns it, inside the append transaction.

Old REC-10:

> Events MUST form a per-project hash chain: each event stores
> `SHA-256(prev_hash ‖ canonical_encoding(event))`, where the canonical
> encoding includes the payload hash.

New REC-10:

> Each origin's events MUST form their own hash chain: each event
> stores `SHA-256(prev_hash ‖ canonical_encoding(event))`, where
> `prev_hash` is the previous event of the same origin. The canonical
> encoding includes the payload hash and the heads of the other origins
> the writer had seen.

No invariant text changes, but I1's "stable address" becomes
(`origin`, `seq`). Knock-on: the glossary entry **seq**; §8.2 `events`
constraints; `seq` parameters in §9.2 and §9.3; LMK `seq` ranges;
ENG-08's property "`seq` strictly increasing and gap-free", now per
origin; NFR-08's "duplicate `seq` values", now per origin; CON-05 holds
if segments live in SQLite, and is otherwise reworded through the
record/index ADR. This is the identity model M1 blocks on.

### C4 — ADM-08 and glossary "Record", for P23 (touches invariant scope)

**Flag:** the I1, I5 and I10 texts stay the same, but redefining
"record" changes what they range over. Review it with C1 in the same
security review.

Old glossary: "**Record** — The append-only log of events for a
project. The source of truth."

New glossary: "**Record** — The set of append-only, per-origin logs a
node holds for a project. The source of truth."

Old ADM-08:

> `cairn rebuild` MUST regenerate all derived state from the record,
> and a subsequent `cairn verify` MUST confirm it is byte-identical to
> the state before rebuild.

New ADM-08:

> `cairn rebuild` MUST regenerate all derived state from the record,
> and a subsequent `cairn verify` MUST confirm it is byte-identical to
> the state before rebuild. Derived state MUST depend only on the set
> of origin logs held, never on the order in which logs or their
> segments arrived.

### C5 — PRV-02, for P27

Old:

> The default trust policy MUST classify as `trusted` only: `operator`;
> `harness_meta` (structural lifecycle data such as timestamps, token
> counts, session start and end); and `user` when the deployment mode
> is `interactive`. Every other class MUST be `untrusted`.

New:

> The default trust policy MUST classify as `trusted` only events whose
> origin is the local tenant's own node and whose class is `operator`,
> `harness_meta` (structural lifecycle data such as timestamps, token
> counts, session start and end), or `user` when the deployment mode is
> `interactive`. Every other event MUST be `untrusted`, including every
> event of any class whose origin is another node or another tenant.

This tightens I2; it is not an invariant change. Trust becomes relative
to the node, so the same event is trusted at home and untrusted on a
peer. Rebuild stays deterministic per node. OQ-14 tracks the route by
which a co-author might ever be trusted.

### C6 — §8.1 and glossary "Project", for P29

Old §8.1 line:

> `projects/<project-id>/  project-id =
> hex(SHA-256(canonical project path))[:32]  (N)`

New:

> `projects/<project-id>/  project-id =
> hex(SHA-256(canonical repository path))[:32]  (N)`, where the
> repository path is the git common directory when the working
> directory lies in a git work tree, so every worktree of one clone
> maps to one project, and the working directory otherwise.

Old glossary: "**Project** — A working directory as identified by
Claude Code (one transcript directory). Each project has its own
store."

New glossary: "**Project** — A repository clone, identified by its git
common directory, with all of its worktrees; outside git, a working
directory. Each project has its own store and may span several
transcript directories."

Knock-on: ASM-03 maps several transcript slugs to one project. SEC-18
admits the git common directory as a read root. RCL-05 is unchanged:
worktrees now share a project rather than crossing projects.

### C7 — REC-09 and the §8.2 tombstone, for P49

Old REC-09 fragment: "MUST be stored in the content-addressed payload
store keyed by SHA-256".

New: "MUST be stored in the content-addressed payload store keyed by
HMAC-SHA-256 under the tenant's content key (`tenant.json`, §8.1), so
that no stored or tombstoned hash confirms a guessed content without
that key".

Old §8.2 `tombstones`: "purge event `seq`, purged range, reason, hash
of removed content".

New: "purge event `seq`, purged range, reason, keyed hash
(HMAC-SHA-256, tenant content key) of removed content".

Knock-on: §4.3 "a hash of the removed content" says keyed hash. A peer
cannot check a keyed address, so a peer's payload integrity rests on
the segment signature (REC-18). Erasing copies on peers stays out of
reach (T18); pitch.md's "erasure by key" means this keyed-hash fix, not
crypto-shredding of replicas.

### C8 — NG4, for P8 (post-v1)

Old: "NG4 — Multi-host shared stores — Multiplies blast radius;
requires provenance-gated sharing design first (OQ-07)".

New: "NG4 — Multi-host stores in v1.0, and any shared mutable store at
any version — v1.0 is single-host. Later releases replicate per-origin,
append-only logs between enrolled peers (PEER, P2), never a shared
mutable store." OQ-07's resolution path becomes "the PEER design
(§5.13), M8".

### C9 — NG5, for P10

Old: "NG5 — Graphical UI — CLI and MCP only".

New: "NG5 — A graphical interface that is required, or reachable from
off the machine — The lane view (UI-01..04) is an optional local
client behind B1; every operation stays available through CLI and MCP
(UI-03)."

### C10 — §2.2, for P51 (post-v1)

Add a bullet after the durable-volume bullet:

> **Later, with peering:** ephemeral sandboxes whose `CAIRN_HOME` is
> reclaimed. Their record survives only by syncing, outbound, to a
> peer the tenant enrolled before the sandbox is reclaimed (PEER-02,
> SEC-22). Until then, such sandboxes are out of scope.

### C11 — SEC-17 and §6.1, for P36

Old SEC-17:

> A threat-model document MUST be maintained in the repository and
> reviewed at every minor release.

New SEC-17:

> A threat-model document MUST be maintained in the repository and
> reviewed at every minor release. It MUST cover every boundary in the
> register (SEC-19) and the actors that act across them: co-authors and
> enrolled peers, a network attacker between peers, and web pages in
> the tenant's own browser. It MUST compare Cairn, control by control,
> with the closest product the stakeholder names (Zed Delta as of 2026)
> on record signing, co-author trust, dependence on a central service
> and key custody.

Plus the actor rows and threats T14..T18 above.

### C12 — §12, for P25 and P35

Add three milestones after M4. Their ids keep existing references
stable; the order follows D6, experience before sync:

| Milestone                          | Scope                                                                                                     | Exit criteria                                                                                                                    |
| ---------------------------------- | --------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| **M7 — Lane and view, standalone** | LANE-01..04, UI-01..03, RCL-08, REC-18, SEC-19..21, S9                                                    | With all but loopback denied, two harnesses on separate worktrees show in one view with their results; the stakeholder signs off |
| **M8 — Peer network**              | PEER-01..04, SEC-22, LANE-05, OQ-14; needs C1 approved                                                    | Two peers split by a partition keep working and converge to byte-identical lane state on reconnect                               |
| **M9 — Seamless sync and beyond**  | Discovery, outbound-only sandboxes (C10), live co-editing in one worktree, LANE-06, SEC-23, UI-04, REC-17 | Each item lands by its own proposal and ADR                                                                                      |

## Post-v1 claims

P8, P17..P20, P22, P24, P25, P32, P46, P47 and P51 land in M8 or M9,
after v1.0. pitch.md already calls itself "direction, not
requirement". Before the pitch is used outside the project, it should
say that peer to peer is the next step, not part of the first release.

## Proposed pitch changes

### For P26: only you can instruct your agents

Reason: Cairn mediates only its own channels, the restore block
(INJ-03) and recall (RCL-04). The harness still reads CLAUDE.md, files,
web pages and tool output that others wrote, with no Cairn in between.
In automation mode (PRV-04), Cairn does not trust even the owner's
prompts, only operator actions.

Old:

> Only you can instruct your agents; everyone else's words reach them
> as untrusted data.

New:

> Nothing Cairn carries instructs your agents unless you put it there:
> everyone else's words reach them only as untrusted data, and only
> when they ask for it.

### For P31: a pull request in all but its interface

Reason: a pull request also runs review, approvals, required checks
and landing. No requirement specifies any of them. LANE-06 is P2, and
plan 2610022338 puts the merge gate at task 5. The research review
already told the pitch to stop overclaiming beside Zed Delta.

Old:

> That is a pull request in all but its interface. Git keeps the code;
> Cairn keeps the lane.

New:

> It holds what a pull request's conversation holds, plus the runs and
> results behind it. Git keeps the code and lands it; Cairn keeps the
> lane.

### For P43: standalone means nothing leaves the machine

Reason: I4 itself carves out recall. What an agent recalls goes to its
model provider through the harness. Cairn sends nothing, but the claim
as worded is stronger than I4.

Old (decision 9):

> Standalone means nothing leaves the machine.

New:

> Standalone means Cairn sends nothing off the machine; what an agent
> recalls travels to its model provider with the rest of its context,
> as it does today.
