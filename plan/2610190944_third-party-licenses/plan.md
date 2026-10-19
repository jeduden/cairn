---
id: 2610190944
title: "Ship third-party license texts with every release"
status: "🔲"
summary: >-
  Fail the pull request that adds a crate with no license text
  (ENG-30), and embed a committed, regenerated THIRD_PARTY_LICENSES in
  every executable, shown by cairn licenses (ENG-31), so a new
  dependency is covered the moment it is added.
model: sonnet
depends-on: []
---
# Ship third-party license texts inside every executable

## Goal

A new dependency reaches a release with its license text and copyright
lines inside the executable, and no manual step. CI stops the pull
request that adds a crate with no license text, and the one that adds a
dependency without regenerating `THIRD_PARTY_LICENSES`.

## Context

The stakeholder ruled that third-party license texts ship with
releases, inside each executable and not as a separate asset, and that
a crate with no license text is never allowed. Today a release
publishes bare `cairn-*` binaries, an SPDX SBOM, a signed
`checksums.txt` and attestations (ENG-20). Nothing carries a license
text. `deny.toml` checks each crate's declared license against the
ENG-18 allow-list but never reads a text.

Mechanism, as the proposed
[ADR](../../docs/adr/ADR-2610190950-third-party-licenses-tool.md)
sets out: one committed `THIRD_PARTY_LICENSES` at the repository root,
the union of the crates linked on any release target. The core embeds
it with `include_str!` and prints it with `cairn licenses`. A CI gate
fails when the committed file differs from a regeneration. The release
build fails when a built executable's output differs from the file
regenerated at the released commit. The ENG-20 signature and
attestations cover the file because it is inside the binary.

Core rules. The text is a compiled-in string: no dependency (ENG-18),
no socket (I4, SEC-01). `licenses` is a new node-wide utility name; this
change proposes it for the domain model's list. The stakeholder rules
on it before phase 2.

Reuse, searched first:

- **`cargo-deny`** (in CI): declared licenses only. Kept for that.
- **The SBOM** (syft, ENG-20): identifiers, not texts. Kept.
- **`tooling/engineering`**: reads `cargo metadata` for ENG-18 and walks
  from the binaries under `crates/`. The linked-set walk is reused.
- **`cargo-about`, `cargo-bundle-licenses`**: compared in the ADR's
  table; the ADR stays `proposed` and recommends the in-repository tool.

Stakeholder paths: phases 1 to 3 touch `/tooling/`, `/.github/` and
`/DEPENDENCIES.md` if the tool needs a line, all owned in
[CODEOWNERS](../../.github/CODEOWNERS); each waits for the
stakeholder's approval. The requirement change (SRS rows, scenarios,
ADR, plan) is its own change set.

## Tasks

1. Proving slice: the ENG-30 check with the shared linked-set reader,
   fixtures, its scenario bound, its drift case and its CI step
   (phase 1)
2. The generator: a deterministic `THIRD_PARTY_LICENSES` for the union
   of release targets, the non-crate entries, and the CI regeneration
   gate with its drift case (phase 2)
3. Embedding: `cairn licenses` prints the file, the release build
   compares each built executable's output with the regeneration, and
   ENG-31 closes (phase 3)

## Execution

| Phase | Model  | Gate                                                                                                                    |
| ----- | ------ | ----------------------------------------------------------------------------------------------------------------------- |
| 1     | sonnet | `cargo test -p scenario` runs ENG-30; a scratch copy with a license-less crate added fails the new CI step              |
| 2     | sonnet | The built tool, run twice, gives identical bytes naming every linked crate; a stale committed file fails the gate       |
| 3     | opus   | The built `cairn licenses` equals the committed file in an isolated `CAIRN_HOME`; the release build fails on a mismatch |

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
