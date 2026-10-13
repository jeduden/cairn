---
id: 2609292011
title: "The remaining engineering gates in CI"
status: "🔳"
summary: >-
  Dependency directions between crates, deadlines on blocking calls,
  the isolation guard, the confined end-to-end suite, the benchmark
  gate, SLSA level 3 provenance, branch protection and per-component
  reach evidence: the ENG P0 gates M1 exits on.
model: sonnet
depends-on: []
---
# The remaining engineering gates in CI

## Goal

Turn the engineering requirements that are CI gates rather than product
behavior into running checks, so M1 can exit on "CI gates green".

## Context

The toolchain moved from Go to Rust in plan 2610050706, so this plan
follows the intent of its Go tasks with the Rust tools that replaced
them. CI already gates on:

- `cargo fmt --check` and `cargo clippy` with warnings denied
- `cargo-deny` (licenses, bans, sources) and `cargo-audit`
- the workspace lint `unsafe_code = "forbid"`
- each product crate's `clippy.toml`, which bans sockets, program
  starts, clocks and hash collections, held to every crate under
  `crates/` by the gate tests `the_product_crates_keep_their_bans`
  and `no_product_crate_keeps_mutable_global_state`
- the core's reach test, `crates/cairn/tests/reach.rs`, which refuses
  any crate linked into it
- the testkit's `isolate` and the scenario runner's `is_isolated`,
  which point every process a test starts at a temporary home
- a release built through the reusable `build.yml`, with an SPDX SBOM,
  attested provenance and keyless cosign over the checksums

What the Go plan named and Rust replaces:

| Go plan               | Rust                                                       |
| --------------------- | ---------------------------------------------------------- |
| depguard import rules | declared crate directions, checked against every manifest  |
| custom analyzers      | clippy `disallowed-*` lists and gate tests over the source |
| `go test -race ./...` | `cargo test --workspace`                                   |
| `golangci-lint run`   | `cargo clippy` with warnings denied, `cargo fmt --check`   |
| govulncheck           | `cargo audit`                                              |

Two scenarios are Scenario Outlines whose rows name components that do
not exist until M7 to M9: ENG-12 and ENG-16 (§12.2 lists both as
changed in M7). A tag on any Examples block pends the whole outline,
so those two stay `@pending` here. This plan builds and proves their
core rows and leaves the bindings for the other rows to the plans that
build those components. ENG-16's restore-content check waits on
TrustedText from plan 2609292013.

## Tasks

1. The isolation guard that aborts a test touching the real home, and
   the check on agent instruction files (ENG-14)
2. Declare the allowed dependency directions between workspace crates
   and one responsibility per library crate, and enforce them (ENG-02)
3. A check that every public blocking operation takes a deadline,
   beside the existing global-state and clock bans (ENG-03)
4. The benchmark harness and its significance-tested 10% regression
   gate in CI (ENG-15)
5. SLSA Build Level 3 provenance from the reusable build workflow, and
   a release check that verifies what was published (ENG-20)
6. Main's protection as a committed ruleset checked against the live
   one, and CODEOWNERS checked against the paths that enforce the SRS
   (ENG-21)
7. The end-to-end suite runs the core with all network denied and
   fails on any socket (ENG-12, core row)
8. CI gates on the core's reach evidence and on unsafe code in every
   crate (ENG-16, core row)

## Execution

Phases 1 to 8 run in parallel, in five teams: 1; 2 and 3; 4; 5 and 6;
7 and 8.

| Phase | Model  | Gate                                                                     |
| ----- | ------ | ------------------------------------------------------------------------ |
| 1     | sonnet | `@ENG-14` passes; drift cases for the guard and the instruction check    |
| 2     | sonnet | `@ENG-02` passes; a forbidden direction is a caught drift                |
| 3     | sonnet | `@ENG-03` passes; a deadline-less blocking call is a caught drift        |
| 4     | opus   | the gate's decision is unit-proven and its job runs in CI                |
| 5     | sonnet | `@ENG-20` passes; provenance signed by the reusable workflow             |
| 6     | sonnet | `@ENG-21` passes against the committed ruleset and CODEOWNERS            |
| 7     | opus   | a socket-opening probe fails the confined suite; `@ENG-12` stays pending |
| 8     | sonnet | the core row's steps pass; `@ENG-16` stays pending on M7 and TrustedText |

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

| #   | Status | Phase                                                                                                                                                                                                   |
| --- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | 🔲     | [The isolation guard and the agent instruction check (ENG-14)](phase-1.md)                                                                                                                              |
| 2   | 🔲     | [Declared dependency directions between crates (ENG-02)](phase-2.md)                                                                                                                                    |
| 3   | 🔲     | [Deadlines on every blocking operation (ENG-03)](phase-3.md)                                                                                                                                            |
| 4   | 🔲     | [The benchmark gate (ENG-15)](phase-4.md)                                                                                                                                                               |
| 5   | 🔲     | [SLSA Build Level 3 provenance (ENG-20)](phase-5.md)                                                                                                                                                    |
| 6   | 🔲     | [Main's protection and CODEOWNERS (ENG-21)](phase-6.md)                                                                                                                                                 |
| 7   | 🔳     | [The core confined in the end-to-end suite (ENG-12)](phase-7.md)                                                                                                                                        |
|     | ↳      | The core's end-to-end tests run it with all network denied, and a socket probe fails the suite naming the core and the caller; the CI wiring awaits its first run on the hosted runner.                 |
| 8   | ✅     | [The core's reach evidence and unsafe code (ENG-16)](phase-8.md)                                                                                                                                        |
|     | ↳      | Gate tests prove the core row: CI gates on the static checks under the ci job, every crate keeps the unsafe ban, the core's reach evidence and the determinism bans are in place; ENG-16 stays pending. |
<?/catalog?>

## Acceptance Criteria

- [ ] Scenarios @ENG-02, @ENG-03, @ENG-14, @ENG-20 and @ENG-21 pass,
  off `@pending`
- [ ] @ENG-15 passes once benchmarks cover NFR-01..05; until then its
  gate is unit-proven and runs in CI
- [ ] The core rows of @ENG-12 and @ENG-16 are proven; both outlines
  stay `@pending` until M7 to M9 build their other components and plan
  2609292013 defines TrustedText
- [ ] Every new check has a drift case that proves it fails (ENG-27)
- [ ] All tests pass: `cargo test --workspace`
- [ ] `cargo clippy --workspace --all-targets --locked -- -D warnings`
  and `cargo fmt --all --check` are clean
- [ ] `mdsmith check .` is clean
