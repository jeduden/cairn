---
title: "Seats and keys"
order: "06"
summary: >-
  Seats and the keys behind them: run and device seats, room and seat ids, seat, device, principal and token keys, certificates, access tokens, authenticators and CI keys.
---
# Seats and keys

- **Seat**: One run's or one device's place in one room, its **seat kind** `run`
  or `device`, named by its seat certificate, made when the seat starts
  (PRV-11), the only source Cairn shows a seat's kind from; a seat key minted on
  a node identity change, a node clone or a backup restore starts another (Seat
  key). The unit of membership, signing and authorship. A run's personal-room
  seat exists from its first event, a paired phone's device seat in the personal
  room of the node it pairs with from its pairing, and a device seat a newly
  minted key starts in the personal room from its first event, each with no add;
  every other seat keeps its place while its **add** stands: the act that placed
  it, its create room act, a join its principal asked for or accepted that the
  room's admission admitted, or a device seat's join without admission; the seat
  ingest starts keeps its place while the run seat it names does, sharing that
  seat's add, one for both, where it has one (LANE-25, REC-19).
- **Run seat**: A run's seat. A witnessed run's run-seat key, but for the seat
  ingest starts, lives only in the memory of its harness session's MCP server,
  which keeps the run-seat keys of the main run and its subagents' runs and
  seals their writers (SEC-10); what `cairn ingest` appends, whether or not that
  server still runs, goes to the one seat and writer ingest starts beside that
  run seat, which every later `cairn ingest` extends and the core seals, naming
  the run seat, in the same room, sharing its add, where it has one, and taking
  over its role, so a kick or leave of either seat ends that one add for both
  (REC-19); that seat is a run seat whose key the core keeps like a device
  seat's key; an ingested run's seat key is kept like a device seat's key, and
  the core seals its writer.
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
  seat's id and writer and goes to that seat's own writer (SEC-27). A key minted
  because the node identity changed, a node clone or a backup restore (REC-24,
  ADM-06), starts a new seat and writer that names the old one and inherits no
  add, role or appointment: outside the personal room it joins as any seat does
  (LANE-23), and roles and appointments are assigned again; a personal-room seat
  is a member from its first event. The seat ingest starts shares the add, where
  it has one, and takes over the role instead (Run seat).
- **Device key**: A device's key, certified by a principal key's **device
  certificate**, with a **device scope** (the kinds of principal act it may
  sign, and of post and pin its seats may write) and a maximum rule level. It
  signs principal acts and expire acts once PRV-10 ships, and a room's segments
  the git carrier carries or a blind peer holds are encrypted to its principals'
  device keys, or a token-key-only node's token key (PEER-08, PEER-12). Before
  PRV-10 ships no principal key certifies a device key, so a device seat's pins
  qualify only on its own node.
- **Principal key**: A principal's root key, kept offline or in a
  platform-protected key store and never kept by a Cairn component (the
  principal's own key tooling signs with it), which certifies its device keys
  and may certify a service account's principal key by a **service-account
  certificate**, which marks whether that account relays text others wrote
  (PRV-10).
- **Token key**: A key a device key certifies by a **token certificate**, which
  may stand between a device key and a seat key for an ephemeral node, limited
  to an access token's rooms, the rooms it may create, its node's own personal
  room and its expiry (PRV-10). A node with only a token key signs no principal
  or expire acts; every pin it writes restores only once a principal stamps it
  from one of its devices whose device scope allows it, though its own events
  are trusted on that node as that node's trusted sources.
- **Seat certificate**: A device key's or token key's signature over a seat key,
  scoped to the seat's room, naming the seat kind, `run` or `device`. The node's
  device key makes one for every seat the node uses from the first release
  (PRV-11); once
  PRV-10 ships every seat key chains to a principal key. A node's **key set** is
  the keys, certificates and revocations it holds (I10).
- **Access token**: A short-lived credential a principal mints to enroll a
  device or peer, certify an ephemeral node's seats or carry an invite link
  (PEER-06, PRV-10, LANE-18). "Token" is always an access token, a model token,
  an access token's token key, a token certificate or a token-key-only node.
- **Authenticator**: A hardware-backed key that gives presence proofs (OWN-11).
- **CI key**: A key the room's owner enrolled in the room to sign CI
  attestations.
