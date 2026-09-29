---
title: "2. Context"
summary: >-
  Stakeholders, deployment context, the assumptions register
  (ASM-01..10) about Claude Code behaviour that M0 must verify, and
  the hard constraints (CON-01..05).
---
# 2. Context

## 2.1 Stakeholders

| Stakeholder                      | Interest                                                                                                        |
| -------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| Platform team (primary operator) | Runs Claude Code on self-hosted runners and Agent SDK workers; needs isolation, observability, predictable cost |
| Developers using Claude Code     | Sessions that don't forget; no setup friction; no slowdown                                                      |
| Security team                    | No new exfiltration or injection paths; auditability; purge on request                                          |
| Claude (the agent)               | Small, stable context; precise tools to recover exact detail when needed                                        |

## 2.2 Deployment context

- **Primary:** Claude Code sessions executed by self-hosted runners, and Claude
  Agent SDK workers, inside sandboxes we operate.
- **Secondary:** Claude Code on developer workstations (Linux and macOS).
- The model runs at the provider. Tool inputs and outputs, including anything
  Cairn returns through recall, are sent to the model provider as ordinary
  context. Cairn does not change this boundary (I4).
- Runner workspaces, `~/.claude/`, and `CAIRN_HOME` live on durable volumes so
  sessions can pause and resume.

## 2.3 Assumptions register

Cairn depends on harness behaviour we don't control. Each assumption below MUST
be verified in milestone M0 (§12) and re-verified by contract tests on every
supported Claude Code release (ENG-17). If an assumption fails, the listed
requirements are re-planned.

| ID     | Assumption                                                                                                                                                                                                                                                                             | Evidence so far                                                                          | Verified in | Affects        |
| ------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- | ----------- | -------------- |
| ASM-01 | Hook inputs include `session_id`, `transcript_path`, `cwd`, `hook_event_name`; `SessionStart` adds `source` ∈ {`startup`, `resume`, `clear`, `compact`}; `PreCompact` adds `trigger` and `custom_instructions`; `PostCompact` adds `compact_summary`; `UserPromptSubmit` adds `prompt` | Claude Code hook docs and community references                                           | S1          | §9.1           |
| ASM-02 | `SessionStart` and `UserPromptSubmit` hooks can return `additionalContext`; `PostCompact` cannot                                                                                                                                                                                       | Hook docs; claude-compact-controller v1.0.1 release notes                                | S1          | INJ-*          |
| ASM-03 | Transcripts are JSONL files under `~/.claude/projects/<project-slug>/`, one per session, with subagent transcripts under `<session>/subagents/`                                                                                                                                        | kcp-memory, claude-lens                                                                  | S3          | REC-01..06     |
| ASM-04 | Transcripts are append-mostly, but a file can be truncated or rewritten (e.g. rewind or resume flows)                                                                                                                                                                                  | Unverified                                                                               | S3          | REC-07         |
| ASM-05 | Claude Code may delete transcripts after `cleanupPeriodDays` (default 30)                                                                                                                                                                                                              | session-index README                                                                     | S3          | REC-08         |
| ASM-06 | Hooks fired for a subagent's compaction carry the parent's identity and no subagent fields                                                                                                                                                                                             | anthropics/claude-code#91910                                                             | S1          | REC-04, INJ-05 |
| ASM-07 | Claude Code plugins can bundle hooks and MCP servers, and the Agent SDK can load plugins                                                                                                                                                                                               | Claude Code and Agent SDK docs                                                           | S4          | ADM-01         |
| ASM-08 | Self-hosted runners seed the runner host's `~/.claude/` into each session; each runner is locked to one user account                                                                                                                                                                   | Claude Code self-hosted environments docs                                                | S4          | ADM-01, SEC-01 |
| ASM-09 | A `PreCompact` hook exiting with code 2 blocks compaction                                                                                                                                                                                                                              | One hooks guide says yes; Morph plugin README says native compaction cannot be prevented | S6          | OQ-01          |
| ASM-10 | Hook timeouts are configurable per hook, but `SessionEnd` receives a short shared budget (≈1.5 s)                                                                                                                                                                                      | lcm PR #517                                                                              | S1          | NFR-02         |

## 2.4 Constraints

| ID     | Constraint                                                                                                        |
| ------ | ----------------------------------------------------------------------------------------------------------------- |
| CON-01 | Implementation language is Go.                                                                                    |
| CON-02 | The core ships as a single statically linked binary (`CGO_ENABLED=0`) for linux/amd64, linux/arm64, darwin/arm64. |
| CON-03 | No runtime dependency on Python, Node.js, or a database server for any P0 feature.                                |
| CON-04 | No network access, at build-verified level, in any binary that ships P0 features (SEC-01, ENG-12).                |
| CON-05 | Storage is embedded SQLite (WAL mode) plus a content-addressed file store.                                        |
