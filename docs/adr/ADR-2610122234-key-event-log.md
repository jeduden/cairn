---
id: ADR-2610122234
title: "A node's keys form a key event log anchored by recovery keys"
status: proposed
summary: >-
  PRV-12 makes a node's device keys, or a token-key-only node's token
  keys, a hash-linked key event log whose keys commit to an offsite
  recovery key, closing OQ-52. A process that can append to the store
  can no longer make its own writers `witnessed`. Proposed until the
  security reviewer approves it (ENG-29).
---
# ADR-2610122234: A node's keys form a key event log

## Context

[ADR-2610112004](ADR-2610112004-derived-artifacts-by-content.md) made
origin derive from the record (PRV-13). An event reads `witnessed` when a
key of the node's own certified its writer's seat, and SEC-10's key events
say which keys those are. A key event is signed only by the keys it names,
so a process that can append to the store can record one naming its own
key and make its writers trusted (I2). OQ-52 tracked that gap.

A first design had a key event log with recovery keys. It followed
KERI's pre-rotation ([KERI][keri], [identity-keys research][ik]). The
domain-model agents, the personas and the test-engineer reviewed it.
Their findings and the stakeholder's rulings of 12 October 2026 shape
this record:

- A key lost and minted anew keeps the earlier writers `witnessed`;
  only the new key's writers wait.
- A headless node can be confirmed from another device of its principal,
  or by managed policy, without a terminal on the node.
- An unconfirmed log, a duplicity or a recovery key still on the node
  shows in the Needs you queue, Catch up and `cairn doctor`, not only in
  `cairn status`.

[keri]: https://trustoverip.github.io/kswg-keri-specification/
[ik]: ../../research/notes/identity-keys/identity-keys.md

## Decision

PRV-12 is added. Key events form a key event log in which each key
commits, by digest, to a recovery key:

- **Inceptions and rotations.** An inception (`minted`,
  `token key adopted`, `minted anew`) starts a log. A rotation
  (`rotated`, `recovered`) carries the committed recovery key, is signed
  by it and names a freshly minted key, never the recovery key itself.
- **The current log.** The genesis log is the one the device seat that
  created the personal room started. The current key event log follows
  it through its rotations and the inception that replaces its newest
  key. "Current" and "newest" come from hash links, never from a clock
  or arrival order.
- **Counting a later log.** A log other than the genesis log certifies
  no `witnessed` writer until one of these happens:
  - a `recovered` rotation joins it;
  - `cairn key-event-log confirm` names its inception, at the
    controlling terminal or, once PRV-10 ships, from another device of
    the principal;
  - for a token key, the same issuing device key, or one its rotations
    led to, made its token certificate;
  - managed policy marks the node ephemeral.
- **Lost keys.** A `minted anew` carries the chain head of the log it
  replaces. Seat certificates whose certificate records lie at or before
  that head keep counting, so earlier writers stay `witnessed`.
- **Duplicity.** Two newest key events of one log, or two inceptions
  replacing one key, stop `witnessed` after them. Only
  `cairn key-event-log choose`, or a rotation that voids both inceptions,
  resolves it.
- **Robustness.** Confirm and choose take effect whatever their own
  writer derives. Recording key events is serialized across a home's
  processes. A locked key store is never a missing key.
- **The recovery key.** `cairn recovery-key write`, a widening act at the
  controlling terminal, writes it once, encrypted under a
  passphrase-derived key or to a recipient key managed policy names, to
  a new file outside every worktree and the Cairn home. Its principal act
  never names the file.
- **Surfacing.** While a log is unconfirmed, a duplicity stands or a
  recovery key is on the node, `cairn doctor` exits non-zero, the Needs
  you queue holds a Q2 item, Catch up lists the key events, and restore
  blocks name each withheld pin with its reason.

SEC-10, SEC-27, REC-24, PRV-01, PRV-13, ADM-06, ADM-07, PEER-06, the CLI
table, ENG-29's list and the domain model follow. OQ-52 is closed.

The stakeholder, `@jeduden`, is to approve this record as security
reviewer (ENG-29) by a comment on the pull request that adds it, naming
the record's id and the pull request's head commit. The approval covers
PRV-12 and the requirements listed above as worded there.

Approval: pending.

### Security analysis (author)

This is the author's analysis for the security reviewer, not the review
itself. Each row names what must hold; the PRV-12 scenario, still
`@pending`, is to prove it.

| Attack                                                           | What answers it                                                                  |
| ---------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| a process forges a rotation to add its own key                   | a rotation needs the recovery key, which is off the node                         |
| a process appends a `minted anew` to make its writers trusted    | a later log certifies no `witnessed` writer until confirmed or joined            |
| the same forgery strips the principal's pins                     | earlier certificate records keep counting; withheld pins are named, never silent |
| a node clone keeps its source's writers `witnessed`              | a node identity change starts a new log; the clone's keys are no longer its own  |
| a retired or revoked key certifies new seats                     | certificate records count only at or before the chain head that replaced the key |
| a peer or bundle delivers key events naming this node's key      | only key events this node recorded count                                         |
| ten processes mint at once and cause a duplicity                 | recording key events is serialized across a home's processes                     |
| a locked key store turns into a lost key                         | a locked store is never a missing key; appends wait at an ingest marker          |
| the recovery key leaks into a backup, log or the model's context | its own key-store entry, never backed up or logged; written only at a terminal   |

What stays open: confirm and choose are widening principal acts. Where
the principal or managed policy requires an authenticator, each carries
a presence proof bound to its digest (OWN-11). Where none is required,
a process that can write the home and use the device seat's key can
forge one, as it can every widening act: residual risks R3 and R7
(§6.1), which only a sandbox removes. Also open is the window before a
recovery key leaves the node, which `cairn doctor` and the Needs you
queue show.

## Alternatives

- **Simple key events, as SEC-10 had them.** Any process that can append
  could add its own key (OQ-52).
- **Demote every earlier writer after a lost key.** Strictest, but one
  stray process or a locked key store would strip every agent's pins
  until a person acts. The stakeholder rejected it.
- **Confirm only at the node's own terminal.** Headless runners, cloud
  sandboxes and home servers would never return to `witnessed`.
- **Resolve duplicity by any new inception.** Anyone who can append
  could choose which side wins.

## Consequences

- M1 keeps the device key's key event log, its certificate records and
  its recovery key, and ships `cairn recovery-key write`,
  `cairn key-event-log confirm` and `choose`.
- M8 carries a token key's recovery key in its access token; PRV-10
  brings confirmation from another device.
- Restore blocks, the Needs you queue, Catch up and `cairn doctor` gain
  the states PRV-12 names.
