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
tool run and its result, and who did what: agent or person. Requests
for input surface at the top, as dots' Activity view does. Background
agents appear as child threads you can open and steer. A result shows
what checked it, a claim, a local run or the canonical CI, because a
completed run is not a verified one. Standalone shows claims and own
runs; witness runs and CI attestation come later (LANE-05).

Standalone means Cairn sends nothing off the machine; what an agent
recalls travels to its model provider as it does today. The UI is a web page in
the browser, served on a loopback-only port with a token per launch
and Host and Origin checks. Harnesses connect directly
through the same store, so two of them exchange lane events with no
UI running, and the UI joins later and shows the same lanes. The view
is built against recorded lanes first, so the design can be judged
before any live harness is wired in.

RED: a test runs in a sandbox that denies all but loopback, with `HOME` and
`CAIRN_HOME` in a temporary directory (ENG-14). It starts two
harnesses on separate worktrees of one clone. It fails until both
lanes, their edits and their results appear in one view, the view
updates as each harness writes, and no socket is reachable from off
the machine and no outbound connection is made. The same test holds
every NFR-01 hook budget with the view open and ten harnesses writing,
and runs a recorded-weekend fixture through Catch up, search to replay
and lane verify.

GREEN sites: the lane view, the local event feed it reads, and the
recorded-lane fixtures.

Gate: the RED test passes against the built binaries. The stakeholder
walks through the view and signs off the design. The listening socket
stays a prototype, behind a build tag that release builds exclude,
until the ENG-29 security review of I4 and SEC-01 is accepted. Only
then does phase 2 add peer to peer.
