---
id: 2609292013
title: "M2: pins and restore"
status: "🔲"
summary: >-
  Verbatim pins and the deterministic, TrustedText-only restore
  block after compaction, plus plugin install.
model: opus
depends-on: [2609292012, 2609292006]
---
# M2: pins and restore

## Goal

Re-inject pinned constraints verbatim after every compaction through a
byte-deterministic restore block that untrusted content can never reach.

## Context

Milestone M2. Scope: PIN P0, INJ P0, ADM-01 to ADM-03, ADM-10 and
SEC-11.

## Tasks

1. Pins as operator events, immutable, budgeted: PIN-01 to PIN-04,
   PIN-06, PIN-08
2. The TrustedText type and the restore builder: INJ-01 to INJ-09,
   SEC-07
3. Project config may only tighten: SEC-11
4. Plugin distribution, install and uninstall, read-only doctor: ADM-01
   to ADM-03, ADM-10

## Acceptance Criteria

- [ ] Every scenario in scope passes; none of them is still @pending
- [ ] The constraint-survival target of §11.1 is met
- [ ] Golden files for the restore block are stable (ENG-10)
- [ ] Install then uninstall round-trips the configuration exactly
