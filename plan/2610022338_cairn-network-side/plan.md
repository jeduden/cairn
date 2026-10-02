---
id: 2610022338
title: "Build Cairn's network side: sync, relay, lane server and public host"
status: "🔲"
summary: >-
  Builds the processes that move lane records between machines and
  people: a sync agent per host, a relay for real-time fan-out, a lane
  server with live chat and result views, and a public host. Each runs apart from the core binary, so
  the core still never opens a socket (I4), and everything they carry
  reaches the model only as untrusted, pull-only recall (I2).
model: sonnet
depends-on: [2610012322]
---
# Build Cairn's network side: sync, relay, lane server and public host

## Goal

Agents on many machines, in cloud sandboxes and across contributors
share session records in seconds. The components that carry them are
built and run by this project. The core binary that holds the record
and feeds the model stays network-free.

## Context

Plan 2610012322 scopes Cairn for agent fleets: one signed, hash-chained
log segment per writer, and an index any node rebuilds from them. It
left the relay and public bundles as unplanned spikes. Real-time
sharing across machines needs a server, so this plan builds one rather
than leaving it to a third party.

The contract rules these components out today. I4 says Cairn never
talks to the network, SEC-01 bans sockets in every shipped binary,
CON-04 bans network access in any binary with P0 features, and NG4 and
NG5 put shared stores and a graphical interface out of v1. Building
them therefore needs one of two decisions, made in plan 2610012322's
SRS change: narrow I4 to the core that feeds the model, through a
security review and a new major version, or ship the network side as
a separate product with its own contract. Either way the core keeps
no socket. The network components are separate binaries:

- `cairn-sync`, one per host: pushes the host's sealed segments and
  pulls others' into the local inbox the core imports from;
- `cairn-relay`, the server: stores and fans out signed segments and
  checkpoints, and never sees a plaintext payload when the origin
  encrypts;
- a lane server: live chat, presence and result views for each lane,
  behind a web interface; the live pull request;
- a public host: serves reviewed, signed session bundles read-only.

A compromised network component can only deliver bytes. The core
verifies every segment against its origin's key and chain, and treats
every foreign event as untrusted (I2). The import-closure test in
[cmd/cairn/imports_test.go](../../cmd/cairn/imports_test.go) keeps
`net`, `net/http` and `os/exec` out of the core; the new binaries get
their own closure, reviewed separately.

What was searched, from
[the research](../../research/README.md) on agent session storage:

- filippo.io/torchwood (Go): tlog-tiles, signed notes, a witness and
  litebastion. The segment and checkpoint format should follow C2SP
  tlog-tiles, so torchwood is the first candidate to reuse.
- Trillian Tessera (Go): a tile-log library for the relay's storage.
- Durable Streams, S2 and NATS JetStream: hosted or heavyweight stream
  servers; their append-with-offset rule is borrowed, not the service.
- iroh and Hypercore: peer-to-peer replication, kept as a later
  carrier, since sandboxes need an outbound-only path to one server.
- Git as a carrier: one ref per writer stays an option for open source,
  built after the relay.

The SRS change from plan 2610012322 must first name these components,
their trust boundary and their own invariants. Each new direct
dependency needs an ADR (ENG-18).

## Tasks

1. Proving slice: two hosts exchange a segment through a relay in
   seconds, and a tampered segment is refused
2. Harden the relay: authentication per origin, quotas, retention,
   and encrypted payloads it cannot read
3. Run `cairn-sync` alongside the hooks: start, back-off, offline
   queue, and a status the core reports without a socket
4. Public host: export with stricter redaction and a review step,
   signed bundles, read-only serving, import as untrusted
5. Git as a second carrier: one ref per writer for open-source
   projects, after the relay is proven
6. Lane server and web interface: live chat in a lane, presence,
   several agents and humans per lane, and interactive result views
   for test runs, benchmarks and diffs. Messages from anyone but the
   lane owner reach agents as untrusted, enveloped data

## Execution

| Phase | Model | Gate                                                                                                                              |
| ----- | ----- | --------------------------------------------------------------------------------------------------------------------------------- |
| 1     | opus  | Two isolated CAIRN_HOMEs exchange a segment through a local relay in under 5 s, a tampered segment is refused, core closure holds |

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

| #   | Status | Phase                                                      |
| --- | ------ | ---------------------------------------------------------- |
| 1   | 🔲     | [Two hosts exchange a segment through a relay](phase-1.md) |
<?/catalog?>

## Acceptance Criteria

- [ ] A segment sealed on one host is imported on another within 5 s
  through the relay, with its origin, chain and trust intact
- [ ] A segment with a broken signature or chain is refused and
  audited, never imported (I6)
- [ ] The core's import closure still holds no `net`, `net/http` or
  `os/exec`
- [ ] Every new direct dependency has an accepted ADR (ENG-18)
- [ ] All tests pass: `go test ./...`
- [ ] `mdsmith check .` is clean
