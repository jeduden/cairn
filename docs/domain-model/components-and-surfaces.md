---
title: "Components and surfaces"
order: "10"
summary: >-
  Cairn's components inside their network boundaries, the room view's surfaces and the outcome window.
---
# Components and surfaces

- **Component**: A part Cairn plays, inside exactly one network boundary (I4).
  The set is closed, listed here and in §6.3; how the components ship is open
  (OQ-32), and a new one is a model change and a §6.3 row.
  - **Core (B0):** the **hook handlers** (answering hooks), the **CLI**
    (`cairn`, but for `cairn ui`, `cairn launch` and each B1 to B3 component's
    own entry point, which the person, a service manager or an ephemeral node's
    entrypoint starts and which runs only while the act turning it on stands:
    for the room-view component and the launcher, the configuration acceptance
    that turns it on, until a configuration turning it off applies, which needs
    no acceptance and which `cairn configuration accept` records as the cut act
    turning it off (ADM-04, SEC-01); for the bridge component, a bridge enabled
    for a host, while any stands), each harness adapter's transcript and hook
    part among them, the **MCP server** (the MCP tools of §9.2, one per harness
    session, serving its runs), the **kernel worker** (running the kernel's
    executions), the **TUI** (the terminal room view), the **commit hook**
    (LANE-28, run as the CLI), and everything that builds what reaches the
    model.
  - The **harness adapter** is no component: Cairn's code for one harness, split
    between the core and the launcher: its transcript and hook part in the core
    (it parses, opens no socket, starts no process); its run part in the
    launcher, which starts, hosts and controls runs, pauses them at the harness
    prompt, records sandbox state and carries in only text the core built.
  - **Room-view component (B1):** serves the browser room view on loopback only
    (SEC-20), and records the principal acts taken in it, marked with their
    principal surface (VIEW-03). A **room-view secret** makes the browser room
    view a principal surface (SEC-20): a **launch secret** it mints per launch,
    exchanged once for an **origin secret** only its own origin, port included,
    can read or send, which lasts until the component stops; a phone-scoped one
    serves a browser on the principal's phone (OWN-16).
  - **Launcher (B1):** `cairn launch`, which starts, hosts and controls runs
    through each harness adapter's run part, carrying into the harness input
    only text the core built and recorded, read from the record, never text from
    the room-view component, whose loopback connection to it carries no text for
    the model; and runs witness checks; the only component that starts programs,
    but for the core's own kernel worker (CMP-05).
  - **Peer component (B2):** exchanges segments with peers (sync) and serves
    paired phones; on a paired phone it is Cairn's only code, with the room
    view's reduced client inside it.
  - **Publish component (B3):** read-only publishing and the segment exchange of
    the **git carrier**, which keeps a room's segments in a namespaced location
    of a remote of the node's principal, only while that principal has enabled
    it for that remote and the room's owner for that room, each a widening
    principal act (PEER-08, ADM-13).
  - **Bridge component (B3):** outbound exchange with hosts the node's principal
    names, through three **bridges**: the **forge bridge** (reads pull requests,
    reviews and the checks the forge reports), the **CI bridge** (fetches CI
    attestations) and the **notification bridge**.
- **Boundary**: One of B0 core, B1 machine, B2 peer and B3 public (I4).
  Unqualified, "boundary" means a network boundary; any other boundary is
  qualified, such as a span boundary or a crate boundary.
- **Room view**: What shows rooms to a person: the browser, through the
  room-view component, the one client that shows every surface, and the TUI, the
  CLI, the paired phone and the **harness strip** (a status line the harness
  shows) as reduced clients that say what they leave out (VIEW-14). A client of
  the record. Its surfaces are a closed set, and unqualified, "surface" means
  one of them; where principal acts are taken is a principal surface:
  - **Fleet:** every live and recorded run of the principal's rooms, grouped by
    room.
  - **Room page:** one room, with the tabs Timeline, Review and Replay, the
    Replay tab's **context lens** showing what the model's context window
    contained at an event; the verify and why panels, the comparison and the
    **quarantine list** (the room's quarantine set), with its **forensic view**
    of quarantined content, open from it, and a foreign room opens in it, marked
    foreign.
  - **Catch up:** the one surface answering "what happened since a starting
    point" the principal picks (VIEW-08).
  - **Needs you:** the one queue of items waiting on a principal (VIEW-05).
  - **Health:** counters, failures and store locations (I6).
    - **Setup:** configuration and harness configuration, every change shown as
      a diff (I7, SEC-23).
  - **Peers:** enrolled peers and their state.
- **Outcome window**: The part of the Room page beside a room's conversation
  that shows one presentation (VIEW-22), never the outcome itself.
