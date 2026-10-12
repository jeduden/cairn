---
summary: >-
  Release notes and upgrade notes per version, newest first (ENG-23).
---
# Changelog

Every release gets a section here, newest first, with its changes and
its upgrade notes (ENG-23). Cairn follows semantic versioning.

## Unreleased

- Repository bootstrap: the SRS, one pending scenario per requirement,
  CI, a nightly fuzz job and the release pipeline. `cairn version` is
  the only command.
- The invariants I1 to I10 are reworded to the domain model; their
  initial wording is approved in ADR-2610091555.
- I10 says rebuilding reproduces derived artifacts' content exactly,
  whatever their storage layout; proposed in ADR-2610112004, pending
  the security reviewer's approval.

### Upgrade notes

- An invariant change is a design change (§1.3), so the release that
  carries this rewording is a new major version.
