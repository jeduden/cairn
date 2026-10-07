---
title: "Seats and keys"
order: "06"
summary: >-
  Seats and the keys behind them: run and device seats, room and seat ids, seat, device, principal and token keys, certificates, access tokens, authenticators and CI keys.
---
# Seats and keys

- **Seat**: One run's or one device's place in one room, its **seat kind** `run`
  or `device`; a seat key minted anew starts another (Seat key). The unit of
  membership, signing and authorship. A run's personal-room seat exists from its
  first event, and a paired phone's device seat in the personal room of the node
  it pairs with from its pairing, each with no add; every other seat keeps its
  place while its **add** stands: the act that placed it, its create room act, a
  join its principal asked for or accepted that the room's admission admitted,
  or a device seat's join without admission (LANE-25).
- **Run seat**: A run's seat. Its key lives only in the memory of that run's MCP
  server, which seals its writer (SEC-10); an ingested run's seat key is kept
  like a device seat's key, and the core seals its writer.
- **Device seat**: A principal's seat for one device, a node or a paired phone.
  The one seat kind for acting without a run. A token-key-only node's device
  seat is certified by its token key, and that node signs no principal or expire
  acts.
- **Room id**: 128 random bits its create room act fixes, never chosen
  (LANE-01).
- **Seat id**: A seat's id, derived from the room id and the seat's first key.
  Nobody chooses it.
- **Seat key**: A seat's one current key. It signs the seat's room acts and
  seals its writer. A rotation, signed by the old and the new key, keeps the
  seat's id and writer (SEC-27). A key minted because the node changed, a clone
  or a backup restore (REC-24, ADM-06), starts a new seat and writer, which
  names the old one and inherits no add, role or appointment: it joins as any
  seat does (LANE-23), and roles and appointments are assigned again.
- **Device key**: A device's key, certified by a principal key's **device
  certificate**, with a **device scope** (the kinds of principal act it may
  sign, and of post and pin its seats may write) and a maximum rule level. It
  signs principal acts and expire acts once PRV-10 ships, and a room's segments
  sent to a blind peer are encrypted to its principals' device keys (PEER-08).
  Before PRV-10 ships no key certifies a device seat, and its pins qualify only
  on its own node.
- **Principal key**: A principal's root key, kept offline or in a
  platform-protected key store, which certifies its device keys and may certify
  a service account's principal key by a **service-account certificate**
  (PRV-10).
- **Token key**: A key a device key certifies by a **token certificate**,
  limited to an access token's rooms and expiry, which may stand between a
  device key and a seat key for an ephemeral node (PRV-10). A node with only a
  token key signs no principal or expire acts; every pin it writes restores only
  once a principal stamps it from one of its devices whose device scope allows
  it, though its own events are trusted on that node as that node's trusted
  sources.
- **Seat certificate**: A device key's or token key's signature over a seat key,
  scoped to the seat's room, so every seat key chains to a principal key. A
  node's **key set** is the keys, certificates and revocations it holds (I10).
- **Access token**: A short-lived credential a principal mints to enroll a
  device or peer, seat an ephemeral node or carry an invite link (PEER-05,
  PRV-10, LANE-18). "Token" is always an access token, a model token, an access
  token's token key, a token certificate or a token-key-only node.
- **Authenticator**: A hardware-backed key that gives presence proofs (OWN-11).
- **CI key**: A key a principal enrolled to sign CI check results.
