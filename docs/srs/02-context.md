---
title: "2. Context"
summary: >-
  Stakeholders, deployment context, the assumptions register
  (ASM-01..10, 17..20) about Claude Code behaviour that M0 must verify,
  the hard constraints (CON-01..06), and the personas U1–U9.
---
# 2. Context

## 2.1 Stakeholders

| Stakeholder                      | Interest                                                                                                        |
| -------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| Platform team (primary operator) | Runs Claude Code on self-hosted runners and Agent SDK workers; needs isolation, observability, predictable cost |
| Developers using Claude Code     | Sessions that don't forget; no setup friction; no slowdown                                                      |
| Security team                    | No new exfiltration or injection paths; auditability; purge on request                                          |
| Claude (the agent)               | Small, stable context; precise tools to recover exact detail when needed                                        |
| Reviewers                        | Decide whether a lane may land from its story, diff and evidence, with a signature that counts                  |
| Live collaborators               | Join someone else's lane live without derailing it, and know their rights                                       |
| Open-source maintainers          | Read how an outside contribution was made, with foreign history never instructing their agents                  |

## 2.2 Deployment context

- **Primary:** Claude Code sessions executed by self-hosted runners, and Claude
  Agent SDK workers, inside sandboxes we operate.
- **Primary, too:** developer workstations (Linux and macOS) running a fleet
  of harnesses, one worktree each.
- **Later, with PEER:** ephemeral sandboxes whose `CAIRN_HOME` is reclaimed
  keep their record only by sealing and offering segments to an enrolled peer
  (PEER-05). Until then they are out of scope.
- The model runs at the provider. Tool inputs and outputs, including anything
  Cairn returns through recall, are sent to the model provider as ordinary
  context. Cairn does not change this boundary; its own components stay
  inside the boundaries of §6.3 (I4).
- Runner workspaces, `~/.claude/`, and `CAIRN_HOME` live on durable volumes so
  sessions can pause and resume.

## 2.3 Assumptions register

Cairn depends on harness behaviour we don't control. Each assumption below MUST
be verified in milestone M0 (§12) and re-verified by contract tests on every
supported Claude Code release (ENG-17). If an assumption fails, the listed
requirements are re-planned.

| ID     | Assumption                                                                                                                                                                                                                                                                             | Evidence so far                                                                          | Verified in | Affects                  |
| ------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- | ----------- | ------------------------ |
| ASM-01 | Hook inputs include `session_id`, `transcript_path`, `cwd`, `hook_event_name`; `SessionStart` adds `source` ∈ {`startup`, `resume`, `clear`, `compact`}; `PreCompact` adds `trigger` and `custom_instructions`; `PostCompact` adds `compact_summary`; `UserPromptSubmit` adds `prompt` | Claude Code hook docs and community references                                           | S1          | §9.1                     |
| ASM-02 | `SessionStart` and `UserPromptSubmit` hooks can return `additionalContext`; `PostCompact` cannot                                                                                                                                                                                       | Hook docs; claude-compact-controller v1.0.1 release notes                                | S1          | INJ-*                    |
| ASM-03 | Transcripts are JSONL files under `~/.claude/projects/<project-slug>/`, one per session, with subagent transcripts under `<session>/subagents/`                                                                                                                                        | kcp-memory, claude-lens                                                                  | S3          | REC-01..06               |
| ASM-04 | Transcripts are append-mostly, but a file can be truncated or rewritten (e.g. rewind or resume flows)                                                                                                                                                                                  | Unverified                                                                               | S3          | REC-07                   |
| ASM-05 | Claude Code may delete transcripts after `cleanupPeriodDays` (default 30)                                                                                                                                                                                                              | session-index README                                                                     | S3          | REC-08                   |
| ASM-06 | Hooks fired for a subagent's compaction carry the parent's identity and no subagent fields                                                                                                                                                                                             | anthropics/claude-code#91910                                                             | S1          | REC-04, INJ-05           |
| ASM-07 | Claude Code plugins can bundle hooks and MCP servers, and the Agent SDK can load plugins                                                                                                                                                                                               | Claude Code and Agent SDK docs                                                           | S4          | ADM-01                   |
| ASM-08 | Self-hosted runners seed the runner host's `~/.claude/` into each session; each runner is locked to one user account                                                                                                                                                                   | Claude Code self-hosted environments docs                                                | S4          | ADM-01, SEC-01           |
| ASM-09 | A `PreCompact` hook exiting with code 2 blocks compaction                                                                                                                                                                                                                              | One hooks guide says yes; Morph plugin README says native compaction cannot be prevented | S6          | OQ-01                    |
| ASM-10 | Hook timeouts are configurable per hook, but `SessionEnd` receives a short shared budget (≈1.5 s)                                                                                                                                                                                      | lcm PR #517                                                                              | S1          | NFR-02                   |
| ASM-17 | Claude Code appends each transcript line within 1 s of the event while a session runs                                                                                                                                                                                                  | none yet                                                                                 | S9          | VIEW-02                  |
| ASM-18 | While a `PermissionRequest` hook runs, Claude Code's own prompt stays answerable, and the hook may return no decision                                                                                                                                                                  | none yet; the fleet-developer review suspects not                                        | S9          | OWN-06, OWN-15           |
| ASM-19 | The Agent SDK, ACP and the Codex app-server expose input interfaces for steer, interrupt and stop                                                                                                                                                                                      | none yet                                                                                 | S10         | OWN-03, OWN-15           |
| ASM-20 | The root commit, HEAD, refs and trees of a clone are readable from git's files without starting a process                                                                                                                                                                              | none yet                                                                                 | S11         | LANE-02, LANE-06, REC-20 |

## 2.4 Constraints

| ID     | Constraint                                                                                                                                                                                                                                                         |
| ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| CON-01 | Implementation language is Go.                                                                                                                                                                                                                                     |
| CON-02 | Cairn ships as one binary, `cairn`, whose components (the core, `cairn ui`, `cairn run`, `cairn peer`, `cairn publish`, `cairn bridge`) are its entry points, statically linked (`CGO_ENABLED=0`) for linux/amd64, linux/arm64, darwin/arm64.                      |
| CON-03 | No runtime dependency on Python, Node.js, or a database server for any P0 feature.                                                                                                                                                                                 |
| CON-04 | No network access, at build-verified level, in the core component. Every other component is confined, at build-verified level, to the one boundary the register assigns it, whether or not it shares an executable with the core (SEC-01, SEC-19, ENG-12, ENG-16). |
| CON-05 | The record is append-only, one log per writer, held in sealed segments. Derived state is rebuilt from the segments a node holds. Payloads live in a keyed store. Formats and storage engines are chosen by ADR.                                                    |
| CON-06 | No feature at any boundary may depend on a central or third-party service. Every node can serve what it holds, and self-hosting is the normal case.                                                                                                                |

## 2.5 Personas

Each persona is a reviewer agent in `.claude/agents/`, run by the
persona-review skill against every pitch, plan and requirement change.
Every requirement traces to at least one persona; Appendix C lists them.
Stakeholders map to personas: platform team U1; developers U2, U3 and U4;
reviewers U5; live collaborators U6; open-source maintainers U7; security
team U8; Claude U9.

| #   | Persona                 | Agent                             | Who                                                                                           | Core need                                                                                     | Gives up when                                                                            |
| --- | ----------------------- | --------------------------------- | --------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| U1  | Platform operator       | `persona-platform-operator`       | Runs Cairn for many developers on self-hosted runners and Agent SDK workers; primary operator | Isolation, every failure counted, explicit reversible installs, bounded disk, memory and cost | A failure is silent, tenants touch each other, config changes unasked, use is unbounded  |
| U2  | Fleet developer         | `persona-fleet-developer`         | Five to ten agents on one machine, one worktree each, steered all day                         | See who needs them and why; answer from anywhere; keep their speed                            | Setup is slow or changes config; a new step per agent; the tool slows them               |
| U3  | Multi-machine developer | `persona-multi-machine-developer` | Agents on a laptop, a home server and reclaimed cloud sandboxes; often offline                | One view across nodes; partitions that merge; no vendor relay; sandbox history that survives  | A central service; a partition loses or duplicates work; sandbox history vanishes        |
| U4  | Returning owner         | `persona-returning-owner`         | Comes back after hours or days and asks what the agents did                                   | A summary that points into the record; ranked waiting items; proof nothing was lost; search   | The summary replaces the record; gaps are silent; catching up beats reading git log      |
| U5  | Reviewer                | `persona-reviewer`                | Decides whether a lane may land                                                               | Diff, why and evidence in one place; claim versus run versus CI; a signature that counts      | Authors approve their own lanes; done with nothing that checked it; landed link lost     |
| U6  | Live collaborator       | `persona-live-collaborator`       | Joins someone else's lane live to help, pair or take over                                     | See it live; help without derailing; know their rights; take over on handover                 | Watch-only; words lost or reaching the agent unseen; joining needs a service             |
| U7  | Open-source maintainer  | `persona-oss-maintainer`          | Receives outside contributions with their lanes from strangers                                | Read how it was made; foreign history never instructs; redaction; no account to join          | Foreign history reaches agents as more than data; secrets leak; fabrication undetectable |
| U8  | Security officer        | `persona-security-officer`        | Signs off that Cairn adds no exfiltration or injection path and that audit holds              | No automatic path to the model; no unasked traffic; verifiable record; erasure that holds     | Any outside content reaches the model unasked; a socket beyond its boundary              |
| U9  | Agent                   | `persona-agent`                   | Claude, compacted many times in a long session                                                | Pins back verbatim; small exact recall; provenance on everything recalled; no pushed text     | Pins summarized or missing; recall floods or misleads; surprise text in context          |
