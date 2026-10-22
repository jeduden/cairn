---
summary: >-
  Release notes and upgrade notes per version, newest first (ENG-23).
---
# Changelog

Every release gets a section here, newest first, with its changes and
its upgrade notes (ENG-23). Cairn follows semantic versioning.

## Unreleased

- Upgrade note: the product is now ParleyInn and its executable is
  `parinn`. The executable, its release assets (`parinn-*`,
  `parinn.spdx.json`), its MCP server name and every command's name
  change from `cairn`; scripts and hook entries that call `cairn` must
  call `parinn`. An existing home keeps working: `~/.cairn`,
  `CAIRN_HOME`, `cairn:` addresses, `Cairn-Room:` and `Cairn-Link:`
  trailers, and the payload, node-identity, key-store and Keychain
  labels keep their spelling.
- Repository bootstrap: the SRS, one pending scenario per requirement,
  CI, a nightly fuzz job and the release pipeline. `cairn version` is
  the only command.
- The invariants I1 to I10 are reworded to the domain model; their
  initial wording is approved in ADR-2610091555.
- I2 trusts user events matched to a prompt, and I3 lets a pin be
  omitted for run ambiguity; approved in ADR-2610112209.
- I10 says rebuilding reproduces derived artifacts' content exactly,
  whatever their storage layout; approved in ADR-2610112004.
- A release-only reusable workflow, `release-build.yml`, attests each
  binary's build provenance (SLSA Build Level 3); the build CI runs
  attests nothing. A release runs only from `main`. The release attaches
  its SBOM, and a last job verifies the published release against each
  signer's exact identity at `refs/heads/main` (ENG-20).
- Every command refuses a home that another OS user owns or can reach,
  or that `home.json` binds to another home id than `CAIRN_HOME_ID`
  (SEC-02, SEC-03). The home keeps a hash-chained audit log under
  `audit/` that `cairn verify` checks (OPS-02). New commands: `cairn
  hook`, which fails open and audits, `cairn status`, `cairn doctor`
  and `cairn counter ack` (OPS-03, ENG-04).

### Upgrade notes

- An invariant change is a design change (§1.3), so the release that
  carries this rewording is a new major version.
