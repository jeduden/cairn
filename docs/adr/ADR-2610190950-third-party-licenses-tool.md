---
id: ADR-2610190950
title: "Third-party license texts embedded from an in-repository tool"
status: proposed
summary: >-
  A small tool under tooling/ reads Cargo.lock and cargo metadata to
  check every crate's license text and to regenerate one committed
  THIRD_PARTY_LICENSES file, which each executable embeds and prints
  with cairn licenses. It adds no dependency to the core or the repository.
---
# ADR-2610190950: Third-party license texts embedded from an in-repository tool

## Context

ENG-30 asks CI to stop a pull request that brings in a crate with no
license text. ENG-31 asks every shipped executable to carry the license
text and copyright lines of every crate it links, so the texts travel
with the binary wherever it goes. The stakeholder wants this to work
with no manual step when a dependency arrives.

The ENG-18 allow-list licenses (MIT, BSD, ISC, Apache-2.0) require the
text and copyright line to travel with a binary. The bundled SQLite
(public domain) and, on the musl targets, the C library (MIT) are
compiled in without being crates, so no crate-based tool sees them.

## Decision

Write the generator and the check as one Rust tool under `tooling/`.
It reads only `Cargo.lock`, `cargo metadata` (with `--filter-platform`
per release target, following normal-dependency edges from the binaries
under `crates/`) and the crate sources `cargo` has fetched. A short
fixed list in the tool names the code compiled in without being a
crate.

**One committed file, the union of the targets (option a).** The tool
writes `THIRD_PARTY_LICENSES` at the repository root, covering every
crate linked on any release target. The core embeds it with
`include_str!` and prints it with `cairn licenses`. CI regenerates the
file and fails the pull request when the committed one differs, so a
developer who adds a dependency regenerates it in the same change. A
crate with no license file fails the check; the repository keeps no
substitute text, and the crate is removed or replaced.

The release build runs each built executable's `cairn licenses` on its
own platform and compares it with the file regenerated at the released
commit. The file is inside the executable, so the existing signature and
attestations (ENG-20) cover it and no separate asset exists.

**Why the union, not a file per target (option b).** Per-target
embedding through `cfg` would list exactly what each binary links.

| Question              | (a) One file, union                          | (b) Per-target file through `cfg`                    |
| --------------------- | -------------------------------------------- | ---------------------------------------------------- |
| Exactness             | Windows-only crates appear in the Linux text | Each binary lists exactly its own crates             |
| Compliance            | Over-discloses, never under                  | Exact, but a wrong `cfg` silently ships a wrong text |
| Files and checks      | One file, one regeneration check             | One file per target, one check each, a `cfg` map     |
| Where CI can check it | Any host, since it needs no target toolchain | Same, plus a build per target to test the `cfg`      |
| Size cost             | The union's text in every binary             | Smaller on the targets with fewer crates             |

(a) is recommended: the extra text costs a few kilobytes, while (b) adds
a failure mode (the `cfg` picking the wrong file) for no compliance gain.
Switch to (b) only if a target's binary size ever matters.

**Constraints.** The core gains no dependency: the text is a `&str`
compiled in, so ENG-18's list and its ten-dependency target are
unchanged. It gains no network access: `cairn licenses` only prints,
which I4 and SEC-01 allow, and the crate's clippy and reach test still
pass. The tool uses `serde_json`, which ADR-2610142221 accepts, and
adds no module. The binary grows by the size of the text, a few hundred
kilobytes at most for the current tree; no requirement bounds binary
size. `cairn licenses` is a new node-wide utility name, proposed for the
domain model's list in this change; the stakeholder may rule otherwise.

## Alternatives

How the three candidates compare, for the tool that writes the file:

| Question                        | In-repository tool                                      | `cargo-about`                                            | `cargo-bundle-licenses`                              |
| ------------------------------- | ------------------------------------------------------- | -------------------------------------------------------- | ---------------------------------------------------- |
| Maintenance burden              | Ours: a few hundred lines to keep with `cargo metadata` | Upstream's; we track its config and releases             | Upstream's, a smaller project with fewer releases    |
| License-file discovery          | Our rules; wrong guesses show in the tests we write     | Mature, with per-crate clarifications in a config        | Takes the first license-like file; no clarifications |
| Non-crate code (SQLite, musl)   | A fixed list in the tool                                | A clarification entry or a template addition, by hand    | None; appended by hand                               |
| Reproducibility, byte stability | We fix the order and format; tested twice per run       | Template-driven; stable for one pinned version           | Output order and metadata shift between versions     |
| Supply-chain trust              | Our code, under the ENG-21 review                       | A pinned external binary fetched in CI                   | A pinned external binary fetched in CI               |
| CI time                         | Builds with the workspace, seconds                      | Install or cache the binary, then run                    | Install or cache the binary, then run                |
| Review surface                  | The tool's source, in stakeholder paths                 | Its config and template, plus a trust decision           | Its output only; less control                        |
| Fit with ENG-18 and ENG-27      | No new module; drift cases written against our own code | A tool outside the manifests; its own ADR and drift case | Same as `cargo-about`                                |

**Recommendation.** The in-repository tool: it is the only candidate
that knows the non-crate code, reproduces byte for byte by construction
and adds no external binary to the release chain. `cargo-about` is the
fallback if the stakeholder prefers a maintained tool; it would need its
own accepted ADR, a drift case, and a hand-kept entry for each
non-crate library.

- **`cargo-deny`'s license check alone.** It already runs in CI and
  checks the declared SPDX expression, but it never reads or ships a
  text, so it cannot satisfy ENG-30 or ENG-31.
- **The SBOM (ENG-20).** SPDX lists license identifiers, not texts or
  copyright lines.
- **A separate release asset.** Rejected by the stakeholder: a file
  beside the binary can be lost when the binary is copied.

## Consequences

- A crate with no license file blocks its pull request, and the crate
  is removed or replaced; the repository keeps no text for it.
- A pull request that changes `Cargo.lock` regenerates
  `THIRD_PARTY_LICENSES` in the same change, or CI fails it.
- The non-crate entries are a short list in the tool, reviewed on the
  stakeholder's paths; a new compiled-in library needs a line there.
- Every executable is a little larger and answers `cairn licenses`.
- The stakeholder may still choose `cargo-about`; the requirements do
  not name a tool.
