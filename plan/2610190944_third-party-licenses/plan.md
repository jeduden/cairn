---
id: 2610190944
title: "Ship third-party license texts with every release"
status: "🔲"
summary: >-
  Check on every pull request that each crate has a recoverable license
  text (ENG-30), and publish per release target a signed, reproducible
  THIRD_PARTY_LICENSES file for the crates that binary links (ENG-31),
  so a new dependency is covered the moment it is added.
model: sonnet
depends-on: []
---
# Ship third-party license texts with every release

## Goal

A new dependency reaches a release with its license text and copyright
lines shipped beside the binaries, and no manual step. CI stops the pull
request that adds a crate with no recoverable text. The release
publishes a signed `THIRD_PARTY_LICENSES` file per target.

## Context

The stakeholder ruled that third-party license texts ship with
releases, and that it must work automatically as soon as there is a new
dependency. Today a release publishes bare `cairn-*` binaries, an SPDX
SBOM, a signed `checksums.txt` and attestations (ENG-20). Nothing
carries a license text. `deny.toml` checks each crate's declared
license against the ENG-18 allow-list but never reads a text.

Reuse, searched first:

- **`cargo-deny`** (already in CI): declared licenses only, no texts.
  Kept for what it does.
- **The SBOM** (syft, ENG-20): identifiers, not texts. Kept; it is not
  the answer.
- **`tooling/engineering`**: already reads `cargo metadata` for
  ENG-18, including the walk from the binaries under `crates/` that
  counts linked dependencies. The linked-set walk is reused, not
  copied.
- **`cargo-about`, `cargo-bundle-licenses`**: weighed in the proposed
  [ADR-2610190950](../../docs/adr/ADR-2610190950-third-party-licenses-tool.md);
  neither knows the bundled SQLite or the musl C library, and one more
  pinned binary is not needed. The ADR stays `proposed` until the
  stakeholder rules.

The release already attests each binary and signs `checksums.txt`. The
new files join `dist/` before the checksums are created. That puts them
under the existing signature (ENG-20) with no second mechanism.

Stakeholder paths. Phases 1 and 3 touch `/tooling/` and
`/.github/workflows/`, both owned by the stakeholder in
[CODEOWNERS](../../.github/CODEOWNERS), so each waits for the
stakeholder's approval; keep them out of any other pull request. The
requirement change (the SRS rows, the scenarios, the ADR and this plan)
is its own change set.

The file is named `THIRD_PARTY_LICENSES`, not `THIRD_PARTY_NOTICES`:
the domain model gives "notice" one meaning, the opt-in notice
(INJ-10).

## Tasks

1. Proving slice: the check behind ENG-30, with the shared linked-set
   reader, a failing and a passing fixture, its scenario bound, its
   drift case and its CI step (phase 1)
2. The generator: a deterministic `THIRD_PARTY_LICENSES` for any
   target, with entries for code compiled in without being a crate, and
   a regeneration check (phase 2)
3. Release wiring: generate per target from the released commit, list
   in `checksums.txt`, attest with the binaries, verify after
   publishing (phase 3)

## Execution

| Phase | Model  | Gate                                                                                                                   |
| ----- | ------ | ---------------------------------------------------------------------------------------------------------------------- |
| 1     | sonnet | `cargo test -p scenario` runs ENG-30; a scratch copy with a license-less crate added fails the new CI step             |
| 2     | sonnet | The built tool, run twice for each release target, gives identical bytes naming every linked crate and no dev-only one |
| 3     | opus   | A dry run of the release workflow on a branch publishes the file, and its verify job fails when the file is removed    |

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

| #   | Status | Phase                                                                      |
| --- | ------ | -------------------------------------------------------------------------- |
| 1   | 🔲     | [Every crate has a license text, checked on each pull request](phase-1.md) |
<?/catalog?>

## Acceptance Criteria

- [ ] ENG-30 is off `@pending` and its scenario passes in the scenario
  suite; a crate with no recoverable license text fails CI in the pull
  request that adds it
- [ ] ENG-31 is off `@pending` and its scenario passes; each release
  publishes `THIRD_PARTY_LICENSES` per target, listed in the signed
  checksum file and attested with the binaries
- [ ] Generating the file twice gives identical bytes; it names every
  crate linked into its target, the bundled SQLite and the musl C
  library, and no build-only, test, fuzz or tooling crate
- [ ] Drift cases for the ENG-30 check and the ENG-31 verification are
  caught (ENG-27)
- [ ] No module is added to any shipped executable; ENG-18's list is
  unchanged, or an accepted ADR covers any addition
- [ ] `mdsmith check .` passes
