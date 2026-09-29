---
summary: >-
  Build and test commands, the executable requirement matrix (every SRS
  id has one tagged Gherkin scenario, pending until written), the
  coverage floors, and how CI, the nightly fuzz run and the
  reproducible, signed release pipeline work.
---
# Development

Build, test and release reference, plus the mechanics behind the
rules in [CLAUDE.md](../CLAUDE.md).

## Build & test commands

Requires Go 1.26. `go.mod` pins the exact toolchain (ENG-01), and the
`go` command fetches it on first use. Dev tools build from
[tools/go.mod](../tools/go.mod), so their dependency trees never enter
the module graph that ENG-18 counts.

- `go build ./...` — build all packages
- `go test ./...` — run all tests, scenarios included
- `go test -race ./...` — the same under the race detector (ENG-09)
- `go test -run TestName ./...` — run a specific test
- `go test ./... -coverpkg=./... -coverprofile=cover.out` — all tests,
  one coverage profile; `go tool cover -func=cover.out` summarises it
- the coverage floor CI enforces:

  ```sh
  scripts/check-coverage.sh 100 ./cmd/cairn ./internal/srs ./internal/scenario ./internal/adr
  ```

- `go vet ./...` — run go vet
- `go tool -modfile=tools/go.mod golangci-lint run` — lint
- `go tool -modfile=tools/go.mod govulncheck ./...` — known
  vulnerabilities (ENG-16)
- `go mod tidy -modfile=tools/go.mod` — tidy the tools module
- `mdsmith check .` — lint the Markdown; `mdsmith fix .` rewrites
  what it can, including the generated catalogs

## The executable requirement matrix

The SRS under [docs/srs](srs/index.md) is executable. Every
requirement row — `REC-01` through `ENG-25`, plus the assumptions
`ASM-01` to `ASM-10` — has exactly one Gherkin scenario under
`features/`, one file per SRS section. The scenario carries the row's
id, its priority and its traced invariants as tags:

```gherkin
@REC-03 @P0 @I1 @I10 @pending
Scenario: re-ingesting a source creates no duplicate events
```

`internal/scenario`'s `TestSpecificationAndFeaturesAgree` keeps the
two sides in step. A requirement with no scenario fails it. So does a
scenario naming no requirement, an id tagged twice, or a priority or
invariant tag that disagrees with the row. `internal/srs` also checks
that Appendix B's invariant coverage and requirement counts match the
tables they summarise.

A scenario still tagged `@pending` is declared but not written. It
is skipped and reported as such, never counted as a pass. godog runs
the rest in strict mode from `cmd/cairn`'s `TestFeatures`, so a step
whose text matches no definition fails instead of passing as
undefined. CI's test job prints the count of passing and pending
scenarios on every run.

- `go test ./cmd/cairn -run TestFeatures -v` — every scenario, pending
  ones listed as skipped
- `go test ./cmd/cairn -run 'TestFeatures/^REC-03:'` — one scenario by
  id; the anchor and colon keep `REC-0` from matching `REC-03`
- `go test ./internal/scenario ./internal/srs` — the gates alone

### Adding or writing a scenario

A new requirement lands in the SRS table and in its section's feature
file in the same change, tagged `@pending`. To write a scenario, drop
`@pending`, make its Given/When/Then concrete, and bind the steps. A
section's step functions live in `cmd/cairn/bdd_<section>_test.go`,
appended to the registry from `init` the way
[bdd_engineering_test.go](../cmd/cairn/bdd_engineering_test.go) does.
A section adds a file and never a line to `bdd_test.go`, so sections
land in any order.

Every step binds on the one `world` a scenario threads. The world
points `HOME` and `CAIRN_HOME` at fresh temporary directories, so no
step can reach the real home (ENG-14). State a section tracks beyond
the world lives in its own struct, reached through `section[T]`,
never as a new field on `world`. A step text two sections both define
fails as ambiguous, so reuse the existing text. The feature headers
list the shared vocabulary the pending scenarios already use.

## Coverage

Two measures run on every push. Codecov's project and patch
thresholds in [codecov.yml](../codecov.yml) are a soft ratchet.
[scripts/check-coverage.sh](../scripts/check-coverage.sh) is a hard
floor: it fails when a named package drops below the given percent of
its non-excluded statements. Today every package is gated at 100%.
ENG-11's floors — 90% for the security-sensitive packages, 80%
overall — join the CI call as those packages land.
[scripts/coverage-exclude.txt](../scripts/coverage-exclude.txt) lists
each justified exclusion, one statement range per line with its
reason; only a pure process boundary such as `main()` belongs there.

## Static analysis

[.golangci.yml](../.golangci.yml) enables the ENG-16 set:
staticcheck, gosec, errcheck and the `go vet` suite. It adds
`depguard`, which refuses `net`, `net/http` and `os/exec` in
non-test code, and `gochecknoglobals` for ENG-03's
no-mutable-package-state rule. depguard stops a direct import;
[cmd/cairn/imports_test.go](../cmd/cairn/imports_test.go) walks the
whole import closure of the shipped binary. That test catches the
same packages arriving through a dependency. Both are the import half
of SEC-01; the network-deny sandbox half is ENG-12, still pending.

## CI, nightly and release

Every workflow lives in `.github/workflows`. Every action is pinned
by commit SHA with its version in a trailing comment, and dependabot
proposes the bumps after a seven-day cooldown.

[ci.yml](../.github/workflows/ci.yml) runs on every push and pull
request to `main`. It covers build, vet, the race-enabled test suite
with coverage, and a static cross-build for the three release targets
(NFR-10). It also runs golangci-lint, govulncheck, `mdsmith check .`,
and zizmor over the workflows themselves. A final job named `CI`
passes only when every other job passed. It is the one check the
branch and release-tag rulesets require, because GitHub names a
required check after a job, never after the workflow.

[nightly.yml](../.github/workflows/nightly.yml) fuzzes every `Fuzz*`
target in the tree for five minutes each (ENG-07). It discovers the
targets, so a new one is fuzzed the night after it merges. The crash
harness (ENG-06), the concurrency soak (ENG-09) and the live Claude
Code run (ENG-17) join it as their plans land.

[release.yml](../.github/workflows/release.yml) runs from the Actions
"Run workflow" button with a version like `v0.1.0`. A pushed tag is
deliberately not the trigger: a failed build would leave a public tag
pointing at nothing. The run takes these steps:

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
