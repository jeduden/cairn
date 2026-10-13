---
id: 2610112059
title: "M9: review, publishing, bridges"
status: "🔲"
summary: >-
  Forge review beside the room, handover, bundles that carry a room
  with a pull request, read-only publishing and outbound bridges, each
  through its own proposal.
milestone-order: 8
model: opus
depends-on: [2610112058]
---
# M9: review, publishing, bridges

## Goal

Let a room reach the forge and outside contributors: review beside its
branches, hand it over, and publish or import it as untrusted data.

## Context

Milestone M9 of [SRS §12.2](../docs/srs/12-delivery-plan.md), the last.
Its rows are P2, and each item lands through its own proposal and ADR.

Scope: LANE-08, LANE-11, LANE-15; VIEW-13, VIEW-19; REC-23; the rest
of SEC-26; SEC-28.

## Tasks

1. Forge state beside the room, shown as `asserted`: LANE-08, VIEW-13,
   VIEW-19
2. Handover and succession: LANE-11
3. Bundles, export and import as a foreign room: LANE-15, REC-23,
   SEC-26
4. Read-only publishing and the forge, CI and notification bridges:
   SEC-26, SEC-28
5. M9's initial targets (§12.3): freeze each with its evidence,
   removing its † and its §12.3 row

## Acceptance Criteria

- [ ] §12.2's M9 exit criteria hold
- [ ] Every scenario in scope is off @pending and passes, each item
  through its accepted proposal and ADR
