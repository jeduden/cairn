---
id: ADR-2610050528
title: "The implementation language: Rust for the core, TypeScript for the page"
status: proposed
summary: >-
  Every Cairn process is Rust, around one core library linked into the
  Tauri shell on all five platforms and into Cairn's server for the
  browser; the one UI page is TypeScript and holds no logic. It closes
  OQ-32's language half and replaces CON-01's Go.
---
# ADR-2610050528: The implementation language

## Context

OQ-32 in [§13](../srs/13-open-questions-and-risks.md) asks which
language fits the complete SRS. CON-01 names Go, written before the
stakeholder set the target that Cairn must meet: the same app on
Linux, Windows, macOS, iOS and Android, a browser when Cairn is
hosted, and a software factory of agents that builds it.

[ADR-2610042341](ADR-2610042341-app-shell-tauri.md) chose Tauri 2 as
the shell around one web page. That left two cores in the
[comparison][cmp]: a Rust core in process (option D) or a Go node
beside the shell (option C). With Tauri, Rust is in every build
already. A Go node adds a second process and a bridge on the desktop,
and no core reaches the phone, which could then only be a window onto
a node. The stakeholder chose Rust and TypeScript on 5 October 2026.

## Decision

- **Rust for every process.** The hooks, the MCP server, the CLI, the
  TUI, the kernel worker, the app's backend, the run component and the
  peer component are Rust, built around one core library. The library
  links into the Tauri shell on all five platforms, so a phone
  verifies, syncs and signs with the node's code. The same library
  runs in Cairn's own server for the browser.
- **TypeScript for the page only.** The one UI is a TypeScript page. It
  draws the view model the core computes (VIEW-04, VIEW-05, VIEW-14)
  and holds no logic of its own.
- **Memory safety (HC-23 in the [frame][frame]).** Every crate that
  parses content from outside the trusted boundary, or decides trust,
  carries `#![forbid(unsafe_code)]`. `unsafe` lives only in listed
  shims (SQLite, libghostty-vt, pty and platform calls), each with its
  fuzz target.
- **Reach evidence (SEC-01, SEC-19, ENG-16).** One crate per
  component, clippy's `disallowed-types` and `disallowed-methods` on
  `std::net` and `std::process` outside the components the register
  allows, and a symbol check of each shipped executable.
- **Two executables per desktop.** A static core executable runs as
  the hooks and the MCP server; the app executable carries the window.
  Both come in one download.
- **Vendored forks.** A crate Cairn changes or ports lives in the
  repository with its upstream version, a divergence list and an
  equivalence job, as mdsmith keeps goldmark.
- **Repository tooling may stay Go.** The gates under `internal/` and
  `cmd/review-gate` check the repository, not the product. They stay
  Go until a plan ports them, and drive the shipped binary as a black
  box through the CLI and MCP.

## Alternatives

- **A Go node beside the Tauri shell** (option C). It keeps CON-01,
  every gate and the fastest build loop. It costs a second process and
  a bridge on the desktop, three languages in the build, and no core on
  the phone without the experimental gomobile.
- **Zig in Ghostty's shape** (option H). The best terminal and
  cross-compilation, and smalt already pays for the tooling. But no
  compiler-checked temporal safety for long-lived components, a
  toolchain that breaks each release, and upstream bans LLM-written
  bug reports. Bun left Zig for Rust over memory bugs.
- **TypeScript on Bun** (option E). It fails HC-3, HC-13 and HC-22.
- **Kotlin Multiplatform** (option F). Weak core slots, and a JVM
  desktop shell.

## Consequences

- **SRS changes, each the stakeholder's,** made in SRS 2.1-draft.
  CON-01 names Rust and TypeScript, and the rows the [frame][frame]
  lists as worded for Go are restated: CON-02 (no cgo; adding Windows
  is a separate ruling), CON-03, ENG-01 to ENG-05, ENG-07 (cargo-fuzz),
  ENG-09 (data races excluded by the compiler, Miri or ThreadSanitizer
  for `unsafe`), ENG-11, ENG-16 (clippy, `cargo-deny`, `cargo-audit`,
  `forbid(unsafe_code)`), INJ-03 (a `pub(crate)` constructor), SEC-07,
  OPS-05, ADM-01, NFR-10, NFR-13, ADR-04, ADR-06, ADR-07, S2, S7,
  OQ-03 and OQ-32.
- **ADRs to supersede** once their successors are written:
  [ADR-2609292234](ADR-2609292234-test-stack.md) (godog and testify;
  the successor picks a Rust test stack or keeps godog driving the
  binary) and [ADR-2609302341](ADR-2609302341-sqlite-driver.md)
  (ncruces/go-sqlite3; the successor picks `rusqlite` with FTS5
  bundled). Spike S2's measurements of FTS5 at 10M events carry over.
- **Dependencies (ENG-18).** Rust's standard library has no HTTP, JSON
  or cryptography, so the direct crates pass the target of ten. The
  allow-list is enforced with `cargo-deny`, and needs widening for
  Zlib and Unicode-3.0, and a ruling on MPL-2.0. The page's npm
  packages count too (open point 8).
- **Build times.** Rust's incremental builds take seconds where Go's
  took less. The workspace splits into crates, uses ThinLTO only in
  release builds, and the bake-off's factory metrics track the loop.
- **The factory.** Agents write Rust and TypeScript fluently, and the
  compiler rejects memory and data-race errors before review.
- **The bake-off** becomes a proving slice: the Rust core in a Tauri
  app on one desktop and one phone, and in the browser.
- The status stays `proposed` until the stakeholder approves it and
  the SRS change lands.

[cmp]: ../../research/notes/implementation-path/comparison.md
[frame]: ../../research/notes/implementation-path/constraints.md
