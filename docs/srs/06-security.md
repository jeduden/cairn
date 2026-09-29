---
title: "6. Security"
summary: >-
  Threat model (assets, actors, threats T1–T12 and their controls) and
  the normative security requirements SEC-01..18.
---
# 6. Security

## 6.1 Threat model

### Assets

| Asset                         | Why it matters                                                                         |
| ----------------------------- | -------------------------------------------------------------------------------------- |
| Record content                | Contains code, conversations, tool output, and possibly secrets that escaped redaction |
| Pins                          | Directly shape agent behaviour after every compaction                                  |
| Restore channel               | Text Cairn places into the model's context without a tool call                         |
| Recall channel                | Text Claude pulls into context on request                                              |
| Agent configuration           | Hooks and MCP registrations control what runs in every session                         |
| Cairn binary and dependencies | Runs inside every session                                                              |

### Actors

| Actor                                            | Capability                                                                                  | In scope                                        |
| ------------------------------------------------ | ------------------------------------------------------------------------------------------- | ----------------------------------------------- |
| External content author                          | Controls text Claude reads: web pages, repositories, issues, dependency READMEs, file names | Yes — primary threat                            |
| Malicious or compromised MCP server              | Controls its tool results and tool names                                                    | Yes                                             |
| Other tenant on a shared host                    | Separate OS user on the same machine                                                        | Yes                                             |
| Repository author                                | Controls project files, including `.cairn.toml` and `CLAUDE.md`                             | Yes                                             |
| Supply-chain attacker                            | Targets Cairn's dependencies, build, or release                                             | Yes                                             |
| Process running as the same OS user              | Can read the tenant's files directly                                                        | **No** — equivalent to the tenant; out of scope |
| Root on the host, compromised Claude Code binary | Full control                                                                                | **No**                                          |

### Threats and controls

| #   | Threat                              | Vector                                                                           | Controls                                                                                                                  |
| --- | ----------------------------------- | -------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| T1  | Persistent prompt injection         | Untrusted content stored, then placed into a later session's context             | Pull-only recall and `TrustedText` (INJ-03, INJ-04), envelopes (RCL-04), trust policy (PRV-02..05), no promotion (MEM-01) |
| T2  | Compaction-driven poisoning         | Text crafted to survive summarization into compaction summaries or derived state | `harness_text` untrusted and never restored (PRV-03, INJ-09); structural landmarks only (LMK-02..04)                      |
| T3  | Injection through structural fields | File named, or MCP tool named, like an instruction                               | Sanitization to allow-list (LMK-03)                                                                                       |
| T4  | Pin forgery                         | Agent or external content creates a pin                                          | Trusted-only pin creation (PIN-01, PIN-02, PIN-05)                                                                        |
| T5  | Security downgrade via repository   | Malicious `.cairn.toml` enables per-prompt injection or trusts more provenance   | Project config may only tighten (SEC-11)                                                                                  |
| T6  | Cross-tenant access                 | Another user reads or writes a tenant's store                                    | Permissions and ownership checks (SEC-02), tenant binding (SEC-03), no sockets (SEC-01)                                   |
| T7  | Secret disclosure                   | Secrets in transcripts stored, recalled, or backed up                            | Redaction before storage (SEC-08), encryption at rest (SEC-09), purge (ADM-07, SEC-14)                                    |
| T8  | Denial of service                   | Pathological queries, oversized inputs, runaway kernel code                      | Bounded queries (SEC-04), input limits (SEC-05), kernel limits (CMP-05), fail-open (NFR-06)                               |
| T9  | Path traversal                      | Hook payload or transcript fields pointing outside allowed roots                 | Path validation (SEC-18)                                                                                                  |
| T10 | Configuration tampering             | Tool silently rewrites agent settings                                            | Plugin distribution, explicit install, read-only doctor (ADM-01..03, ADM-10)                                              |
| T11 | Supply-chain compromise             | Malicious dependency or release artifact                                         | Dependency policy, reproducible signed builds (ENG-16..20)                                                                |
| T12 | Unnoticed compromise                | Poisoned content acted on by unattended agents                                   | Audit log (OPS-01, OPS-02), flagging (PRV-07), recall-taint signal (SEC-13), quarantine (SEC-12)                          |

## 6.2 Security requirements (SEC)

| ID     | Pri | Requirement                                                                                                                                                                                                                                                                                                    | Ver  | Traces |
| ------ | --- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---- | ------ |
| SEC-01 | P0  | Shipped binaries MUST NOT open listening sockets or make outbound network connections. This MUST be enforced by an import allow-list check in CI and by a test suite run under a network-deny sandbox.                                                                                                         | T, I | I4     |
| SEC-02 | P0  | Cairn MUST create directories `0700` and files `0600`, and MUST refuse to operate if `CAIRN_HOME` or any store file is not owned by the running UID or has looser permissions.                                                                                                                                 | T    | I8     |
| SEC-03 | P0  | If a tenant ID is configured, Cairn MUST refuse to open the home unless the tenant ID supplied by the environment matches.                                                                                                                                                                                     | T    | I8     |
| SEC-04 | P0  | The query compiler MUST treat all caller text as literal terms (escaping FTS5 syntax), expose boolean and phrase operators only through structured parameters, limit queries to 16 terms† of at most 64 characters†, forbid leading wildcards, and cancel any query exceeding its deadline (default 2 s†).     | T    | I9     |
| SEC-05 | P0  | Cairn MUST enforce input limits: hook stdin ≤ 1 MiB†, JSON nesting depth ≤ 64†, and transcript lines above the payload threshold MUST be streamed to the payload store rather than held in memory.                                                                                                             | T    | I9     |
| SEC-06 | P0  | Recalled content MUST be carried as JSON string values inside the envelope, so no stored content can alter the envelope's structure, and each envelope MUST state that its contents are historical data and not instructions.                                                                                  | T    | I2     |
| SEC-07 | P0  | The `TrustedText` guarantee (INJ-03) MUST be covered by a static check in CI that fails if any code outside the injection package constructs restore content.                                                                                                                                                  | I, T | I2     |
| SEC-08 | P0  | Redaction MUST apply gitleaks-compatible rules plus tenant-defined patterns to every text field, payload, and hook input before anything is written. Redacted spans MUST be replaced by `[REDACTED:<rule>:<HASH8>]`, where HASH8 is a tenant-salted hash that allows correlation without revealing the secret. | T    | I1     |
| SEC-09 | P1  | Cairn SHOULD support encryption at rest for the database and payload store, with keys obtained through a secret reference (SEC-10). Deployment documentation MUST require encrypted volumes where this is not enabled.                                                                                         | T, I | —      |
| SEC-10 | P0  | The v1 core MUST handle no credentials. Any future credential MUST be resolved per use from an explicit secret reference (a `0600` file, the OS keychain, or a secret-manager command), MUST NOT be inherited from the parent environment by default, and MUST NOT be logged or persisted.                     | I    | I4     |
| SEC-11 | P0  | Project configuration (`.cairn.toml`, which is repository-controlled) MUST only be able to tighten security-relevant settings: it MUST NOT enable any injection, widen recall scope, change trust policy, change deployment mode, or disable redaction or flagging.                                            | T    | I2, I7 |
| SEC-12 | P0  | Quarantine MUST be available by `seq` range, session, provenance class, flag, and time window; MUST be recorded as an `operator` event; MUST take effect for all subsequent recall, landmark, and injection operations immediately; and release MUST be equally recorded.                                      | T    | I5     |
| SEC-13 | P1  | Cairn SHOULD maintain a per-session recall-taint flag, set when untrusted content has been returned to that session, queryable by `cairn policy check`, and SHOULD ship an example `PreToolUse` policy hook that requires approval for configured sensitive actions once the flag is set.                      | T    | I2     |
| SEC-14 | P0  | Purge MUST enable SQLite `secure_delete` for the operation, rewrite the database, and unlink payload files. Documentation MUST state that physical erasure on SSDs is not guaranteed without encryption at rest.                                                                                               | T, I | I5     |
| SEC-15 | P0  | Cairn MUST NOT include telemetry, crash reporting to remote services, or update checks.                                                                                                                                                                                                                        | I, T | I4     |
| SEC-16 | P0  | Hook inputs MUST be validated against a schema; invalid inputs MUST be rejected fail-open (no injection) and audited.                                                                                                                                                                                          | T    | I6, I9 |
| SEC-17 | P0  | A threat-model document MUST be maintained in the repository and reviewed at every minor release.                                                                                                                                                                                                              | I    | —      |
| SEC-18 | P0  | Every path received from hook inputs, transcripts, or configuration MUST be resolved (including symlinks) and MUST lie within the configured transcript roots or `CAIRN_HOME`; otherwise it MUST be rejected and audited.                                                                                      | T    | I8     |
