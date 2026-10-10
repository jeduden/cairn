---
summary: >-
  Build commands, static analysis and supply-chain checks, and how CI,
  the nightly fuzz job and the reproducible, signed release pipeline
  work. Testing has its own page, docs/testing.md.
---
# Development

Build and release reference, plus the mechanics behind the rules in
[CLAUDE.md](../CLAUDE.md). How Cairn is tested, and how the tests are
measured, is in [testing.md](testing.md).

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
  held to the floors (see [testing.md](testing.md))
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
