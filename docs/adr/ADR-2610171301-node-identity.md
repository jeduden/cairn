---
id: ADR-2610171301
title: "The node identity is the machine id, kept in the home only as a digest"
status: proposed
summary: >-
  REC-24 names no source for the node identity. On Linux Cairn reads
  systemd's machine id, /etc/machine-id or D-Bus's copy, and the home
  keeps only its digest. An unavailable value has a digest of its own,
  so a node that never reads one changes once. macOS and Windows read
  none yet, so no change is found there until a shim reads one.
---
# ADR-2610171301: The node identity

## Context

REC-24 checks a node identity before the first append of each start:
a value outside the home that a cloned runner image, a copied volume
or a restored snapshot cannot carry over. On a change Cairn mints a new
device key and new seat keys and records a node identity change event.
An unavailable value counts as a change, though Cairn may read it again
within the hook budgets. The SRS names no source for the value. Phase 5
of plan [2609292012](../../plan/2609292012_m1-record-and-recall/plan.md)
builds the check.

The core opens no socket and starts no program but its kernel worker
(I4, SEC-01), and only the `dpapi` shim may hold unsafe code
([ADR-2610171300](ADR-2610171300-key-stores.md)). So the source must be
a file the core can read, or a call through that shim.

## Decision

- **Linux.** The node identity is systemd's machine id: the first of
  `/etc/machine-id` and `/var/lib/dbus/machine-id` that holds a value.
  A provisioning tool regenerates it for each machine an image starts
  on; a copied volume or a restored snapshot does not carry it.
- **The home keeps a digest.** It keeps SHA-256 over a label and the
  value, never the value, in `node-identity`. A home that recorded none
  yet records the one it reads and counts as unchanged.
- **Unavailable.** A value still unavailable after a second read counts
  as a change, as REC-24 says. Its digest is a fixed one of its own, so
  a node that never reads a value changes once, not at every start.
- **macOS and Windows** read none yet: no change is found there. Each
  needs a call outside the core's safe code: `gethostuuid` on macOS,
  the `MachineGuid` registry value on Windows. Both could join the
  `dpapi` shim, which then becomes the platform shim. That widens
  ruling 2's one listed shim, so it waits for the stakeholder.
- **Tests.** A development build reads the value from the file
  `CAIRN_TEST_NODE_IDENTITY` names, as it reads managed policy from
  `CAIRN_TEST_MANAGED_POLICY`; a release build never does.

## Alternatives

- **The hostname.** Containers and cloned images share or reuse one,
  and a person can rename a machine without moving the home.
- **A random id kept in the home.** A copied home carries it, which is
  the case REC-24 exists to catch.
- **The boot id.** It changes at every boot, so every restart would
  mint new keys.

## Consequences

- A container started without a machine id changes once, then keeps
  its keys while the value stays unavailable. A copy of that home on
  another machine without one is not found changed; the deployment
  documentation should require a machine id per runner.
- Until macOS and Windows read a value, REC-24's check holds on Linux
  alone, and `@REC-24`'s outline passes only there.
