---
id: 2610022338
title: "Cairn's lane experience and peer network: no central service"
status: "🔲"
summary: >-
  Builds the live lane — a pull request with chat, real time,
  multiplayer and result views — over a peer network that needs no
  central service. Every node is a full peer, self-hosting is normal,
  and a partitioned lane keeps working and merges on reconnect. The
  experience comes first; making sync seamless follows.
model: opus
depends-on: [2610012322]
---
# Cairn's lane experience and peer network: no central service

## Goal

People and agents work a lane together live, from any machine or
sandbox, with no central service. Like git, every node holds the whole
lane and can serve it; a self-hosted node is the normal case, not an
upgrade. A lane split by a network partition keeps working on every
side and merges cleanly when the sides meet again.

## Context

The stakeholder's decisions (3 October 2026):

1. Cairn must not depend on a central service. Self-hosting is normal,
   as it is for git.
2. Multiplayer must work without a central server, because networks
   partition. CRDTs and other conflict-resolution methods handle the
   merge.
3. Design the user experience first. Making it seamless is the second
   step. Security must be good enough, measured against Zed Delta,
   which syncs through a central Cloudflare backend.

What merges without conflict already: plan 2610012322 gives each
writer its own append-only, signed log. The set of all writers' logs
is a grow-only set, so any two peers that exchange logs converge, in
any order, after any partition. Messages, tool runs, results and
approvals are events in those logs; their order across writers is
causal (each event names the heads it saw), not wall-clock.

What needs a CRDT: state that several participants edit at once. Two
cases are known:

- lane metadata (title, status, labels, assignment): small registers
  and sets;
- files edited live by several people and agents in one worktree, as
  Delta's CRDT worktrees do. Agents in separate worktrees, the common
  case, need none: their edits are events, and git merges code at
  landing.

The contract rules a network side out today. I4 says Cairn never talks
to the network. SEC-01 and CON-04 ban sockets in shipped binaries. NG4
and NG5 put shared stores and a graphical interface out of v1. Plan
2610012322's SRS change must decide first: narrow I4 to the core that
feeds the model, through a security review and a new major version,
or ship this as a separate product. Either way the core
keeps no socket, and everything a peer delivers reaches an agent only
as untrusted, pull-only data (I2).

What was searched, from [the research](../../research/README.md):

- Zed Delta: the benchmark for the experience; central, Cloudflare
  Durable Objects, no signing, no trust model for co-authors.
- Block Buzz: branch as room over a signed event log, but one relay
  holds the community.
- Radicle, NIP-34 and Tangled: peer-to-peer git collaboration with
  signed refs; Radicle's per-peer namespaces fit one log per writer.
- iroh, Hypercore and Willow: peer-to-peer transport and range-based
  reconciliation; candidates for the sync layer.
- Automerge, Loro and Yjs: CRDTs for the metadata and live co-editing
  cases; Go support decides between them (ENG-18).
- torchwood and C2SP tlog-tiles: the segment and checkpoint format.

A cloud sandbox can only dial out, so it needs some reachable peer.
That is any self-hosted node, including the user's own machine; no
node is special.

## Tasks

1. Proving slice: the lane experience over two peers, with a partition
   and a merge, and no server
2. Seamless sync: peer discovery, sandboxes behind outbound-only
   networks, background sync and offline queues
3. Live co-editing in one worktree, with a CRDT chosen by ADR
4. The merge gate on the lane: signed approvals, required checks and
   landing in git, working across partitions
5. Public lanes: export with stricter redaction and review, signed,
   served by any peer, imported as untrusted
6. Git as a carrier: one ref per writer for open-source projects

## Execution

| Phase | Model | Gate                                                                                                                               |
| ----- | ----- | ---------------------------------------------------------------------------------------------------------------------------------- |
| 1     | opus  | Two isolated peers, split and rejoined, render the same lane; the stakeholder walks through the lane view and signs off the design |

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

| #   | Status | Phase                                                              |
| --- | ------ | ------------------------------------------------------------------ |
| 1   | 🔲     | [The lane experience over two peers, with a partition](phase-1.md) |
<?/catalog?>

## Acceptance Criteria

- [ ] The stakeholder signs off the lane experience before sync is
  made seamless
- [ ] Two peers that never reach a server, split by a partition, both
  keep working and converge to the same lane after they reconnect
- [ ] A segment with a broken signature or chain is refused and
  audited, never imported (I6)
- [ ] The core that feeds the model holds no `net`, `net/http` or
  `os/exec` in its import closure
- [ ] Every new direct dependency has an accepted ADR (ENG-18)
- [ ] All tests pass: `go test ./...`
- [ ] `mdsmith check .` is clean
