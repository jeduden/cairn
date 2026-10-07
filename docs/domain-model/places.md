---
title: "Places"
order: "03"
summary: >-
  Where Cairn's state lives and is shared: homes, nodes, devices and paired phones, sandboxes, rooms and their kinds, visibility, peers and blind peers.
---
# Places

- **Home**: The directory containing all of one principal's Cairn state on a
  node (`CAIRN_HOME`, default `~/.cairn`), belonging to one OS user and
  optionally bound to a **home id** (`home.id`) that the environment must supply
  to open it (SEC-03). The unit of isolation (I8).
- **Node**: One home on one machine, container or cloud environment, for one
  principal. Its device key signs principal acts and expire acts once PRV-10
  ships; an **ephemeral node**, one in a short-lived cloud environment, may have
  only a token key, a **token-key-only node**. Its **node identity** is a value
  outside the home that a cloned image or a restored snapshot cannot carry over
  (REC-24). A **node clone** is a copy of a home started where its node identity
  differs (a cloned image, a copied volume or a restored snapshot): a new node,
  with its own device key, or a token key, and its own device seat, that mints
  new seat keys and shares the carried-over personal room (REC-24). The node's
  principal is the principal whose home it is.
- **Device**: A node or a paired phone. Once PRV-10 ships, a device key
  certifies its seats; a token-key-only node's token key does so in its place.
- **Paired phone**: A device with no home, limited to reading and to allowing or
  denying held permission requests within its device scope, reaching its node
  over B2. It signs with its own device key and seals its device seat's writer
  with that seat's key; the node it pairs with holds the writer.
- **Sandbox**: A confinement around a run that blocks some residual risks
  (OWN-22).
- **Room**: Where an intent is worked on: at most one intent, a **conversation**
  (its ordered posts), seats, pins, and branches in any repositories, each named
  by a branch link. The unit Cairn shows and shares.
- **Personal room**: A principal's private room on each of its nodes, created by
  the principal's `cairn install` as the first act of its device seat there, or
  carried over to a node clone, which shares it. It stays private: no invite,
  admission or visibility change applies to it (LANE-17). Its create room act is
  the device seat's add there. Every run its node records has a seat in it from
  its first event, without a join, and every event or pin that belongs to no
  other room goes there.
- **Foreign room**: A room this node holds, other than only as a blind peer,
  that its principal neither owns nor has a seat in, such as the room of an
  imported bundle or a room every seat of its principal has left or lost to a
  kick or a bar. Recall marks every item it returns from it untrusted, whatever
  the item's trust level or trust grant covers its keys (RCL-10), and leaves it
  outside every extended recall scope unless named in the call; only its
  principal's own device-seat pins and the versions it stamped stay trusted and
  keep restoring, to runs that had a seat in it. A room the principal owns or
  has a seat in is never foreign.
- **Principal's rooms**: The rooms a principal owns or has a seat in.
- **Visibility**: Whether a room is private, shared with the room's principals,
  published or stored on blind peers (LANE-17). Changing it is a widening
  principal act of the owner; a node's principal publishes a room, or enrolls a
  blind peer for it, only as its visibility allows.
- **Peer**: Another node this node's principal enrolled by key and exchanges
  sealed ranges with through the peer component (B2). Being a peer never makes
  content trusted (PRV-02). Exchanging the sealed ranges each may receive is
  **sync**.
- **Blind peer**: A peer that holds a room's segments without any key they are
  encrypted to, so it stores and serves them encrypted and reads none of them
  (PEER-12); a room it holds only so is never foreign to it.
