---
id: ADR-2610112004
title: "Derived artifacts are compared by content, not by storage layout"
status: proposed
summary: >-
  I10 now says a rebuild reproduces derived artifacts' content exactly,
  whatever their storage layout, and ADM-08's verify compares a canonical
  digest of that content. A purge must leave no byte of what it erased
  anywhere in the store. Spike S12 found that an index kept per append
  never matches a rebuild byte for byte. Proposed until the security
  reviewer approves it.
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
it, it derived the search index and room state. It found:

- An index file rebuilt from nothing is byte-identical whatever order
  the segments arrived in.
- An index file kept per append, which NFR-15's latencies need, holds
  the same rows as a rebuild but never the same bytes. That stays true
  after FTS5's `optimize` and `VACUUM`, and with row ids taken from the
  event hash. FTS5's structure record keeps a segment id and a write
  counter from its history.
- A full rebuild runs at about 35,000 events/s on a 4 vCPU host, so it
  cannot run on every append.
- Two peers' index files differ in the trust levels and origin each
  node derives for itself (PRV-02, RCL-09). By the domain model's
  definitions those are per node, so PEER-03's byte-identity across
  peers could not hold for them.
- An in-place purge left the purged text in the file for 18 of 40
  events without SQLite's `secure_delete`, and left the FTS5 postings
  in every case without FTS5's own `secure-delete`.

The stakeholder chose, on 11 October 2026, to compare derived artifacts
by content, and to narrow PEER-03 to what is the same on every node.

## Decision

I10's last sentence becomes: "Rebuilding reproduces their content
exactly, whatever their storage layout." The rest of I10 is unchanged.
Derived artifacts stay a deterministic function of the writer logs a
node holds and its own key set. Arrival order still plays no part.

With it, SRS 3.0-draft changes three requirements:

- **ADM-08.** `cairn verify` confirms each derived artifact is identical
  in content to its state before rebuild, by a canonical digest of that
  content that no storage layout, insertion history or arrival order
  affects.
- **ADM-07.** A purge leaves no byte of the purged content, its
  commitment keys or its search index entries in any file of the store,
  its free space and write-ahead log included.
- **PEER-03.** Two peers holding the same segments derive room state and
  a search index identical in content, apart from the trust levels and
  origin each node derives for itself.

The domain model's derived artifact is its content, never its storage
layout.

[invariants.md](../srs/invariants.md) holds the text approved here. Its
SHA-256 is:

```text
78212f9e15402a4bf04014b3913ca4d3c11ba514f665e54750380898ad877253
```

The stakeholder, @jeduden, approves this record as security reviewer
(ENG-29) by a comment on the pull request that adds it, naming the
record's id and the hash above. On that approval the record becomes
accepted and supersedes
[ADR-2610091555](ADR-2610091555-initial-invariants.md).

### Security review

What the change gives up: a byte comparison would also have caught
bytes no row holds, such as a purged event's text left in a free page
or its postings left in FTS5's index tables. A content digest cannot
see them.

How that is made up for:

- ADM-07 now forbids any such residue outright, free space and the
  write-ahead log included. Its scenario checks the store's raw bytes
  after a purge, as the spike's scan did. The spike measured what meets
  it: SQLite's `secure_delete`, FTS5's `secure-delete`, or rebuilding
  the index file, and a write-ahead log checkpoint.
- SEC-31 still erases every other copy a node holds, and REC-17 keeps
  every replicated structure from confirming a guess at purged content.

What stays as strong as before:

- Every derived artifact is still a deterministic function of the
  writer logs and the key set, independent of arrival order (I10).
- Verify still fails on any difference in content, trust levels and
  quarantine set included.
- The record is untouched. Segments and hash chains are still compared
  byte for byte (REC-10, REC-18).

PEER-03's narrowing adds no path from untrusted content to the model.
Each node keeps deriving trust levels and origin for itself. The change
only stops asking two nodes to agree on values the model defines as per
node.

## Alternatives

- **Keep byte identity and compare only rebuilt snapshots.** Verify
  would then compare one rebuild with another, which proves little, and
  the store would need a second index lifecycle.
- **Keep byte identity and rebuild instead of indexing per append.** It
  breaks NFR-15: a full rebuild of 10M events takes about five minutes.
- **Keep PEER-03 byte-identical but require the same key set.** Two
  nodes never share a key set, so the requirement would almost never
  apply.

## Consequences

- An invariant changed, so the SRS moves to a new major version,
  3.0-draft, and the release that carries it is a new major version.
- M1 keeps its search index per append and implements the canonical
  digest `cairn verify` compares. The spike's `logical_digest` is the
  prototype.
- M1's purge sets SQLite's `secure_delete` and FTS5's `secure-delete`,
  or rebuilds the index file, and checkpoints the write-ahead log.
- M1's schema may keep trust levels and origin apart from the parts
  every node shares, so PEER-03's shared parts compare directly.
