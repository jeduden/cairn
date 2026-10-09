---
title: "Places"
order: "03"
summary: >-
  Where Cairn's state lives and is shared: homes, nodes, devices and paired phones, sandboxes, rooms and their kinds, visibility, peers and blind peers.
---
# Places

- **Home**: The directory with one principal's Cairn state on a node
  (`CAIRN_HOME`, default `~/.cairn`), to which the keys SEC-10 keeps in a
  platform key store, named per home, or only in an MCP server's memory are
  bound, belonging to one OS user and optionally bound to a **home id**
  (`home.id`) that the environment must supply to open it (SEC-03). The unit of
  isolation (I8).
- **Node**: One home on one machine, container or cloud environment, for one
  principal. Its device key signs principal acts and expire acts once PRV-10
  ships; an **ephemeral node**, one in a short-lived cloud environment, may have
  only a token key, a **token-key-only node**. On an ephemeral node, PEER-01's
  environment variable, a structural event, stands in for turning on its peer
  component; minting its access token is the widening act. A token-key-only
  node signs no principal act, so only managed policy makes a change there
  that ADM-04 holds for acceptance (open: OQ-06). Minting its access token
  is also its principal's ask for each of its runs to join, at the run's
  start, every room the token names to continue, as that room's admission
  permits, and the token carries that ask and those rooms' qualifying pins
  (LANE-23, PEER-06), so those pins restore to its runs from their first
  restore block; every pin it writes restores only once stamped. Its **node
  identity** is a value outside the home that a cloned image or a restored
  snapshot cannot carry over (REC-24). A **node clone** is a copy of a home
  started where its node identity differs (a cloned image, a copied volume or a
  restored snapshot): a new node, with its own device key, or a token key, and
  its own device seat, that mints new seat keys and shares the carried-over
  personal room (REC-24). On it, as on any node whose node identity changed,
  the events recorded before the change are another node's: they count as
  received, a `witnessed` one read as `peer`, and are trusted only as PRV-02
  trusts received events (I8). The events a backup restore reinstates count as
  received in the same way, even on the node that made the copy. For both,
  ADM-06 says which principal acts keep effect on the node, a quarantine it
  keeps being its principal's own (open: OQ-48). A backup restore reinstates
  the copy's segments and payloads, never its derived artifacts (I10).
- **Device**: A node or a paired phone. A node's device key certifies the seats
  it uses (PRV-11), a token-key-only node's token key in its place (PRV-10); a
  paired phone's rests on OQ-39.
- **Paired phone**: A device with no home, limited to reading and to allowing or
  denying held permission requests within its device scope, reaching its node
  over B2. It signs with its own device key and seals its device seat's writer
  with that seat's key; the node it pairs with holds the writer. Where its keys
  and writer live under I8 rests on OQ-39.
- **Sandbox**: A confinement around a run that blocks some residual risks
  (OWN-22).
- **Room**: Where an intent is worked on: at most one intent, a **conversation**
  (its ordered posts), seats, pins, and branches in any repositories, each named
  by a branch link. The unit Cairn shows and shares.
- **Personal room**: A principal's private room on each of its nodes, created as
  ADM-02 says, or carried over to a node clone, which shares it. It stays
  private: no invite, admission or visibility change applies to it (LANE-17),
  and its segments go only to its principal's other nodes (Peer). Its create
  room act is the device seat's add there. Every run its node records has a seat
  in it from its first event, without a join, and every event or pin that
  belongs to no other room goes there. Whether a room counts an act recorded
  here for it, by a principal with no member seat in it, rests on OQ-41. Unlike
  a node clone, a backup restore of a copy another node made does not share that
  node's personal room: this node's runs get no seat in it from their first
  event, and how their recall may reach it is open (OQ-45).
- **Foreign room**: A room this node holds, other than only as a blind peer,
  that its principal neither owns nor has a seat in, such as the room of an
  imported bundle or a room every seat of its principal has left or lost to a
  kick or a bar. Recall marks every item it returns from it untrusted, whatever
  the item's trust level or trust grant covers its keys (RCL-10), and leaves it
  outside every extended recall scope unless named in the call; only its
  principal's own device-seat pins, the versions it stamped and
  the pins a leave, kick or bar keeps there stay trusted and
  keep restoring to the runs Pin names: those that have had a seat in it
  during the run, or whose agent's earlier runs had one (PIN-10,
  REC-02). A room the principal owns or
  has a seat in is never foreign. Yet a leave, kick or bar that makes a room
  foreign drops no pin: what qualified there for its principal's agents, a
  trust grant's pins included, keeps restoring as it stood, in causal order,
  before that act, until that principal's own act changes it (PIN-10), or
  until a certificate it qualified through is revoked (PRV-10) or the trust
  grant it qualified by covers nothing (Trust grant), as in any other room.
  But after a change of ownership, the intent restores in a foreign room only
  as Intent says (LANE-11).
- **Principal's rooms**: The rooms a principal owns or has a seat in.
- **Visibility**: Whether a room is private, shared with the room's principals,
  published or stored on blind peers (LANE-17). Changing it is a widening
  principal act of the owner; a node's principal publishes a room, or enrolls a
  blind peer for it, only as its visibility allows.
- **Peer**: Another node this node's principal enrolled by key and exchanges
  sealed ranges with through the peer component (B2). Being a peer never makes
  content trusted (PRV-02). A node serves a token-key-only node only the
  segments of the rooms its token key is limited to and the cross-room posts
  they show (Token key). A node serves a room's segments only to the peers
  whose principal has a seat in that room and to the blind peers holding it;
  from any other room but a personal room it serves a peer only a cross-room
  post, by its address, when a seat of that peer's principal in the room the
  post targets asks for it
  (PEER-02, LANE-29). Whether it so serves a principal act a room shows by
  address, or a personal room's cross-room post to another principal's peer,
  rests on OQ-41. Exchanging the sealed ranges each may receive is
  **sync**.
- **Blind peer**: A peer that holds a room's segments without any key they are
  encrypted to, so it stores and serves them encrypted and reads none of them
  (PEER-12); a room it holds only so is never foreign to it.
