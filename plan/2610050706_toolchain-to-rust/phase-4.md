---
n: 4
title: "The core executable in Rust; Go removed"
status: "🔲"
result: false
---
# Phase 4: the core executable in Rust; Go removed

Requirements. None closes.

What lands:

- `cairn version` as the Rust core executable; the release workflow
  builds it reproducibly on two builders (ENG-19) and ENG-01's steps
  check the Rust release flags.
- Reach evidence per crate replaces the import-closure test (SEC-01).
- `go.mod`, `go.sum`, `.golangci.yml`, `tools/` and the last Go
  sources go; CON-01, ENG-01 and ENG-16 drop their Go clauses.

Gate: no Go source remains, and the release workflow publishes the
Rust executable.
