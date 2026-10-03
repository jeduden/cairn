---
n: 1
title: "The lane experience over two peers, with a partition"
status: "🔲"
result: false
---
# Phase 1: the lane experience over two peers, with a partition

Requirements. None closes yet: plan 2610012322's SRS change names the
lane and decides I4 first. This phase fixes the experience and the
test approach later phases copy.

BDD coverage: a scenario per behavior once the SRS ids exist. Here the
gates are a test against the built binaries and the stakeholder's
walk-through.

Experience first. A lane view shows, in one timeline, the
conversation, each edit as a diff, each tool run and its result, and
who did what: agent or person, owner or not. Results render as views
you can open, not pasted logs. Messages from anyone but the owner are
marked untrusted, and the view shows which ones an agent has read.
The view is built against recorded lanes first, so the design can be
judged before any sync exists.

RED: a test starts two peers on loopback, each with its own `HOME` and
`CAIRN_HOME` in a temporary directory (ENG-14), and no server. Both
write to one lane. The test then cuts the link, writes on both sides,
and restores it. It fails until both peers hold the same event set and
render the same lane: causal order, every author and trust class
intact. A second case flips one byte of a segment in transit and fails
until the receiver refuses it, audits the refusal and imports nothing.

GREEN sites: the lane view and the peer that exchanges segments. The
segment format comes from plan 2610012322. It follows C2SP tlog-tiles.

Gate: the RED test passes against the built binaries. The stakeholder
walks through the lane view and signs off the design. Only then does
phase 2 make sync seamless.
