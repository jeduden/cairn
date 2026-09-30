# Spike S2 benchmark report: the pure-Go SQLite driver

Status: in progress. The measurements below are recorded as they land.
The search, ingest, hook-memory and WAL results at 10M events follow.

## Setup

- Candidates: `modernc.org/sqlite` v1.60.1 and
  `github.com/ncruces/go-sqlite3` v0.35.6, both on SQLite 3.53.4.
- Harness: the separate module in [bench](bench/main.go), so neither
  driver enters cairn's module graph. It builds the same seeded
  synthetic store with each driver.
- Host: 4 vCPU, 15 GiB RAM, Linux, Go 1.26.8, `CGO_ENABLED=0`.

## Correctness

| Check                               | modernc | ncruces                                    |
| ----------------------------------- | ------- | ------------------------------------------ |
| FTS5 compiled in                    | yes     | separate module, registered per connection |
| BM25, snippet, phrase, prefix, NEAR | pass    | pass                                       |
| unicode61, trigram, porter          | pass    | pass                                       |
| rebuild FTS from the record (I10)   | pass    | pass                                       |

## Import closure (SEC-01, I4)

| Driver  | Forbidden packages linked        | Via                                               |
| ------- | -------------------------------- | ------------------------------------------------- |
| modernc | net, net/netip, net/url, os/exec | libc (`system()`, `popen()`), google/uuid, sqlite |
| ncruces | net/netip, net/url               | URI parsing in the driver and its VFS             |

## Store size at 10M events

Both drivers write byte-identical content: 4.61 GB of text, 9.01 GB of
file, a ratio of 1.96 against NFR-09's 1.5.
