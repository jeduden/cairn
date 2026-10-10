---
summary: >-
  How Cairn is tested: the test pyramid of unit, integration and
  end-to-end tests, line coverage and test counts per test layer with
  their floors, the executable requirement matrix (one Gherkin scenario
  per SRS id), drift injection, and the test-engineer agents that
  monitor and shape the pyramid.
---
# Testing

Every behaviour is proven at the lowest test layer that can prove it.
[CLAUDE.md](../CLAUDE.md) states the rules; this page is how they are
kept. Build and CI commands are in [development.md](development.md).

## The test pyramid

- **Unit tests** prove every function and module directly. They live
  in the `tests.rs` beside the module they test, use fakes for I/O,
  and run in milliseconds. They are most of the tests.
- **Integration tests** guard behaviour at a crate's public API: the
  gates against the real repository, and each real process or file
  boundary. They live in a crate's `tests/`.
- **End-to-end tests** guard requirements through the entry points:
  the `bdd` scenario runner, and every `tests/e2e*` target, which runs
  a built executable as a process. ENG-12's confined suite and ENG-17's
  live test join this test layer when they land.

## Coverage and shape

`cargo run -p coverage` measures the pyramid in one run. It runs
[cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) once per
test layer, over one build, and prints line coverage per crate for
each test layer and for all of them. It counts the tests in each test
layer too, scenarios included, and writes each layer's lcov to
`target/coverage/`. Install the tool first, with `cargo install
--locked cargo-llvm-cov@0.9.1`.

The policy lives in [Cargo.toml](../Cargo.toml) under
`[workspace.metadata.coverage]`, and the run fails when a crate falls
below it:

- **Unit: 99% of each crate's lines**, from unit tests alone. Unit
  tests prove every function and module directly.
- **Entry points** (`src/main.rs`) are left out of the unit and
  integration measures: an end-to-end test runs the executable and
  proves its `main`.
- **Integration and end-to-end** coverage is measured and reported,
  with no floor: those tests guard behaviour, not lines.
- **Every test layer together: 100%** of each crate's lines.
- **The shape:** more unit tests than integration tests, and more of
  those than end-to-end tests. An inverted pyramid fails the run.

A crate may name its own floors under
`[workspace.metadata.coverage.crates.<name>.floors]`, with the reason
beside them. ENG-11's floors for the product crates (90% for the
security-sensitive ones, 80% overall) join that way as they land. A
misspelled key or crate name in the policy fails the run, so a typo
cannot switch a floor off.

The measure counts shipped code only. Unit-test files (`tests.rs`)
and every `tests/` directory, step bindings included, are left out.
That is why a binding stays thin: the logic it calls lives in a library
crate, where the unit tests reach it. Every cargo the tool starts
carries `CAIRN_COVERAGE_MEASURING`, and a run that finds it set refuses
to start, so a test cannot set off a measurement inside a measurement.

CI's `coverage` job writes the table and the test counts into its
summary, and uploads one Codecov flag per test layer; Codecov's
thresholds in [codecov.yml](../codecov.yml) are a soft ratchet on top.
The Go stub keeps its own floor of 100%, which
[scripts/check-coverage.sh](../scripts/check-coverage.sh) enforces.
[scripts/coverage-exclude.txt](../scripts/coverage-exclude.txt) names
its one exclusion: `main()`, a pure process boundary.

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

## Proving the checks: drift injection

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

## Adding or writing a scenario

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
temporary directory. Each world gets its own fresh `HOME` and
`CAIRN_HOME` as well, for the processes its steps start (ENG-14).
State a section tracks lives in its own struct, reached through
`World::section::<T>()`, never as a new field on the world.

## The test-engineer agents

The `test-engineer` agent reviews the pyramid as a whole. Three
specialists go deep on one test layer each: `test-engineer-unit`,
`test-engineer-integration` and `test-engineer-end-to-end`. They read
and report; they never edit or approve.

- The `test-review` skill runs all four on a pull request, plan or
  design, beside the coverage table, and merges what they find.
- The `test-shape` skill goes on from there: it turns each finding into
  a change, written red then green, that adds a missing unit test,
  moves a test to its test layer, or moves logic out of a binding into
  a library its unit tests reach. It then measures again.
