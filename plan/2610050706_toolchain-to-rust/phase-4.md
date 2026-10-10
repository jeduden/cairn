---
n: 4
title: "The core executable in Rust; Go removed"
status: "✅"
result: false
---
# Phase 4: the core executable in Rust; Go removed

Requirements. None closes. ENG-01's scenario now checks the Rust
release flags; ENG-19 and SEC-01 stay pending, with the build-time
evidence below in place.

What lands:

- [x] `cairn version` as the Rust core executable, the `cairn` crate
  under `crates/`: the same command line as the Go stub, and
  `--json` adds the commit the build stamped.
- [x] The release build in one reusable workflow, `build.yml`, which
  CI runs on every change and the release workflow runs before it
  publishes. Two independent builders, an x86_64 and an arm64 Linux
  runner, cross-build `linux/amd64`, `linux/arm64` (musl, static) and
  `darwin/arm64` with zig as the linker through cargo-zigbuild, both
  pinned. A third job compares the builders bit for bit (ENG-19), and
  each binary runs once on its own platform. The plan named Linux and
  macOS builders. On macOS, rustc would find the runner's own Xcode
  SDK, which a Linux builder lacks, so the second builder is the arm64
  Linux runner and macOS only runs the binary.
- [x] ENG-01's steps check the Rust release flags:
  `--remap-path-prefix`, the stamped commit, and the musl targets.
- [x] Reach evidence for the core crate replaces the import-closure
  test (SEC-01): its own clippy.toml refuses the `std::net` socket
  types and `std::process::Command`, and a reach test refuses any
  crate linked into it. Each has a drift case.
- [x] `go.mod`, `go.sum`, `.golangci.yml`, `tools/`, `scripts/`'s Go
  coverage gate and `cmd/cairn` go; CON-01, ENG-01 and ENG-16 drop
  their Go clauses (SRS 2.32-draft). testify leaves with them;
  ADR-2610101913 supersedes ADR-2610101442.
- [x] The nightly job waits for cargo-fuzz targets instead of Go's.

Spike S2's Go benchmark stays in its plan folder as archived
evidence, as the plan's context says. No job builds it any more.

Gate: no Go source remains outside that archived spike, and CI runs
the release build the release workflow publishes.
