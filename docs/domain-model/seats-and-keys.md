---
title: "Seats and keys"
order: "06"
summary: >-
  Seats and the keys behind them: run and device seats, room and seat ids, seat, device, principal and token keys, certificates, access tokens, authenticators and CI keys.
---
# Seats and keys

- **Seat**: One run's or one device's place in one room, of the **seat kind**
  `run` or `device` its seat certificate names (PRV-11). The unit of membership,
  signing and authorship. A seat is a member only while its **add** stands, the
  act that placed it: its create room act, a join its principal asked for or
  accepted that the room's admission admitted, or a device seat's join without
  admission; a run's personal-room seat, and a paired phone's or a newly minted
  key's device seat there, are members with no add, and the seat ingest starts
  is a member while its run seat is, sharing its add, one for both, where it has
  one (LANE-25, REC-19).
- **Run seat**: A run's seat. A witnessed run's run-seat key, but for the seat
  ingest starts, lives only in the memory of its harness session's MCP server,
  which keeps those of the main run and its subagents' runs and seals their
  writers (SEC-10). What `cairn ingest` appends for a witnessed run goes to the
  one run seat ingest starts beside each run seat it extends, in its room,
  naming it, sharing its add, where it has one, and taking over its role,
  certified only by the key that certified that run seat or the key a rotation
  replaced it with, by a seat certificate naming it so; any other seat naming a
  run seat shares no add or role (REC-19). The core keeps that seat's key, and
  an ingested run's, like a device seat's key and seals their writers (SEC-10).
  A run-seat key is lost with the MCP server that holds it, and a run that
  continues on a harness resume or a restart does so on a new run seat (Seat
  key; open: OQ-43). A witnessed run records nothing until its MCP server holds
  its run-seat key, and one no MCP server serves is ingested (REC-19).
- **Device seat**: A principal's seat for one device, a node or a paired phone.
  The one seat kind for acting without a run. Its room acts are taken only at a
  principal surface, as principal acts are, but those its node takes on its own
  (Personal room, Join) and the facilitator's (SEC-32); an agent's tool call
  acts only from its run seat (OWN-12).
- **Room id**: A room's id, derived from the creating seat's first key and 128
  random bits its create room act fixes, never chosen; a create room act whose
  seat's first key does not derive the room id it names is void and shown
  (LANE-01).
- **Seat id**: A seat's id, derived from the room id and the seat's first key.
  Nobody chooses it.
- **Seat key**: A seat's one current key. It signs the seat's room acts and
  seals its writer. A rotation, signed by the old and the new key, keeps the
  seat's id and writer (SEC-27). Any other key minted in its place starts a new
  seat and writer that names the old one, inherits no add, role or appointment
  and is a member only as Seat says (SEC-27).
- **Device key**: A device's key, certified by a principal key's **device
  certificate**, with a **device scope** (the kinds of principal act it may
  sign, and of post and pin its seats may write) and a maximum rule level. It
  signs principal acts and expire acts once PRV-10 ships, and a blind peer holds
  a room's segments encrypted to its principals' device keys or a token-key-only
  node's token key (PEER-12). Before PRV-10 ships no principal key certifies
  one, so a device seat's pins qualify only on its own node (PIN-10).
- **Principal key**: A principal's root key, kept offline or in a
  platform-protected key store and never by a Cairn component, which certifies
  its device keys and may certify a service account's principal key by a
  **service-account certificate**, marking whether that account relays text
  others wrote (PRV-10).
- **Token key**: A key a device key certifies by a **token certificate**, which
  may stand between a device key and a seat key for an ephemeral node, limited
  to an access token's rooms, the rooms it may create, its node's own personal
  room and its expiry (PEER-06, PRV-10). A node with only a token key signs no
  principal or expire acts, and every pin it writes restores only once stamped
  (PEER-06, PRV-10), though its own events are trusted on that node as that
  node's trusted sources.
- **Seat certificate**: A device key's or token key's signature over a seat key,
  scoped to the seat's room and naming its seat kind, that the seat key
  countersigns (PRV-10, PRV-11). A node's **key set** is the keys, certificates
  (a service-account certificate once countersigned), revocations, and recorded
  managed-policy listings and their removals it holds (I10), a listing only as
  Service account says.
- **Access token**: A short-lived credential a principal mints to enroll a
  device or peer, certify an ephemeral node's seats or carry an invite link
  (PEER-06, PRV-10, LANE-18). Its expiry ends it by the expire act the minting
  node records, or before PRV-10 ships only as that node's refusal of every
  later use of it (PEER-05). "Token" is always an access token, a model token,
  an access token's token key, a token certificate or a token-key-only node.
- **Authenticator**: A hardware-backed key that gives presence proofs (OWN-11).
- **CI key**: A key the room's owner enrolled in the room to sign CI
  attestations.
