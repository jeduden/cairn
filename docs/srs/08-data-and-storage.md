---
title: "8. Data and storage"
summary: >-
  Normative home layout, logical schema, canonical encoding (RFC 8785
  + SHA-256) and the conservative model-token estimator.
---
# 8. Data and storage

Semantics in this section are normative. Table and column names are
illustrative; the implementation MAY differ as long as every constraint marked
**(N)** is met.

## 8.1 Home layout

```text
$CAIRN_HOME/                          0700, belonging to one OS user       (N)
├── config.toml                       the principal's configuration        0600
├── home.json                         home id binding (home.id, SEC-03)    0600
├── audit/audit-NNNNNN.jsonl          hash-chained audit log               0600
├── logs/                             structured logs (optional)
├── store.db  (+ -wal, -shm)          the store: record and derived artifacts, partitioned by writer  0600
├── payloads/<name>                   the store's payload store, named per REC-09  0600
└── ingest/                           ingest markers for deferred ingestion
```

A home contains one store, `store.db` with its payload store, partitioned by
writer, never by repository or room (N). Names in the home MUST NOT reveal a
repository's path or identity (N).

## 8.2 Logical schema

| Table                | Kind       | Key contents                                                                                                                                                                                                                                                                                       | Constraints                                                                                                                                                                                                    |
| -------------------- | ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `meta`               | state      | schema version, home id                                                                                                                                                                                                                                                                            | single row                                                                                                                                                                                                     |
| `transcript_sources` | state      | path, generation, the harness's transcript id, the runs it holds (a harness session's transcripts hold its main agent's run and one per subagent, a subagent's possibly in its own file, ASM-03), each with its parent run and the harness's agent id, cursor offset, consumed-prefix hash, status | UNIQUE(path, generation) **(N)**                                                                                                                                                                               |
| `events`             | **record** | writer, `seq`, run, transcript source, generation, transcript line, kind, provenance, origin, recorder, tool name, inline text or preview, payload reference and size, commitment (REC-17), model-token estimate, flags, transcript timestamp, `prev_hash`, `hash`, redaction count                | (writer, `seq`) PRIMARY KEY, `seq` strictly increasing and gap-free per writer **(N)**; UNIQUE(transcript source, generation, transcript line) **(N)**; rows immutable except content removal by purge **(N)** |
| `events_fts`         | derived    | FTS5 over event text                                                                                                                                                                                                                                                                               | rebuildable **(N)**                                                                                                                                                                                            |
| `spans`              | derived    | run, address range, span boundary kind                                                                                                                                                                                                                                                             | rebuildable **(N)**                                                                                                                                                                                            |
| `landmarks`          | derived    | tier, address range, rendered text, taint                                                                                                                                                                                                                                                          | rebuildable **(N)**                                                                                                                                                                                            |
| `pins`               | derived    | creating address, pin version, type, priority, room, text, creating event's commitment, each version's author, stamps, active                                                                                                                                                                      | rebuildable from pin events (`operator` for device-seat pins, `assistant` for run-seat pins), stamps and unstamps **(N)**                                                                                      |
| `quarantine`         | derived    | creating address, selector, active                                                                                                                                                                                                                                                                 | rebuildable from quarantine events **(N)**                                                                                                                                                                     |
| `tombstones`         | derived    | purge event address, purged range, counts, reason or retention policy, commitments of removed events                                                                                                                                                                                               | rebuildable **(N)**                                                                                                                                                                                            |

Purge replaces event content with nothing and keeps the row's address, `hash`,
and `prev_hash`, so the chain stays verifiable and each writer's `seq` stays
gap-free **(N)**.

## 8.3 Canonical encoding

The hash chain (REC-10) and the audit chain (OPS-02) MUST use the JSON
Canonicalization Scheme (RFC 8785) over a documented field set, with SHA-256
**(N)**. The field set and encoding version are recorded in `meta` so that
future changes remain verifiable.

## 8.4 Model-token estimation

The pin budget (PIN-08) and the model-token caps of INJ-07, RCL-03 and
CMP-06 depend on model-token counts.
Cairn MUST use a conservative estimator, calibrated in M0 against
Claude's own model-token counts, such that the estimate is greater
than or equal to the true count for at least 99% of a representative
sample **(N)**.
