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
[keri]: https://trustoverip.github.io/kswg-keri-specification/
[ik]: ../../research/notes/identity-keys/identity-keys.md

## Decision

I10's list names the search index where it said "indexes" and adds origins. Its
last sentence becomes: "Rebuilding reproduces their content exactly, whatever
their storage layout." The rest of I10 is unchanged. Derived artifacts stay a
deterministic function of the writer logs a node holds and its own key set.
Arrival order still plays no part.

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
- **PRV-13, new.** Each range a node accepts from a peer, a paired phone,
  a bundle or a backup restore is recorded as an arrival event in its
  device seat in its personal room, once per way it arrived: writer id,
  seq range, digest and how it arrived. An event's origin derives from
  the record and the key set alone: `ingested` for a writer whose seat
  certificate names it an ingest seat, then `bundle` or `peer` from the
  arrival events, then `witnessed` for a writer whose seat key a key on
  the node's current key event log certified, else `peer`. No stored
  origin value, and no origin another node derived, decides it.
- **PRV-12, new, with SEC-10, REC-24 and PEER-06,** makes the node's
  device keys, or a token-key-only node's token keys, a key event log in
  which each key commits to its next key by digest, as KERI's
  pre-rotation does ([KERI][keri], [identity-keys research][ik]).
  - An inception starts a log: `minted`, `token key adopted`, or
    `minted anew` in place of a missing device key. A rotation,
    `rotated` or `recovered`, must reveal the committed next key, the
    recovery key, be signed by it, and name a freshly minted key.
  - The genesis log is taken on trust. Any later log certifies no
    `witnessed` writer until a `recovered` rotation joins it, the
    principal confirms it at the controlling terminal
    (`cairn device-key confirm`), or, for a token key, the same device
    key made its token certificate, which names the token key it
    succeeds.
  - A device key's recovery key leaves the node once, encrypted under a
    passphrase, through `cairn device-key write-recovery-key`, a
    widening principal act at the controlling terminal, under a key a
    memory-hard function derives from the passphrase. A token key's
    stays with the node that issued its access token until a later
    access token carries it.
  - A node identity change always starts a new log, so a clone never
    stays `witnessed`. A backup restore starts one in place of every
    earlier log, since the restored record reads `peer` anyway.
  - Voidness is decided first, along the hash links: a key event that
    breaks the rules counts for nothing, and so does an inception whose
    replaced key its own log has rotated, unless a `recovered` rotation
    names it.
  - Two newest key events in one log, or two inceptions replacing one
    key, are duplicity. Only key events this node recorded count, so a
    peer cannot cause it. Only `cairn device-key choose`, or a rotation
    that voids both inceptions, resolves it.
  - Each seat certificate gets a certificate record. Each rotation
    carries the chain head of the writer it follows, and a retired
    key's certificates count only at or before that head.
  - Revoking a key on the current log keeps the writers it certified
    before `witnessed`; its certificate records after the revocation
    count for nothing.
  - On a node identity change, Cairn may retry reading an unavailable
    identity value first. Otherwise the principal confirms the new log
    to bring its writers from `peer` to `witnessed`.
  - "Current" and "newest" come from the record's hash links, never
    from a clock, arrival order or the audit log.
- **PRV-13** also takes origin `ingested` from a transcript source and
  ingest position, stores each range only with its arrival event, takes
  the arrival event's digest over commitments and headers, never
  content, and refuses a range that extends one of this node's own
  writers.
- **PRV-11.** A seat certificate also names whether its seat is an
  ingest seat, which PRV-13 reads for origin `ingested`. PRV-11's I2
  security review is part of this record's.
- **REC-17, ADM-07 and §8.2.** A tombstone no longer keeps a purged
  event's origin; each node derives it from the writer's seat
  certificate, the arrival events, the key events and the node identity
  change events, none of which a purge covers.
- **PEER-03.** Two peers holding the same segments derive room state and
  a search index identical in content, apart from each event's trust
  level and origin. Each node derives those for itself and never takes
  them from another node.
- **LMK-05** gets back its bound on the landmark index, which a
  rewording dropped without a decision.

The domain model's derived artifact is its content, never its storage
layout, and two copies are the same when their canonical digest matches.

[invariants.md](../srs/invariants.md) holds the text approved here. Its
SHA-256 is:

```text
4204e0b5e13b9e59467c5ee7963d53fd06fdca3a0fcf18a64cd3a0f0fff81fd7
```

That text also holds the changes to I2 and I3 that
[ADR-2610112209](ADR-2610112209-invariants-after-s1-s3.md) proposes.
The two are approved in a fixed order: ADR-2610112209 first, on its
own text, then this record. This record approves only I10, and its hash
names the text in force once both are accepted. Until ADR-2610112209 is
accepted, this record cannot be.

The stakeholder, @jeduden, approves this record as security reviewer
(ENG-29) by a comment on the pull request that adds it. The comment
names the record's id, the hash above and the commit it approves. The
approval covers I10 and the requirements listed above as worded in that
commit. Weakening any of them reopens this record under ENG-29. On that
approval the record becomes accepted, and its hash replaces
ADR-2610112209's as the approved text, which in turn replaced
[ADR-2610091555](ADR-2610091555-initial-invariants.md)'s.

### Security review

A byte comparison never held for a store kept per append, so part of
what it promised was never real. What a content digest alone would
miss, and what answers each:

| A byte comparison would catch                                                   | What answers it now                                                                     |
| ------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| purged text or postings left in free pages or FTS5's index tables               | SEC-31's no-residue rule, checked on the decrypted raw files                            |
| residue from retention, applied erasure requests or a backup restore            | SEC-31 covers every way stored content is removed                                       |
| a plaintext copy in a temporary file, such as `VACUUM`'s                        | §8.1 keeps the storage engine's temporary storage in the home                           |
| a trigger, view or extra index, or a changed tokenizer or ranking configuration | the schema and settings are part of the digest; verify refuses unknown schema objects   |
| an index that disagrees with the table it indexes                               | ADM-08's integrity checks of the store and the search index                             |
| output whose order follows storage                                              | ADM-08: every read orders its output by values                                          |
| a stored origin changed from `peer` to `witnessed`, making the event trusted    | PRV-13 derives origin from the record; verify compares it with the rest of I10          |
| a node's own backup restored and read as `witnessed`                            | PRV-13 reads arrival events before the node's own writers                               |
| a purge of the events origin derives from, turning `peer` back into `witnessed` | ADM-07: no purge scope covers arrival, node identity change or key events               |
| a key event another process forges to make its own writer `witnessed`           | PRV-12: a rotation needs the recovery key, a later log a widening principal act         |
| a clone that keeps its source's writers `witnessed`                             | REC-24, PRV-12: an identity change starts a new, unconfirmed key event log              |
| two conflicting rotations of one key event log                                  | PRV-12: duplicity stops `witnessed` until the principal or the recovery key resolves it |

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
Each node derives trust levels and origin for itself, and never takes
either from another node.

Before this change, origin was a stored per-event value outside the
hash chain. Trust levels derive from it, so changing a peer event's
origin to `witnessed` would have made it trusted (I2), and nothing would
have noticed. PRV-13 closes that: origin now derives from the record and
the key set, as I10 lists it. Until seals ship (REC-18), a node's own
writers are anchored by head receipts alone. That limit is the same for
everything the record holds.

Without a committed next key, any process that could append a key event
could add its own key to the set that reads `witnessed`. With PRV-12, a
rotation needs the recovery key, which is off the node, and a new log
counts only once the principal confirms it at the controlling terminal
or the recovery key joins it. A process that only appends can still
start a new log. Every writer then reads `peer` until the principal
acts, which is a loud denial rather than silent trust: it is audited,
counted and shown, and a rotation with the recovery key voids it.

What stays open: `cairn device-key confirm` and `choose` are widening
principal acts, and in the record a principal act is an event like any
other. Where the principal or managed policy requires an authenticator,
each carries a presence proof bound to its digest (OWN-11), which an
appender cannot make. Where none is required, the controlling terminal
is checked only by the CLI, so a process that can write the home and
the device seat's key can forge a confirmation, as it can forge every
other widening act. This is the same OS user's boundary that I8 already
concedes, and no weaker than for any principal act. Also open is the
window before the recovery key leaves the node, which `cairn status`
shows.

## Alternatives

- **Keep byte identity and compare only rebuilt snapshots.** Verify
  would then compare one rebuild with another, which proves little, and
  the store would need a second index lifecycle.
- **Keep byte identity and rebuild instead of indexing per append.** It
  breaks NFR-15: a full rebuild of 10M events takes about five minutes.
- **Key events without a committed next key.** A transition rule between keys
  needs a signature only the node holds, so any process on it could
  forge one. It also needs an order of key events, and only the hash
  links give one without a clock.
- **Let any new log count at once, and only report it.** A forger's
  writers would read `witnessed` until someone noticed.
- **Heal duplicity by any new inception.** Anyone who can append could
  choose which branch wins.
- **Keep a key lost and minted anew `witnessed`.** Nothing proves the
  new key is the principal's, so a forged `minted anew` would make its
  writer trusted. The recovery key's `recovered` rotation proves it.
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
- M1 derives origin from the record, keeps the device key's key event
  log with its recovery key, and writes node identity change events. M3,
  M8 and M9, which bring backup restore, peers and paired phones, and
  bundles, write the arrival events they need. M8 carries a token key's
  next key in its access token.
