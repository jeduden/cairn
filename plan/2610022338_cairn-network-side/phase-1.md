---
n: 1
title: "The standalone lane experience, with no network"
status: "🔲"
result: false
---
# Phase 1: the standalone lane experience, with no network

Requirements. None closes yet: plan 2610012322's SRS change names the
lane, lifts NG5 and scopes peering first. This phase fixes the
experience and the test approach later phases copy.

BDD coverage: a scenario per behavior once the SRS ids exist. Here the
gates are a test against the built binaries and the stakeholder's
walk-through.

Experience first. One view holds three things side by side: the
harness of the agent being worked with, live (prompts, tool calls,
permission requests, output); the work results; and the other
harnesses on the machine, each a live tile that opens full size. The
lane's timeline shows the conversation, each edit as a diff, each
tool run and its result, and who did what: agent or person.

Standalone means no network at all. The UI reads the local store and
opens no listening socket (SEC-01): a terminal UI or a desktop app,
not a web server on a loopback port. Harnesses connect directly
through the same store, so two of them exchange lane events with no
UI running, and the UI joins later and shows the same lanes. The view
is built against recorded lanes first, so the design can be judged
before any live harness is wired in.

RED: a test runs under a network-deny sandbox with `HOME` and
`CAIRN_HOME` in a temporary directory (ENG-14). It starts two
harnesses on separate worktrees of one clone. It fails until both
lanes, their edits and their results appear in one view, the view
updates as each harness writes, and nothing opens a socket.

GREEN sites: the lane view, the local event feed it reads, and the
recorded-lane fixtures.

Gate: the RED test passes against the built binaries. The stakeholder
walks through the view and signs off the design. Only then does phase
2 add peer to peer.
