---
summary: >-
  How to report a vulnerability in Cairn privately, the 90-day
  coordinated disclosure policy, which versions get fixes, and how to
  verify a release (ENG-24, ENG-20).
---
# Security policy

Cairn stores long agent histories and feeds recalled content back to
a model. Its invariants — no automatic path from untrusted content to
the model, no network, isolation per tenant — are security claims.
A way to break one is a vulnerability. See
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
Sigstore, build provenance for every binary, and an SPDX SBOM. The
release workflow notes carry the exact commands. In short:

```sh
gh attestation verify cairn-linux-amd64 -R jeduden/cairn
cosign verify-blob --bundle checksums.txt.sigstore.json \
  --certificate-identity-regexp \
  '^https://github.com/jeduden/cairn/.github/workflows/release.yml@' \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  checksums.txt
sha256sum -c checksums.txt --ignore-missing
```
