---
title: "8. Data and storage"
summary: >-
  Normative home layout, logical schema, canonical encoding (RFC 8785
  + SHA-256) and the conservative token estimator.
---
# 8. Data and storage

Semantics in this section are normative. Table and column names are
illustrative; the implementation MAY differ as long as every constraint marked
**(N)** holds.

## 8.1 Home layout

```text
$CAIRN_HOME/                          0700, owned by tenant UID            (N)
├── config.toml                       tenant configuration                 0600
├── tenant.json                       tenant ID binding, redaction salt    0600
├── index.db                          project-id → project path mapping    0600
├── audit/audit-NNNNNN.jsonl          hash-chained audit log               0600
├── logs/                             structured logs (optional)
└── projects/<project-id>/            project-id = hex(SHA-256(canonical project path))[:32]  (N)
    ├── store.db  (+ -wal, -shm)      record and projections               0600
    ├── payloads/ab/cd/<sha256>       content-addressed payloads           0600
    └── work/                         work markers for deferred ingestion
```

Directory names MUST NOT reveal project paths (N).

## 8.2 Logical schema

| Table        | Kind       | Key contents                                                                                                                                                                                                      | Constraints                                                                                                                                                       |
| ------------ | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `meta`       | state      | schema version, project ID, project path                                                                                                                                                                          | single row                                                                                                                                                        |
| `sources`    | state      | path, generation, session ID, parent session ID, agent ID, cursor offset, consumed-prefix hash, status                                                                                                            | UNIQUE(path, generation) **(N)**                                                                                                                                  |
| `events`     | **record** | `seq`, session, source, generation, source line, kind, provenance, trust, tool name, inline text or preview, payload hash and size, token estimate, flags, source timestamp, `prev_hash`, `hash`, redaction count | `seq` PRIMARY KEY, strictly increasing, gap-free **(N)**; UNIQUE(source, generation, source line) **(N)**; rows immutable except content removal by purge **(N)** |
| `events_fts` | projection | FTS5 over event text                                                                                                                                                                                              | rebuildable **(N)**                                                                                                                                               |
| `spans`      | projection | session, `seq` range, boundary kind                                                                                                                                                                               | rebuildable **(N)**                                                                                                                                               |
| `landmarks`  | projection | tier, `seq` range, rendered text, trust                                                                                                                                                                           | rebuildable **(N)**                                                                                                                                               |
| `pins`       | projection | creating `seq`, type, priority, text, text hash, author, active                                                                                                                                                   | rebuildable from `operator`/`user` pin events **(N)**                                                                                                             |
| `quarantine` | projection | creating `seq`, selector, active                                                                                                                                                                                  | rebuildable from quarantine events **(N)**                                                                                                                        |
| `cue_stats`  | projection | moment kind, cues delivered, followed, rejected by gate, muted flag                                                                                                                                               | rebuildable from `cue` and recall events **(N)**                                                                                                                  |
| `tombstones` | projection | purge event `seq`, purged range, reason, hash of removed content                                                                                                                                                  | rebuildable **(N)**                                                                                                                                               |

Purge replaces event content with nothing and keeps the row's `seq`, `hash`, and
`prev_hash`, so the chain stays verifiable and `seq` stays gap-free **(N)**.

## 8.3 Canonical encoding

The hash chain (REC-10) and the audit chain (OPS-02) MUST use the JSON
Canonicalization Scheme (RFC 8785) over a documented field set, with SHA-256
**(N)**. The field set and encoding version are recorded in `meta` so that
future changes remain verifiable.

## 8.4 Token estimation

Budgets (PIN-08, INJ-07, RCL-03, CMP-06) depend on token counts. Cairn MUST use
a conservative estimator, calibrated in M0 against Claude's token counts, such
that the estimate is greater than or equal to the true count for at least 99% of
a representative sample **(N)**.
