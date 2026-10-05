---
id: 2610050706
title: "Move the whole toolchain from Go to Rust"
status: "🔲"
summary: >-
  Port everything the repository runs from Go to Rust: the scenario
  runner (cucumber-rs in place of godog), the gates that keep the SRS,
  scenarios, ADRs and reviews in step, the drift harness, the review
  gate, and CI, release and coverage. Then remove Go. Each gate runs
  in both languages until the drift suite shows they agree.
model: sonnet
depends-on: []
---
# Move the whole toolchain from Go to Rust

## Goal

Every tool the repository builds and runs is Rust. That holds for the
product and for the gates that check it. The page's TypeScript is the
only other language, as ADR-2610050528 and SRS 2.1-draft set out.

## Context

[ADR-2610050528](../../docs/adr/ADR-2610050528-language-rust-typescript.md)
made the product Rust and first let repository tooling stay Go. On 5
October 2026 the stakeholder chose to move the whole toolchain. SRS
2.1-draft still lets tooling stay Go in CON-01, ENG-01 and ENG-16;
those clauses go when Go does.

The Go code today is about 4,800 lines:

| Go code                                      | What it does                                                          |
| -------------------------------------------- | --------------------------------------------------------------------- |
| `cmd/cairn/main.go`                          | the `cairn version` CLI, the only product code so far                 |
| `cmd/cairn/bdd_test.go`                      | runs every scenario through godog, `@pending` skipped, isolated HOME  |
| `cmd/cairn/bdd_engineering*_test.go`         | step bindings for the six bound scenarios: ENG-01, 18, 24, 26, 27, 28 |
| `cmd/cairn/imports_test.go`                  | the import-closure half of SEC-01                                     |
| `internal/srs`                               | parses the SRS tables, Appendix B and C, the personas                 |
| `internal/scenario`                          | parses features and checks the SRS and scenarios agree                |
| `internal/adr`                               | reads ADRs for ENG-18 and ENG-26                                      |
| `internal/drift`                             | injects each registered drift and requires its check to fail (ENG-27) |
| `internal/review`, `cmd/review-gate`         | the reviewer app's gate (ENG-28)                                      |
| `scripts/check-coverage.sh`, `.golangci.yml` | the 100% coverage gate and the linters                                |

Most of these paths are the stakeholder's under
[CODEOWNERS](../../.github/CODEOWNERS), as are the CI workflows, so
every phase needs the stakeholder's approval. The Rust replacements
join CODEOWNERS in the phase that adds them.

Reuse, searched first:

- **The scenario runner.** The `cucumber` crate (0.23, MIT OR
  Apache-2.0) runs Gherkin with step macros. A prototype in the
  session that wrote this plan showed the shape that replaces godog:
  a `[[test]]` target with `harness = false`;
  `filter_run_and_exit` skipping `@pending`; `fail_on_skipped`, which
  turns a step with no definition into a failure (exit 101), as
  godog's strict mode does; and `--tags @REC-01` to run one scenario,
  in place of `-run 'TestFeatures/^REC-01:'`.
- **The feature parser.** The `gherkin` crate (0.16, MIT OR
  Apache-2.0) parses the features for the scenario gate.
- **Kept as external tools.** mdsmith and frit are pinned binaries,
  not repository code; they stay.
- **Not reused.** Keeping godog to drive a Rust binary as a black box
  would keep Go in the repository, which the stakeholder ruled out.

The port follows the vendored-fork discipline of the research's
[options](../../research/notes/implementation-path/options.md). While
both exist, the Go and the Rust gate run side by side. A Go gate is
deleted only after the drift suite shows each registered drift caught
by both.

## Tasks

1. Proving slice: a Cargo workspace, a pinned toolchain, the cucumber
   runner and the scenario gate in Rust, beside Go
2. Port the SRS parser's remaining checks (Appendix B and C, personas)
   and the ADR reader, with the ENG-18 and ENG-26 bindings reading
   `Cargo.toml` as well as `go.mod`
3. Port the drift harness and its cases, and run the drift suite
   against both languages
4. Port the review gate and its workflow (ENG-28)
5. Port `cairn version` to the Rust core executable; the release
   workflow builds it; ENG-01's steps check the Rust toolchain;
   reach evidence per crate replaces the import-closure test (SEC-01)
6. CI on Rust: clippy, `cargo fmt`, `cargo-deny`, `cargo-audit`, and
   coverage through `cargo-llvm-cov` in place of
   `scripts/check-coverage.sh`
7. Remove Go: `go.mod`, `go.sum`, the Go sources, `.golangci.yml`;
   supersede the test-stack ADR; drop the Go clauses from CON-01,
   ENG-01 and ENG-16; rewrite CLAUDE.md's code style,
   `docs/development.md` and DEPENDENCIES.md's introduction

## Execution

| Phase | Model | Gate                                                                                                                                              |
| ----- | ----- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1     | opus  | `cargo test --workspace` and `go test ./...` pass; the Rust gate catches the priority drift; each non-pending scenario runs in exactly one runner |

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

| #   | Status | Phase                                                                    |
| --- | ------ | ------------------------------------------------------------------------ |
| 1   | 🔲     | [Proving slice: the Rust scenario runner and gate beside Go](phase-1.md) |
<?/catalog?>

## Acceptance Criteria

- [ ] `cargo test --workspace` runs every non-pending scenario through
  cucumber-rs, with `@pending` skipped and undefined steps failing
- [ ] Every gate that Go runs today runs in Rust, and the drift suite
  shows each registered drift caught by the Rust gate
- [ ] The release workflow builds the Rust core executable, and
  ENG-01's scenario checks the Rust toolchain and release flags
- [ ] CI gates on clippy, `cargo fmt`, `cargo-deny`, `cargo-audit`
  and the coverage floors
- [ ] No Go source, `go.mod` or `go.sum` remains, and CON-01, ENG-01
  and ENG-16 no longer mention Go
- [ ] `mdsmith check .` passes
