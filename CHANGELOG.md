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
- The invariants I1 to I10 are reworded to the domain model, each change
  under an accepted security-review decision record: ADR-2610032155,
  ADR-2610052329, ADR-2610061500, ADR-2610061700, ADR-2610061900, the
  series ADR-2610062000 to ADR-2610063300, and ADR-2610072203.

### Upgrade notes

- An invariant change is a design change (§1.3), so the release that
  carries this rewording is a new major version.
