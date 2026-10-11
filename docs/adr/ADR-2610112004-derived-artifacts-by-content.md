---
id: ADR-2610112004
title: "Derived artifacts are compared by content, not by storage layout"
status: proposed
summary: >-
  I10 now says a rebuild reproduces derived artifacts' content exactly,
  whatever their storage layout, and ADM-08's verify compares each one's
  canonical digest. Removing stored content must leave no residue in any
  file. Spike S12 found that a search index kept per append never matches
  a rebuild byte for byte. Proposed until the security reviewer approves
  it.
---
# ADR-2610112004: Derived artifacts are compared by content

## Context

I10 said a rebuild reproduces every derived artifact "exactly", and
ADM-08 had `cairn verify` confirm that a rebuild is byte-identical to the
state before it. PEER-03 asked two peers holding the same segments for
byte-identical room state and index.

Spike S12 is the gate of
[plan 2610012322](../../plan/2610012322_cairn-for-agent-fleets/plan.md)'s
phase 1. It built two writers' record in SQLite with FTS5, as ADR-07
and [ADR-2610110553](ADR-2610110553-sqlite-rusqlite.md) propose. From
it, it derived the search index and room state in one file, the spike's
index file. Its [report][s12] found:

- An index file rebuilt from nothing is byte-identical whatever order
  the segments arrived in.
- A search index kept per append, which NFR-15's latencies need, holds
  the same rows as a rebuild but never the same bytes. That stays true
  after FTS5's `optimize` and `VACUUM`, and with row ids taken from the
  event hash. FTS5's structure record keeps an FTS5 segment id and a
  write counter from its history.
- A full rebuild runs at about 35,000 events/s on a 4 vCPU host, so it
  cannot run on every append.
- Two peers' index files differ in each event's trust level and origin,
  which each node keeps for itself (PRV-02, RCL-09). PEER-03's
  byte-identity across peers could not hold for them.
- An in-place purge left the purged text in the file for 18 of 40
  events without SQLite's `secure_delete`. It left the FTS5 postings in
  every case without FTS5's own `secure-delete`.

The stakeholder chose, on 11 October 2026, to compare derived artifacts
by content, and to narrow PEER-03 to what is the same on every node.

[s12]: https://github.com/jeduden/cairn/pull/35

## Decision

I10's list names the search index where it said "indexes", and its last
sentence becomes: "Rebuilding reproduces their content exactly, whatever
their storage layout." The rest of I10 is unchanged. Derived artifacts
stay a deterministic function of the writer logs a node holds and its
own key set. Arrival order still plays no part.

With it, SRS 3.0-draft changes these requirements:

- **§8.3** defines a derived artifact's canonical digest. It is SHA-256
  over the canonical encoding of every value any read of the artifact
  can return, row by row in key order. Its schema and every stored
  setting that decides what a read returns, such as the search index's
  tokenizer and ranking configuration, are part of it.
- **ADM-08.** `cairn verify` confirms each derived artifact is identical
  in content to its state before rebuild, by its canonical digest. It
  also fails on the store's own integrity check, on a search index that
  disagrees with the event text it indexes, and on a schema object
  Cairn does not ship. No canonical digest is written to the audit log,
  a log, a receipt, a backup or an export, or leaves the node. One kept
  between rebuild and verify is erased by any purge whose events it
  covers. Every read that builds output orders it by values, never by
  storage order.
- **SEC-31.** However stored content is removed, by purge, retention,
  an applied erasure request or a backup restore that reapplies a
  purge, no file Cairn or its storage engine writes may afterwards
  contain the removed events' content, their commitment keys or a
  search index entry naming their addresses. A purge that cannot reach
  that state in time does not report the range erased. It audits,
  counts and names each residue. ADM-07's vacuum must reach that state.
- **§8.1.** The storage engine keeps its temporary storage in memory or
  in the home.
- **ENG-08.** Its property test compares canonical digests after any
  sequence of appends and lifecycle events.
- **PEER-03.** Two peers holding the same segments derive room state and
  a search index identical in content, apart from each event's trust
  level and origin. Each node derives or records those for itself and
  never takes them from another node.

The domain model's derived artifact is its content, never its storage
layout, and two copies are the same when their canonical digest matches.

[invariants.md](../srs/invariants.md) holds the text approved here. Its
SHA-256 is:

```text
1a53333abaa3ad65299df8708c5b61834acdaf709c2bcdfe7c237bdf80e98811
```

The stakeholder, @jeduden, approves this record as security reviewer
(ENG-29) by a comment on the pull request that adds it. The comment
names the record's id, the hash above and the commit it approves. The
approval covers I10 and the requirements listed above as worded in that
commit. Weakening any of them reopens this record under ENG-29. On that
approval the record becomes accepted and supersedes
[ADR-2610091555](ADR-2610091555-initial-invariants.md).

### Security review

A byte comparison never held for a store kept per append, so part of
what it promised was never real. What a content digest alone would
miss, and what answers each:

| A byte comparison would catch                                                   | What answers it now                                                                   |
| ------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| purged text or postings left in free pages or FTS5's index tables               | SEC-31's no-residue rule, checked on the decrypted raw files                          |
| residue from retention, applied erasure requests or a backup restore            | SEC-31 covers every way stored content is removed                                     |
| a plaintext copy in a temporary file, such as `VACUUM`'s                        | §8.1 keeps the storage engine's temporary storage in the home                         |
| a trigger, view or extra index, or a changed tokenizer or ranking configuration | the schema and settings are part of the digest; verify refuses unknown schema objects |
| an index that disagrees with the table it indexes                               | ADM-08's integrity checks of the store and the search index                           |
| output whose order follows storage                                              | ADM-08: every read orders its output by values                                        |

The digest is itself a risk: kept past a purge, it would let anyone
holding the other content confirm a guess at the purged content. ADM-08
keeps it on the node, out of every log, receipt, backup and export, and
erases a kept one on purge.

The ADM-07 scenario reads the store decrypted, with encryption at rest
on and off. It looks for a canary token only the purged events carried,
their commitment keys and search index entries naming them, never for
single shared words. It runs while another process holds a read
transaction, and the purge must not report the range erased before the
residue is gone.

What stays as strong as before:

- Every derived artifact is still a deterministic function of the
  writer logs and the key set, independent of arrival order (I10).
- Verify still fails on any difference in content, trust levels and
  quarantine set included.
- Events, commitments and chain heads are still compared exactly
  (REC-10, REC-18, REC-25).

PEER-03's narrowing adds no path from untrusted content to the model.
Each node keeps deriving trust levels and recording origin for itself,
and never takes either from another node.

One question stays open for the stakeholder. Origin is recorded per
event in the record, outside the chained header (§8.2, REC-10), and
trust levels derive from it. Neither the hash chain nor the canonical
digest covers it.

## Alternatives

- **Keep byte identity and compare only rebuilt snapshots.** Verify
  would then compare one rebuild with another, which proves little, and
  the store would need a second index lifecycle.
- **Keep byte identity and rebuild instead of indexing per append.** It
  breaks NFR-15: a full rebuild of 10M events takes about five minutes.
- **Keep PEER-03 byte-identical but require the same key set.** Each
  peer holds its own keys, so two peers' key sets seldom match, and the
  requirement would seldom apply.

## Consequences

- An invariant changed, so the SRS moves to a new major version,
  3.0-draft, and the release that carries it is a new major version.
- M1 keeps its search index per append and implements the canonical
  digest as `canonical_digest`, replacing the spike's `logical_digest`.
- M1's store sets `PRAGMA secure_delete = ON`, FTS5's `secure-delete`
  and `PRAGMA temp_store = MEMORY`. Its purge checkpoints the
  write-ahead log with `TRUNCATE`, or rebuilds the search index.
- M1's schema may keep trust levels and origin apart from the parts
  every node shares, so PEER-03's shared parts compare directly.
