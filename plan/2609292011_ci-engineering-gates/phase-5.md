---
n: 5
title: "SLSA Build Level 3 provenance (ENG-20)"
status: "🔳"
result: false
---
# Phase 5: sLSA Build Level 3 provenance (ENG-20)

Requirement. ENG-20 (P0, Ver T): release artifacts are signed with
Sigstore (keyless), carry SLSA Build Level 3 provenance and an SPDX
SBOM, and ship with a signed checksum file.

What exists: `release.yml` builds through the reusable `build.yml`.
It attests provenance in its own publish job, though. So the
provenance's signer is the caller, not the isolated build.

BDD coverage: `@ENG-20` comes off `@pending`. Its steps read the
release workflows. Provenance is attested inside the reusable build
workflow. The SBOM is attested and the checksums signed keyless. A
verification job checks the published release, with the build
workflow as its signer.

RED: the scenario fails on provenance attested outside `build.yml`.
GREEN: the attestation moved, the verification job, a drift case.

Gate: `@ENG-20` passes.

Status: open. The workflows, their gate tests and drift cases landed.
But `@ENG-20` is back on `@pending` until the stakeholder rules. Its
steps "Given a published release" and "the checksum file verifies…"
read the release workflows, not a published release. So the binding
does not exercise the whole row.
