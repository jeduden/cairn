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
$CAIRN_HOME/                          0700, owned by the person's OS user  (N)
├── config.toml                       the person's configuration           0600
├── home.json                         home ID binding (SEC-03)             0600
├── audit/audit-NNNNNN.jsonl          hash-chained audit log               0600
├── logs/                             structured logs (optional)
├── store.db  (+ -wal, -shm)          the one store: record and projections, partitioned by writer  0600
├── payloads/<name>                   payloads, named per REC-09           0600
└── work/                             work markers for deferred ingestion
```

A home holds one store, partitioned by writer, never by repository or
room (N). No name in the home MUST reveal a repository's path or
identity (N).

## 8.2 Logical schema

| Table        | Kind       | Key contents                                                                                                                                                                                                                                    | Constraints                                                                                                                                                                                     |
| ------------ | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `meta`       | state      | schema version, home ID                                                                                                                                                                                                                         | single row                                                                                                                                                                                      |
| `sources`    | state      | path, generation, the harness's transcript id, run, parent run, agent ID, cursor offset, consumed-prefix hash, status                                                                                                                           | UNIQUE(path, generation) **(N)**                                                                                                                                                                |
| `events`     | **record** | writer, `seq`, run, source, generation, source line, kind, provenance, trust, tool name, inline text or preview, payload reference and size, commitment (REC-17), token estimate, flags, source timestamp, `prev_hash`, `hash`, redaction count | (writer, `seq`) PRIMARY KEY, `seq` strictly increasing and gap-free per writer **(N)**; UNIQUE(source, generation, source line) **(N)**; rows immutable except content removal by purge **(N)** |
| `events_fts` | projection | FTS5 over event text                                                                                                                                                                                                                            | rebuildable **(N)**                                                                                                                                                                             |
| `spans`      | projection | run, address range, boundary kind                                                                                                                                                                                                               | rebuildable **(N)**                                                                                                                                                                             |
| `landmarks`  | projection | tier, address range, rendered text, trust                                                                                                                                                                                                       | rebuildable **(N)**                                                                                                                                                                             |
| `pins`       | projection | creating address, type, priority, room, text, creating event's commitment, author, active                                                                                                                                                       | rebuildable from `operator`/`user` pin events **(N)**                                                                                                                                           |
| `quarantine` | projection | creating address, selector, active                                                                                                                                                                                                              | rebuildable from quarantine events **(N)**                                                                                                                                                      |
| `tombstones` | projection | purge event address, purged range, counts, reason, commitments of removed events                                                                                                                                                                | rebuildable **(N)**                                                                                                                                                                             |

Purge replaces event content with nothing and keeps the row's address, `hash`,
and `prev_hash`, so the chain stays verifiable and each writer's `seq` stays
gap-free **(N)**.

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
