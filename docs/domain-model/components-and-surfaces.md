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
  - **Core (B0):** the hook handlers and the CLI, each harness adapter's
    transcript and hook part among them, the MCP server, the kernel worker, the
    TUI, and everything that builds what reaches the model.
  - The **harness adapter** is no component: Cairn's code for one harness, split
    between them, its transcript and hook part in the core (it parses, opens no
    socket, starts no process); its run part in the launcher, which starts,
    hosts and controls runs, pauses them at the harness prompt, records sandbox
    state and carries in only text the core built.
  - **Room-view component (B1):** serves the room view on loopback, or on a
    local endpoint only the same OS user can reach.
  - **Launcher (B1):** `cairn launch`, which starts, hosts and controls runs
    through each harness adapter's run part, carrying the core's text into the
    harness input.
  - **Peer component (B2):** replicates segments with peers and serves paired
    phones.
  - **Publish component (B3):** read-only publishing, and the **git carrier**,
    which keeps segments in a namespaced location of the principal's remote.
  - **Bridge component (B3):** outbound exchange with hosts the node's principal
    names, through three **bridges**: the **forge bridge** (reads pull requests
    and reviews), the **CI carrier** (fetches CI attestations) and the
    **notification bridge**.
- **Boundary**: One of B0 core, B1 machine, B2 peer and B3 public (I4).
  Unqualified, "boundary" means a network boundary; any other boundary is
  qualified, such as a span boundary or a crate boundary.
- **Room view**: What shows rooms to a person: the browser, through the
  room-view component, and the TUI, the CLI, the paired phone and the **harness
  strip** (a status line the harness shows) as reduced clients that say what
  they leave out (VIEW-14). A client of the record. Its surfaces are a closed
  set:
  - **Fleet:** every live and recorded run of the principal's rooms, grouped by
    room.
  - **Room page:** one room, with the tabs Timeline, Review and Replay, whose
    **context lens** shows what the model's context window held at an event; the
    verify and why panels, the comparison and the quarantine list, with its
    **forensic view** of quarantined content, open from it, and a foreign room
    opens in it, marked foreign.
  - **Catch up:** the one surface answering "what happened since a starting
    point" the principal picks (VIEW-08).
  - **Needs you:** the one queue of items waiting on a principal (VIEW-05).
  - **Health:** counters, failures and store locations (I6).
    - **Setup:** configuration and harness configuration, every change shown as
      a diff (I7, SEC-23).
  - **Peers:** enrolled peers and their state.
- **Outcome window**: The pane beside a room's conversation that shows one
  presentation (VIEW-22).
