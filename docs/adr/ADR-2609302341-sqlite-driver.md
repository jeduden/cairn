---
id: ADR-2609302341
title: "The SQLite driver: ncruces/go-sqlite3"
status: proposed
scope: dependencies
summary: >-
  The store opens SQLite through github.com/ncruces/go-sqlite3, a
  cgo-free wasm2go translation of SQLite. It links no network or
  process package, and it ships encrypting VFSes. Spike S2 measured it
  against modernc.org/sqlite at 10M events (ADR-07, OQ-03, OQ-04).
---
# ADR-2609302341: The SQLite driver

## Context

ADR-07 in [§4.5](../srs/04-reference-architecture.md) leaves the
driver open between `modernc.org/sqlite` and
`github.com/ncruces/go-sqlite3` until spike S2. The driver has to
build a static binary with `CGO_ENABLED=0` (CON-02). It needs FTS5
with BM25 (ADR-05), query cancellation by context deadline (ENG-03),
and WAL writes from 50 concurrent processes (ADR-01, NFR-05). It must
meet NFR-03, NFR-04 and NFR-09 at 10M events. It may not link a
network or process package (SEC-01, I4). An encrypting VFS weighs in
its favour (OQ-04).

The measurements are in the spike's
[benchmark report](../../plan/2609292004_spike-s2-sqlite-driver/report.md).

## Decision

| Module                          | Purpose                                                                                  | License | Maintenance                                                         |
| ------------------------------- | ---------------------------------------------------------------------------------------- | ------- | ------------------------------------------------------------------- |
| `github.com/ncruces/go-sqlite3` | The SQLite driver behind the store: record, projections and FTS5 recall (ADR-01, ADR-05) | MIT     | Active; frequent releases tracking SQLite; one principal maintainer |

## Alternatives

- `modernc.org/sqlite`, the other candidate. It is 15–25% faster at
  search and ingest, a gap that changes no target's outcome. But its
  C runtime, `modernc.org/libc`, links `os/exec` to emulate
  `system()` and `popen()`, and it pulls in `net` through
  `github.com/google/uuid`. Admitting it means allow-listing both in
  the SEC-01 import-closure test, for code Cairn never calls. It also
  has no writable VFS API, so it offers no encryption at rest.
- CGO `mattn/go-sqlite3`, already rejected by ADR-07. It breaks
  CON-02.
- A CGO build for large deployments (OQ-03). Not needed. Pure Go
  meets ingest, expand and hook memory at 10M events. The two misses
  come from FTS5 and the schema, and a CGO build shares both: ranked
  search on common terms, and store overhead.

## Consequences

- One new direct dependency against ENG-18's target of ten. Its own
  requirements, `go-sqlite3-wasm/v6` (MIT-0 over public-domain
  SQLite), `julianday` and `golang.org/x/sys`, stay indirect. Importing
  an encrypting VFS adds `golang.org/x/crypto` as an indirect
  dependency.
- The driver parses `file:` URIs with `net/url`, which imports
  `net/netip`. Neither opens a socket. The import-closure test in
  [cmd/cairn/imports_test.go](../../cmd/cairn/imports_test.go)
  bans the whole `net/` prefix, so it has to allow-list these two
  parsing packages, under review, in the change that first links the
  store into `cairn`. `net` and `os/exec` stay banned.
- FTS5 is a separate translated module registered on each
  connection (`ext/fts5`). The store has to register it before it
  opens a connection.
- An interrupted query returns `sqlite3: interrupted`, not an error
  that wraps `context.DeadlineExceeded`. The store maps interrupts to
  its typed deadline error by checking `ctx.Err()` (ENG-04).
- The status stays `proposed` until the M1 change that adds the
  module to `go.mod`. That change flips it to `accepted`, because
  ENG-18 fails on an accepted ADR naming a module `go.mod` does not
  require.
