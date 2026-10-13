---
summary: >-
  How to report a vulnerability in Cairn privately, the 90-day
  coordinated disclosure policy, which versions get fixes, and how to
  verify a release (ENG-24, ENG-20).
---
# Security policy

Cairn stores long agent histories and feeds recalled content back to
a model. Its invariants — no automatic path from untrusted content to
the model, no network in the core and every other component inside
its declared boundary, isolation per principal — are security
claims. A way to break one is a vulnerability. See
[docs/srs/06-security.md](docs/srs/06-security.md) for the threat
model.

## Reporting a vulnerability

Do not open a public issue. Report privately through GitHub's
private vulnerability reporting:
<https://github.com/jeduden/cairn/security/advisories/new>.

Include the version or commit, the steps to reproduce, and the
invariant or requirement id you believe is broken. We confirm
receipt within five working days.

## Disclosure policy

We follow a 90-day coordinated disclosure policy. We aim to ship a
fix within 90 days of the report. We publish the advisory when the
fix ships, or when the 90 days end, whichever comes first. We will
agree on an earlier or later date with the reporter when the case
calls for it.

## Supported versions

Cairn has not shipped a release yet. From the first release on,
security fixes are backported to the latest minor release. Older
minor releases get no fixes; upgrade to the latest.

## Verifying a release

Each release attaches a `checksums.txt` signed keylessly with
Sigstore, with its Sigstore bundle, build provenance for every binary,
and an SPDX SBOM. A release runs only from `main`, so each signer is
pinned to its exact workflow at `refs/heads/main`: the build provenance
to `release-build.yml`, the checksums and the SBOM to `release.yml`. An
attestation a branch made names that branch, and fails. In short:

```sh
gh attestation verify cairn-linux-amd64 --repo jeduden/cairn \
  --cert-identity https://github.com/jeduden/cairn/.github/workflows/release-build.yml@refs/heads/main \
  --source-ref refs/heads/main \
  --deny-self-hosted-runners
cosign verify-blob --bundle checksums.txt.sigstore.json \
  --certificate-identity https://github.com/jeduden/cairn/.github/workflows/release.yml@refs/heads/main \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  checksums.txt
sha256sum -c checksums.txt --ignore-missing
```

Add `--source-digest` with the SHA of the commit the release's tag
names to tie the provenance to it. The SBOM attestation verifies the
same way. Use the `release.yml` identity and
`--predicate-type https://spdx.dev/Document/v2.3`.
