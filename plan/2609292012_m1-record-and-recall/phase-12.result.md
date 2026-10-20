---
n: 12
title: "Search: the search index, the query compiler and event_search"
status: "✅"
result: true
summary: >-
  @REC-11 and @SEC-04 off @pending and passing. Event and payload text is
  indexed in the append transaction up to 1 MiB, and event_search finds
  it by literal terms, ranked by BM25 over the newest 2,000 matches,
  damped per run. @RCL-02 cannot pass in M1 and is raised; NFR-03 is
  benchmarked; the index is a derived file of its own, so the record
  holds 1.42x its text, within NFR-09's 1.5x; the index is 0.75x.
---
# Phase 12 result

## Handoff

Outcome:

- REC-11 holds: `@REC-11` is off `@pending`, passing against the built
  core, confined. An event's text is indexed in the transaction that
  appends it, payload text too, up to the 1 MiB indexing cap. A search
  finds the start of a 2 MiB tool result, not its text past the cap.
- SEC-04 holds: `@SEC-04` is off `@pending`, five rows passing on a
  store of 1M events. Caller text is literal terms, so an operator in
  it runs nothing. 17 terms, a term of 65 characters and a leading
  wildcard are refused in fixed text. A search past its deadline is
  cancelled, counted `query_cancelled` and audited (I6).
- RCL-02's M1 part: `event_search` takes `query`, `any_of`, `all_of`,
  `phrase`, `scope`, `room`, `run`, `provenance`, `kind`, `trust`,
  `since`, `until` and `k`. It returns at most `k` hits (default 10,
  at most 50) in the envelope, each with its REC-09 preview and score:
  BM25 over the newest 2,000 matches after the filters, each further
  hit of one writer damped.
- The search index is a derived artifact (I10) whose row ids derive from
  the address. ENG-08's property holds over 32 seeded records in either
  arrival order: the index kept per append, a rebuild and the other
  order share one canonical digest.
- The `query_parser` fuzz target ran 3.4M inputs in 60 s without a
  crash, from eight committed seeds.
- NFR-03 is benchmarked: `search_events` is 11 ms median over 2,000
  lines, process start included. On 1M events a release build took
  166 ms p95 at worst (`search_latency.rs`, ignored by default).
- Frozen with evidence, in their own commit: REC-11's 1 MiB cap (290 ms
  to index one event at it, release build), RCL-02's 50 hits and
  SEC-04's 16 terms of 64 characters.
- I2, I6, I9 and I10 hold: hits reach the model only in the envelope,
  refusals are fixed text, a cancelled query is audited.

Inherited:

- Crate `search`: `compile`, `Request`, `rank::{rank, Hit}`.
- `store::search_index`, `Store::{search, rebuild_search_index,
  search_index_digest, search_index_unindexed_writers, preview}`;
  `Item::score`; `Server::take_cancelled`.
- `Context::query_deadline`, a debug build's seam; benchmark
  `search_events`; steps in `record_search.rs`.

For the stakeholder:

- NFR-09 conflicted with REC-11 (2.095x with the index in `store.db`).
  The owner ruled: measure the index apart. It is now a derived
  file, `search_index`, beside `store.db` (I10). The store overhead
  test (`store_overhead.rs`) measures the record alone: 1.418 times the
  stored text of the 23 recorded transcripts, within the frozen 1.5. It
  prints the index apart: 2,801,664 bytes, 0.751 times. The SRS sets no
  index target; the owner may freeze one.
- M1 deviates from RCL-02 and §9.2 twice. A run cannot yet be derived
  from its seats' writers, so `run` takes a writer id and damping is
  per writer. RCL-02 says nothing of ranking only the newest 2,000
  matches. `recall.feature` line 26 ("a short run ... a long run") is
  proven only per writer.
- The index leaves out a writer whose slot another holds, and seqs from
  2^28: counted `search_index_writer_unindexed` and
  `search_index_seq_beyond_limit` (I6).
- `@RCL-02` stays `@pending`, no step bound. Its calls name no scope,
  so `run` needs the session binding (phase 9). Its transcript
  "mixed-provenance" has no steps, and no live ingest (phase 10) reads
  it. `trust "trusted"` needs phase 11, `scope "rooms"` phase 13. Unit
  tests in `mcp` and `search` prove its filters, cap and damping.
- `@SEC-04`'s deadline row sets `CAIRN_TEST_QUERY_DEADLINE_MS`, as no
  bounded search runs for 2 s. NFR-03 is stated at 10M events; the
  evidence is 1M.
- Stakeholder paths, each in its own commit: the `search` crate, the
  fuzz target, the steps and features, the benchmark, the SRS freeze.
- Terms: indexing cap, damping, score, literal term. "Slot" is code only.

Parked:

- `cairn verify` naming a left-out writer or a seq from 2^28 on: M3.
- Purge and quarantine must reach the index (`row_id_of`, FTS5
  `secure-delete`): phase 14.
- Settings for the indexing cap and the search deadline: ADM-04.
- Recall scopes, trust and `run`: phases 9 to 13. `cairn event search`:
  M7.

Phase 13 needs `score` in its golden files, RCL-05's scopes and RCL-07.
