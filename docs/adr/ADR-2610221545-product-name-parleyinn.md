---
id: ADR-2610221545
title: "The product is named ParleyInn and its executable parinn"
status: proposed
summary: >-
  The invariants I1 to I10, PRV-10 to PRV-13 and the rest of the SRS name
  the product ParleyInn instead of Cairn, and the executable and crate are
  parinn. No invariant or requirement changes meaning. Storage and protocol
  spellings that existing records and peers read keep the cairn name. The
  security reviewer's approval is recorded here, as ENG-29 asks.
---
# ADR-2610221545: The product is named ParleyInn

## Context

[Pull request 66](https://github.com/jeduden/cairn/pull/66) renames the
product from Cairn to ParleyInn and its executable and core crate from
`cairn` to `parinn`. The rename edits the invariants in
`docs/srs/invariants.md` (§1.3), which CLAUDE.md and AGENTS.md include. It
also edits the rows of PRV-10, PRV-11, PRV-12 and PRV-13. ENG-29 marks those
four for an I2 security review.

ENG-29 lets an invariant or one of those requirements change only with an
accepted decision record. That record names a human security reviewer's
approval. The wording of an invariant is part of the contract. So a pure
rename counts as a change to it.

## Decision

The product's name in prose is ParleyInn, and the executable, crate, commands,
release assets and MCP server name are `parinn`.

Each of I1 to I10 keeps its meaning. Only the product name in its text changes,
and with it the line wrapping of the include copies. PRV-10 to PRV-13 change
only the product name. The reach and confinement gates (SEC-01, ENG-16) and the
I10 determinism gate follow the renamed crate and binary and are not loosened.
The I2 review these four requirements need before they ship (ENG-29) is
unchanged by this record.

Some spellings are read by existing records, peers or tooling. They keep the
`cairn` name. These are the home `~/.cairn`, `CAIRN_HOME` and its sibling
variables, `cairn:` addresses, the `Cairn-Room:` and `Cairn-Link:` trailers,
`.cairn.toml` and `logs/cairn.jsonl`. The payload, node-identity, key-store,
search-index and Keychain labels keep it too. Renaming them changes what an
existing home or peer resolves. That belongs to its own decision, with a
format version.

## Alternatives

- **Keep the name Cairn.** No edit to an invariant. But the stakeholder
  chose the new name.
- **Rename the storage and protocol spellings too.** One name everywhere.
  But it changes derived on-disk names and keychain accounts without a
  format bump. It also contradicts
  [ADR-2610171300](ADR-2610171300-key-stores.md), which names the Keychain
  service `cairn`. The stakeholder noted there are no installations today.
  So this stays open for a later record.

## Consequences

- Scripts and hooks that call `cairn` must call `parinn`; the Unreleased
  section of CHANGELOG.md says so (ENG-23).
- The domain model, SRS and ADRs still carry some "Cairn" mentions. Moving them
  to ParleyInn follows the domain-model agent's ruling.
- Status stays `proposed` until the named human security reviewer, the
  stakeholder, approves. Setting it to `accepted` records that approval.
