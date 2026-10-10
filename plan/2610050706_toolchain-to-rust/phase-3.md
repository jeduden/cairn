---
n: 3
title: "Coverage per test layer"
status: "🔲"
result: false
---
# Phase 3: coverage per test layer

Requirements. ENG-11's floors stay pending until the product crates
exist; this phase makes their measure exist.

What lands:

- A `coverage` crate and executable that runs `cargo llvm-cov` once
  per test layer and once for all of them, and reports line coverage
  per crate for each:
  - unit: the tests beside the code, in `src/`;
  - integration: a crate's `tests/` targets but the end-to-end ones;
  - end-to-end: the `bdd` runner and every `tests/e2e*` target, which
    drive a built executable as a process.
- Floors per crate from `[workspace.metadata.coverage]`, failing the
  build below them, in place of `scripts/check-coverage.sh`.
- Unit-test files (`tests.rs`) are left out of the measure, so the
  number counts shipped code only.
- Codecov receives one flag per test layer.
- Test-engineer agents review the test pyramid.

Gate: CI's coverage job prints the table per test layer and fails a
crate below its floor.
