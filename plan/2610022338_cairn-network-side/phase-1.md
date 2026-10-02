---
n: 1
title: "Two hosts exchange a segment through a relay"
status: "🔲"
result: false
---
# Phase 1: two hosts exchange a segment through a relay

Requirements. None closes yet: the SRS change from plan 2610012322
names these components first. This phase proves the shape and fixes
the test approach later phases copy.

BDD coverage: a scenario per behavior once the SRS ids exist. Here the
gate is a test that runs the built binaries, since the claim is about
processes talking.

RED: a test starts a relay on a loopback port and two `cairn-sync`
processes, each with its own `HOME` and `CAIRN_HOME` in a temporary
directory (ENG-14). Host A seals a segment. The test fails until host
B's inbox holds it within 5 s, byte for byte, with A's signature and
chain verifying. A second case flips one byte of a segment in the
relay's store and fails until B refuses it, audits the refusal and
imports nothing.

GREEN sites:

- `cmd/cairn-relay`: stores segments per origin and serves them by
  offset; it checks a segment's signature before accepting it;
- `cmd/cairn-sync`: pushes sealed segments, pulls by offset, and writes
  verified ones into the inbox directory the core reads;
- the segment and checkpoint format from plan 2610012322, following
  C2SP tlog-tiles; torchwood is tried first for signed notes.

Gate: the RED test passes against the built binaries, and the core's
import-closure test still passes unchanged.
