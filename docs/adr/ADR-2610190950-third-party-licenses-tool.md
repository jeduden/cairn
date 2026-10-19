---
id: ADR-2610190950
title: "Third-party license texts from an in-repository tool, not cargo-about"
status: proposed
summary: >-
  A small tool under tooling/ reads Cargo.lock and cargo metadata to
  check every crate's license text on each pull request and to write
  each release target's THIRD_PARTY_LICENSES file. It adds no
  dependency to a shipped executable and none to the repository.
---
# ADR-2610190950: Third-party license texts from an in-repository tool

## Context

ENG-30 asks CI to stop a pull request that brings in a crate with no
recoverable license text. ENG-31 asks each release to publish, per
target, a `THIRD_PARTY_LICENSES` file with the license text and
copyright lines of every crate that binary links, generated the same
way twice and covered by the release's signature and attestation
(ENG-20). The stakeholder wants this to work with no manual step when a
dependency arrives.

The licenses on the ENG-18 allow-list (MIT, BSD, ISC, Apache-2.0)
require the text and copyright line to travel with a binary. The
bundled SQLite (public domain) and, on the musl targets, the C library
(MIT) are compiled in without being crates, so no crate-based tool
sees them.

## Decision

Write the generator and the check as one Rust tool under `tooling/`.
It reads only `Cargo.lock`, `cargo metadata` (with
`--filter-platform` per release target, following normal-dependency
edges from the binaries under `crates/`) and the crate sources `cargo`
has fetched. A crate whose package ships no license file gets a text
the repository keeps under a path named for its name and version; the
check fails until one is added. A short fixed list names the code that
is compiled in without being a crate.

The tool uses `serde_json`, which ADR-2610142221 already accepts. It
adds no module, so ENG-18's list does not change. If a later review
prefers a maintained tool, `cargo-about` (MIT OR Apache-2.0) is the
candidate; it would be a pinned CI tool, not a dependency of any crate,
and would need its own accepted ADR and a way to carry the two
non-crate entries.

## Alternatives

- **`cargo-about`.** Mature and template-driven, and it clarifies
  missing licenses in a config. It reads a license policy of its own
  beside `deny.toml`, knows nothing of the bundled SQLite or the C
  library, and pins another external binary in the release workflow.
- **`cargo-bundle-licenses`.** Writes one file for the workspace, not
  one per target, and its output order and metadata change across
  versions, which works against byte-for-byte reproduction.
- **`cargo-deny`'s license check alone.** It already runs in CI and
  checks the declared SPDX expression, but it never reads or ships a
  text, so it cannot satisfy ENG-30 or ENG-31.
- **The SBOM (ENG-20).** SPDX lists license identifiers, not texts or
  copyright lines.

## Consequences

- A crate with no license file blocks its pull request until a text is
  kept for it; that is the point of ENG-30.
- The two non-crate entries are a short list in the tool, reviewed with
  the stakeholder's paths; a new compiled-in library needs a line there.
- The stakeholder may still choose `cargo-about`; the requirements do
  not name a tool.
