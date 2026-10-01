---
id: 2610012322
title: "Scope Cairn for agent fleets: shared, real-time, public sessions"
status: "🔲"
summary: >-
  Widen Cairn from one agent's local memory to secure session memory
  shared across worktrees, cloud sandboxes, machines and the public,
  in real time. Splits the record (per-origin log segments) from the
  index (SQLite), and ends in proposed SRS changes M1 builds on.
model: opus
depends-on: []
---
# Scope Cairn for agent fleets: shared, real-time, public sessions

## Goal

Decide what Cairn is for when many agents share a repository: in
separate worktrees, in ephemeral cloud sandboxes, on several machines,
in open source, and in real time. Turn the decision into SRS changes
the stakeholder approves before M1 builds the store.

## Context

For one agent on one machine, Claude can already grep its own
transcripts. There, Cairn mostly adds security: the envelope,
provenance, quarantine and redaction (I2, I5). It also keeps
constraints verbatim after compaction (I3). The harder problem is a
fleet, which is the stakeholder's own workflow:

- many agents on separate worktrees of one clone, which today hash to
  separate stores, because §8.1 keys a project by its path;
- Claude Code on the web, whose sandboxes are reclaimed, while §2.2
  assumes `CAIRN_HOME` on durable volumes;
- two or three more machines, which share nothing (OQ-07, post-v1);
- open-source contributors, where another person's history is
  untrusted by definition;
- all of it live, which no git-based exchange gives.

Shared memory between agents is a prompt-injection network unless
provenance and trust travel with every event. That makes the security
model the product rather than a feature.

Spike S2 ([plan 2609292004](../2609292004_spike-s2-sqlite-driver/plan.md))
chose the SQLite driver for a single-node store. It also showed
SQLite as the record cannot be merged, streamed or published: one
mutable file, a gap-free `seq` and one hash chain per store. The
candidate this plan tests keeps ADR-02's event sourcing but splits
it in two:

| Layer  | Form                                                                                         |
| ------ | -------------------------------------------------------------------------------------------- |
| Record | append-only, content-addressed, hash-chained log segments, one writer per origin             |
| Index  | local SQLite with FTS5, rebuilt from the segments a node holds (I10), on S2's ncruces driver |

Each origin (node and session) writes only its own log, so there are
no write conflicts and no CRDT. Sync, publishing and sandbox
persistence all become moving segments.

What was searched and weighed:

- Replicated SQLite. Litestream ships one writer to replicas only.
  libSQL embedded replicas put the network in the core (I4).
  cr-sqlite and dqlite or rqlite need CGO (CON-02) or a server
  (ADR-01). None was reused.
- Git as the record. It is a natural transport for reviewed segments
  and open source, but too slow and too coarse as the live store.
- The S2 harness under
  [bench](../2609292004_spike-s2-sqlite-driver/bench/main.go) is
  reused for the index side: its corpus, its FTS5 probes and its
  measurements.

The changes touch §1, §2.2, §4.5, §8 and the invariants, so they wait
for the stakeholder. A change that weakens I1–I10 needs a security
review and a new major version (CLAUDE.md). M1 blocks on this plan
because it builds the store's identity model.

## Tasks

1. Proving slice: a scope proposal, and a prototype that rebuilds the
   same index from two origins' segments in any order
2. The proposed SRS changes in their own pull request: goals and
   non-goals, deployment context, invariants restated for many nodes,
   the identity model, and new ADRs for the record/index split
3. Re-plan M1 against the approved identity model
4. Spike the relay: the one network component, outside the core,
   opt-in and separately reviewed, carrying segments in seconds
5. Spike public session bundles: stricter redaction and a review
   step on export, signing for authorship, and import as untrusted,
   pull-only content

## Execution

| Phase | Model | Gate                                                                                  |
| ----- | ----- | ------------------------------------------------------------------------------------- |
| 1     | opus  | The prototype's test rebuilds a byte-identical index from two origins in either order |

## Phases

<?catalog
glob:
  - "phase-*.md"
  - "phase-*.result.md"
sort: numeric:n
header: |

  | # | Status | Phase |
  |---|--------|-------|
row-expr: |
  [if result {
    "|  | ↳ | \(summary) |"
  }, if !result {
    "| \(n) | \(status) | [\(title)](phase-\(n).md) |"
  }][0]
footer: |

?>

| #   | Status | Phase                                                           |
| --- | ------ | --------------------------------------------------------------- |
| 1   | 🔲     | [Scope proposal and the record/index proving slice](phase-1.md) |
<?/catalog?>

## Acceptance Criteria

- [ ] The scope proposal names, for each fleet setting, what Cairn
  does, which invariant governs it, and what stays out of scope
- [ ] Two origins' segments, merged in either order, rebuild a
  byte-identical index, and foreign events stay untrusted
- [ ] The SRS changes are proposed in a separate pull request
- [ ] M1 is re-planned on the approved identity model
- [ ] All tests pass: `go test ./...`
- [ ] `mdsmith check .` is clean
