---
n: 5
title: "SLSA Build Level 3 provenance (ENG-20)"
status: "🔳"
result: true
summary: >-
  Only the release-only release-build.yml attests build provenance, a verify job checks the published release against each signer's exact identity at main, and the ENG-20 scenario stays pending until the stakeholder rules.
---
## Handoff

What landed:

- `build.yml` only builds: no `attest` input, no attest job, no signing
  permission. CI calls it granting `contents: read` alone.
- A release-only reusable workflow, `release-build.yml`, calls
  `build.yml`, granting it nothing, then its `attest` job attests every
  binary's build provenance. That job alone holds `id-token` and
  `attestations` write, and runs no repository code. `release.yml`
  calls it with both permissions; the signer is `release-build.yml`.
- A first review found the earlier design unsafe: the attestation was
  signed in `build.yml`, which CI also runs, and `verify` checked
  `--signer-workflow` with no ref, so any branch could make an
  attestation that passed. A Sigstore certificate names the workflow at
  the dispatched ref, and the release creates its tag last, so the tag
  cannot be the pinned ref. A release now runs only from `main` (the
  preflight refuses any other ref), and `verify` pins
  `--cert-identity` to `release-build.yml@refs/heads/main` with
  `--source-ref refs/heads/main`, `--source-digest` and
  `--deny-self-hosted-runners`. The SBOM attestation and `cosign
  verify-blob` pin `release.yml@refs/heads/main` the same way.
- `release.yml` attests the SBOM and signs the checksums, now listing
  the SBOM, and attaches `cairn.spdx.json` to the release.
- The `engineering` crate's `release_provenance` module reads the four
  workflows. Three gate tests in `tooling/engineering/tests/checkout.rs`
  hold them, registered in the drift registry's gate tests; sixteen
  drift cases guard them, among them an unpinned `--signer-workflow`
  verify and `ci.yml` granting `id-token` to its build call.
- `engineering_release.rs` binds the scenario, but `@ENG-20` stays
  `@pending` until the stakeholder rules: "Given a published release"
  reads the workflows, since a checkout holds no release; the `verify`
  job is the check against a real one.

What remains: the stakeholder's ruling on the binding. The first real
release run is the first live test of the `verify` job and of the SPDX
predicate type `https://spdx.dev/Document/v2.3`; adjust it there if the
attest-sbom action names another.
