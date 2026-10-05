---
n: 1
title: "Proving slice: the Rust scenario runner and gate beside Go"
status: "🔲"
result: false
---
# Phase 1: the Rust scenario runner and gate beside Go

Requirements. None closes. ENG-01 and ENG-24 keep their scenarios
bound; this phase binds them in Rust as well. No scenario leaves
`@pending`.

BDD coverage: the phase moves the runner itself, so its gate is the
runner running the existing scenarios.

What lands:

- A Cargo workspace at the repository root, with `rust-toolchain.toml`
  naming an exact Rust release.
- A crate for the SRS and feature parsers and the scenario gate, the
  Rust port of `internal/srs`'s table parser and `internal/scenario`.
- A `bdd` test target with `harness = false`, running `features/`
  through cucumber-rs. Each scenario's world points `HOME` and
  `CAIRN_HOME` at a fresh temporary directory (ENG-14). `@pending`
  scenarios are skipped and counted; a step with no definition fails.
- Rust bindings for ENG-01's and ENG-24's steps, the two simplest
  bound scenarios. Their Go bindings stay.
- One list of the scenarios the Rust runner owns. The Go runner skips
  them, and a check fails unless every non-pending scenario runs in
  exactly one runner.
- An ADR for the new direct crates (`cucumber`, `gherkin`, and any
  helper), and the ENG-18 checks extended so `Cargo.toml` counts.
- CI runs `cargo test --workspace` beside `go test ./...`.

RED: the Rust scenario gate's test, run against the repository's SRS
and features, fails until the parser and gate exist. A second test
injects the priority drift the Go suite registers (ENG-25 tagged
`@P0`) and requires the Rust gate to report it.

GREEN sites: the parser crate, the gate, the `bdd` target and its
world, the two bindings, the runner split and its check.

Gate: `cargo test --workspace` and `go test ./...` both pass. The
Rust gate reports the injected priority drift with the same message
as the Go gate. Running `cargo test --test bdd -- --tags @ENG-24`
runs exactly that scenario.
