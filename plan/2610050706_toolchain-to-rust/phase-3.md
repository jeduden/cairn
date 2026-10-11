---
n: 3
title: "Coverage per test layer"
status: "✅"
result: false
---
# Phase 3: coverage per test layer

Requirements. ENG-11's floors stay pending until the product crates
exist; this phase makes their measure exist.

What lands:

- [x] A `coverage` crate and executable that runs `cargo llvm-cov`
  once per test layer and once for all of them, and reports line
  coverage per crate for each:
  - unit: the tests beside the code, in `src/`;
  - integration: a crate's `tests/` targets but the end-to-end ones;
  - end-to-end: the `bdd` runner and every `tests/e2e*` target, which
    drive a built executable as a process.
- [x] Floors from `[workspace.metadata.coverage]`, failing the build
  below them, in place of `scripts/check-coverage.sh`: 99% of each
  crate's lines from its unit tests alone, so every function and
  module is tested directly, and 100% from every test layer together.
  An executable's `src/main.rs`, declared as an entry point, counts
  only toward the second.
- [x] Unit-test files (`tests.rs`) are left out of the measure, so the
  number counts shipped code only.
- [x] The pyramid's shape: the run counts the tests in each test layer,
  a bound scenario as one end-to-end test, and fails when a test
  layer holds no more tests than the one above it.
- [x] The measurement refuses to run inside itself, so a test that
  starts the executable cannot recurse into a second measurement.
- [x] Codecov receives one flag per test layer.
- [x] Test-engineer agents review the test pyramid and its shape; the
  `test-review` and `test-shape` skills drive them.
- [x] `docs/testing.md` holds the testing half of
  `docs/development.md`.

Gate: CI's coverage job prints the table per test layer and the
pyramid's shape, and fails a crate below its floor or an inverted
pyramid.
