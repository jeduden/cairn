---
id: 2609292014
title: "M3: landmarks, cues and lifecycle"
status: "🔲"
summary: >-
  Structural landmarks with tiered roll-up, recall cues at the
  moments a need is likely, backup and restore, purge with
  tombstones, and byte-identical rebuild.
model: opus
depends-on: [2609292013]
---
# M3: landmarks, cues and lifecycle

## Goal

Give Claude a short, clean index of every span, and a pointer into it
at the moment it needs one. Give operators backup and purge. Rebuild
must reproduce all derived state exactly.

## Context

Milestone M3. Scope: LMK P0, CUE P0, ADM-06 to ADM-08 and SEC-14.
The cue moments need ASM-11, verified by spike S1.

## Tasks

1. Spans and structural landmarks with sanitized fields: LMK-01 to
   LMK-04
2. Tiered roll-up bounded to O(k log n) blocks: LMK-05
3. Backup and restore: ADM-06
4. Purge with tombstones and secure delete: ADM-07, SEC-14
5. Rebuild: ADM-08
6. The working view, anchors and failure signatures: CUE-02
7. Back-reference, touch and recurrence cues: CUE-01, CUE-03 to
   CUE-05
8. Gates, budgets, cue events and statistics, configuration: CUE-07
   to CUE-09, CUE-12

## Acceptance Criteria

- [ ] Every scenario in scope passes; none of them is still @pending
- [ ] Recall target met at 10 compactions
- [ ] Cue precision target of §11.1 met
- [ ] Rebuild is byte-identical on every evaluation store
