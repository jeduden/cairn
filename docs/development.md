---
summary: >-
  Build and test commands, the executable requirement matrix (every SRS
  id has one tagged Gherkin scenario, pending until written), the
  coverage floors, and how CI, the nightly fuzz job and the
  reproducible, signed release pipeline work.
---
# Development

Build, test and release reference, plus the mechanics behind the
rules in [CLAUDE.md](../CLAUDE.md).

## Build & test commands

The repository tooling is a Cargo workspace under `tooling/`. Its
Rust release is fixed in [rust-toolchain.toml](../rust-toolchain.toml)
(ENG-01), and rustup installs it on the first `cargo` command. Until
phase 4 of plan
[2610050706](../plan/2610050706_toolchain-to-rust/plan.md), the
`cairn version` stub in `cmd/cairn` stays Go 1.26, fixed by `go.mod`.

- `cargo test --workspace` — every test, scenarios included
- `cargo test -p scenario --test bdd` — every scenario, pending ones
  counted
- `cargo test -p scenario --test bdd -- --tags @REC-03` — one scenario
  by its requirement id
- `cargo test -p srs --test gates` and `cargo test -p scenario --test
  gate` — the gates alone
- `cargo run -p coverage` — line coverage per test layer and per crate,
  held to the floors (see Coverage)
- `cargo fmt --all --check` and `cargo clippy --workspace --all-targets
  -- -D warnings` — layout and lint (ENG-16)
- `cargo deny check licenses bans sources` and `cargo audit` — the
  supply chain (ENG-16), with [deny.toml](../deny.toml)'s policy
- `go test -race ./...` and `scripts/check-coverage.sh 100
  ./cmd/cairn` — the Go that remains, at its 100% floor
- `go tool -modfile=tools/go.mod golangci-lint run` and `go tool
  -modfile=tools/go.mod govulncheck ./...` — lint and known
  vulnerabilities for that Go
- `mdsmith check .` — lint the Markdown; `mdsmith fix .` rewrites
  what it can, including the generated catalogs

## The executable requirement matrix

The SRS under [docs/srs](srs/index.md) is executable. Every
requirement row, from `REC-01` to `ENG-29`, and every
assumption `ASM-` row, has exactly one Gherkin scenario under
`features/`, one file per SRS section. The scenario carries the row's
id, its priority and its traced invariants as tags:

```gherkin
@REC-03 @P0 @I1 @I10 @pending
Scenario: re-ingesting a transcript source creates no duplicate events
```

The `scenario` crate's gate test, `specification_and_features_agree`,
keeps the two sides in step. A requirement with no scenario fails it.
So does a scenario naming no requirement, an id tagged twice, or a
priority or invariant tag that disagrees with the row. The `srs`
crate's gates check that Appendix B's invariant coverage and
requirement counts match the tables they summarise. They also check
that Appendix C gives every requirement a persona, and that §2.5's
personas match the agents in `.claude/agents`.

A scenario still tagged `@pending` is declared but not written. It
is skipped and counted, never counted as a pass. The `bdd` target in
[tooling/scenario](../tooling/scenario/tests/bdd/main.rs) runs the rest
through cucumber-rs, and a step whose text matches no binding fails
instead of being skipped. It prints `scenarios: N bound, M pending`
first, and CI's test job writes that line into its summary.

The runner reads features with the same gherkin parser as the gate.
That parser is stricter than godog's in two places. A description line
that opens with a Gherkin keyword, such as "Scenarios", reads as that
keyword. And `<name>` in a Scenario Outline must name an Examples
column, so a literal placeholder is written another way, such as
`{id}`.

### Proving the checks: drift injection

A check that keeps the SRS, the scenarios and the records in step can
be weakened as easily as any other code, and a weakened check turns
CI greener, not redder. ENG-27 closes that hole. Every such check has
a registered drift case in
[tooling/drift/src/cases.rs](../tooling/drift/src/cases.rs). A case is
one injection into a copy of the repository, the check to run and the
message it must fail with.

- `cargo test -p drift --test suite -- --ignored` — the drift suite.
  It proves the unedited copy passes every check, then injects each
  case into a fresh copy and requires its check to fail. It needs
  mdsmith on PATH; CI's `drift` job runs it.
- A case the checks miss today is marked `known_gap`. The suite fails
  the day a check starts catching it, so the mark goes with the fix.
- The ENG-27 scenario runs in plain `cargo test --workspace`. It fails
  when a case's injection no longer finds its target, or when a
  non-pending scenario that opens on "the repository checkout" has no
  case guarding its id.

A new check lands with its drift case in the same change. A scenario
put back on `@pending` skips, so its drift cases go uncaught and the
suite fails: re-pending shows up here too.

### Adding or writing a scenario

A new requirement lands in the SRS table and in its section's feature
file in the same change, tagged `@pending`. To write a scenario, drop
`@pending`, make its Given/When/Then concrete, and bind the steps. A
section's bindings live in `tooling/scenario/tests/bdd/<section>.rs`,
the way [engineering.rs](../tooling/scenario/tests/bdd/engineering.rs)
does. Bindings register themselves, so a section adds one module and
one `mod` line in `main.rs`.

cucumber-rs matches a step by its keyword: a `#[given]` binding never
matches a `Then` line, and `And` or `But` take the keyword before them.
A step text two sections both bind fails as ambiguous, so reuse the
existing text. Keep a binding thin: the logic it calls lives in a
library crate with unit tests, as the `engineering` crate holds §10's.

Every step binds on the one `World` a scenario threads. The runner
first runs itself again with `HOME` and `CAIRN_HOME` in a fresh
temporary directory. Each world gets its own fresh home as well, for
the processes its steps start (ENG-14). State a section tracks lives
in its own struct, reached through `World::section::<T>()`, never as a
new field on the world.

## Coverage

Coverage is measured per test layer, so the test pyramid is visible
in every run rather than one blended number:

- **unit**: the tests beside the code, in each crate's `src/`, in the
  `tests.rs` beside the module they test;
- **integration**: a crate's `tests/` targets, which drive its public
  API against the real repository or the file system;
- **end-to-end**: the `bdd` scenario runner and every `tests/e2e*`
  target, which drive the system through its entry points, a built
  executable run as a process among them. ENG-12's confined suite and
  ENG-17's live test join this layer when they land.

`cargo run -p coverage` runs
[cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) once per
test layer, over one build. It prints line coverage per crate for
each layer and for all of them. It writes each layer's lcov to
`target/coverage/`. Install the tool first, with `cargo install
--locked cargo-llvm-cov@0.9.1`.

The floors live in [Cargo.toml](../Cargo.toml) under
`[workspace.metadata.coverage]`, and the tool fails below them:

- every test layer together: 100% of each tooling crate's lines;
- the unit test layer alone: 90%, so most lines are proven by unit
  tests, not only by scenarios. A crate whose process or file boundary
  belongs to its integration tests may name a lower unit floor of its
  own, with the reason beside it.

ENG-11's floors for the product crates (90% for the security-sensitive
ones, 80% overall) join the same table as those crates land.

`tests.rs` files are left out of the measure, so the numbers count
shipped code only. A test that runs a built executable as a process
covers its `main`, so no line needs an exclusion. CI's `coverage` job
writes the table into its summary and uploads one Codecov flag per
test layer; Codecov's thresholds in [codecov.yml](../codecov.yml) are
a soft ratchet on top.

The Go stub keeps its own floor of 100%, which
[scripts/check-coverage.sh](../scripts/check-coverage.sh) enforces.
[scripts/coverage-exclude.txt](../scripts/coverage-exclude.txt) names
its one exclusion: `main()`, a pure process boundary.

## Static analysis

The workspace's lints in [Cargo.toml](../Cargo.toml) forbid `unsafe`,
and refuse `unwrap`, `expect` and `panic!` outside tests
([clippy.toml](../clippy.toml) lets a test use them). CI runs clippy
with every warning denied, and rustfmt.
[deny.toml](../deny.toml) holds every crate in the tree to ENG-18's
license allow-list, each exception named, and to crates.io as the
only source. cargo-audit refuses a crate with a known vulnerability.

For the Go stub, [.golangci.yml](../.golangci.yml) enables
staticcheck, gosec, errcheck and the `go vet` suite. It adds
`depguard`, which refuses `net`, `net/http` and `os/exec` in
non-test code, and `gochecknoglobals` for ENG-03's
no-mutable-package-state rule.
[cmd/cairn/imports_test.go](../cmd/cairn/imports_test.go) walks the
whole import closure of the shipped binary, so the same packages
cannot arrive through a dependency either. Both are the import half
of SEC-01; the network-deny sandbox half is ENG-12, still pending.

## CI, nightly and release

Every workflow lives in `.github/workflows`. Every action is fixed
to a commit SHA with its version in a trailing comment, and dependabot
proposes the bumps after a seven-day cooldown.

[ci.yml](../.github/workflows/ci.yml) runs on every push and pull
request to `main`. Its Rust jobs run every test and scenario, clippy
and rustfmt, cargo-deny and cargo-audit, and the drift suite. Its Go
jobs vet, race-test, lint and vulnerability-check the stub, and
cross-build it for the three release targets (NFR-10). It also runs
`mdsmith check .` and zizmor over the workflows themselves. A final
job named `CI` passes only when every other job passed. It is the one
check the branch and release-tag rulesets require, because GitHub
names a required check after a job, never after the workflow.

[nightly.yml](../.github/workflows/nightly.yml) fuzzes every `Fuzz*`
target in the tree for five minutes each (ENG-07). It discovers the
targets, so a new one is fuzzed the night after it merges. The
crash-consistency test (ENG-06), the concurrency soak (ENG-09) and
the live Claude Code contract test (ENG-17) join it as their plans
land.

[review.yml](../.github/workflows/review.yml) reviews each pull
request from a branch of this repository once its CI passes (ENG-21,
ENG-28). It skips drafts, so a pull request is reviewed when it is
marked ready, which reruns CI. The agent runs on the stakeholder's
Claude subscription, not an API key billed per model token. It triggers on
`workflow_run`, so GitHub runs it as `main` defines it, never as the
pull request does. The review job runs an agent with read-only tools
on the pull request's tree, following
[the review skill](../.claude/skills/review/SKILL.md). The agent only
writes a review outcome. The post job alone has the reviewer app's
key. It runs the `review-gate` crate's executable, which approves only an
approving review outcome with no blocking finding, on the reviewed
head, with `CI` green there; otherwise it requests changes.
[ADR-2609301941](adr/ADR-2609301941-agent-review.md) records why.

[release.yml](../.github/workflows/release.yml) runs from the Actions
"Run workflow" button with a version like `v0.1.0`. A pushed tag is
deliberately not the trigger: a failed build would leave a public tag
pointing at nothing. The workflow takes these steps:

1. Validate the version and run vet and the race-enabled suite.
2. Build each target on two builders, Linux and macOS, with
   `CGO_ENABLED=0 -trimpath -buildvcs=true` and the version stamped.
3. Compare the two builders' SHA-256 sums and stop on any difference
   (ENG-19).
4. Generate an SPDX SBOM, attest build provenance and the SBOM, and
   sign `checksums.txt` keylessly with cosign (ENG-20).
5. Create the tag and publish the release, only now that everything
   above succeeded.

GitHub's build-provenance attestation meets SLSA Build Level 2. Level
3 needs the build isolated in a reusable workflow; that step keeps
ENG-20 pending. [SECURITY.md](../SECURITY.md) has the verification
commands.

## The plan catalog and its merge driver

Work is tracked as plans under `plan/`, indexed by
[PLAN.md](../PLAN.md), in frit's format: see
[plan/proto.md](../plan/proto.md) and the `plan-*` skills under
`.claude/skills`. Run `mdsmith merge-driver install` once per clone.
`PLAN.md` is a generated catalog, so two branches adding a plan
conflict on it by construction; the driver regenerates it during a
merge instead of leaving conflict markers.
