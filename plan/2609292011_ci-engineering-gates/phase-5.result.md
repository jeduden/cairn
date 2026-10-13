---
n: 5
title: "SLSA Build Level 3 provenance (ENG-20)"
status: "✅"
result: true
summary: >-
  Provenance is attested inside the reusable build workflow, a verify job checks the published release, and the ENG-20 scenario is off pending.
---
## Handoff

What landed:

- `build.yml` takes an `attest` input and has an `attest` job that
  attests every binary's build provenance, so the signer is `build.yml`.
  The job runs no repository code. `release.yml` calls it with
  `attest: true` and the two permissions; `ci.yml` grants them too (a
  caller must grant what a nested job requests) but passes no `attest`.
- `release.yml` attests the SBOM and signs the checksums, now listing
  the SBOM, and attaches `cairn.spdx.json` to the release. A new
  `verify` job runs after `publish` and checks the published release:
  `cosign verify-blob` on the bundle, `gh attestation verify` with
  `--signer-workflow` set to `build.yml` for provenance, and the SPDX
  predicate for the SBOM.
- The `engineering` crate's `release_provenance` module reads the
  workflows; `engineering_release.rs` binds the scenario. The
  "Given a published release" step reads the workflows (a checkout holds
  no release); the `verify` job is the check against a real one.
- Nine drift cases guard `ENG-20`.

Scenario off `@pending`: `@ENG-20`.

What remains: the first real release run is the first live test of the
`verify` job and of the SPDX predicate type
`https://spdx.dev/Document/v2.3`; adjust it there if the attest-sbom
action names another.
