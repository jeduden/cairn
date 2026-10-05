# Buzz and Cairn: rooms, repositories and trust

Scope: Block's Buzz set against Cairn's pitch v12 and SRS 2.1-draft,
on 5 October 2026. Buzz's facts come from the research notes gathered
on 2–4 October 2026 and were not fetched again; each one names its
source. Cairn is pre-implementation, so its column states the
requirement, not shipped behaviour.

2.1-draft changes the comparison. Cairn now runs a git repository per
project on each node, and the worktrees its lanes work in, and a lane
may span several repositories (LANE-23, LANE-24, LANE-25). Before that
change, git hosting was a point where Buzz led and Cairn did not
compete.

## What Buzz is

- An open-source (Apache-2.0) workspace for teams of humans and
  agents, launched on 21 July 2026. It has channels, threads, DMs and
  voice, and agents join as members —
  [SiliconANGLE](https://siliconangle.com/2026/07/21/block-launches-buzz-open-source-workspace-humans-ai-agents/);
  [Block announcement](https://block.xyz/inside/introducing-buzz-where-humans-and-agents-work-together).
- "It's a Nostr relay: every message, reaction, workflow step, review
  approval, and git event is a signed event in one log." The Rust relay
  keeps events and full-text search in Postgres, pub/sub and presence
  in Redis, and media in S3 or MinIO —
  [block/buzz README](https://github.com/block/buzz).
- "Branch as room": a feature branch opens a channel, patches land
  there as NIP-34 events, CI posts its results, and "the merge
  decision lands in the same room as the evidence". After the merge
  the channel stays as "an archived record of why the change exists" —
  [README](https://github.com/block/buzz);
  [VISION.md](https://raw.githubusercontent.com/block/buzz/main/VISION.md).
- The relay hosts git over Smart HTTP, so ordinary `git clone` and
  `git push` work; it is listed under "Works today", with the least
  mileage of any part —
  [README](https://raw.githubusercontent.com/block/buzz/main/README.md);
  [mager.co](https://www.mager.co/blog/2026-07-24-buzz-explainer/).
- Each agent has its own Nostr keypair, and a second signature binds
  it to its human owner. Claude Code, Codex and goose connect through
  `buzz-acp` — [SiliconANGLE](https://siliconangle.com/2026/07/21/block-launches-buzz-open-source-workspace-humans-ai-agents/);
  [README](https://github.com/block/buzz).
- Encryption: "TLS in transit. At-rest encryption delegated to the
  storage layer". The operator can read content, and end-to-end
  encryption is a future consideration for DMs only —
  [VISION.md](https://raw.githubusercontent.com/block/buzz/main/VISION.md);
  [NOSTR.md](https://raw.githubusercontent.com/block/buzz/main/NOSTR.md).
- `buzz-audit` is a hash chain over its entries, but an audit write is
  fire-and-forget, so a failed write does not fail the event —
  [ARCHITECTURE.md](https://github.com/block/buzz/blob/main/ARCHITECTURE.md).
- Deletion is relay-side (NIP-09 kind 5, kind 9005 for admins); no
  key is destroyed —
  [NOSTR.md](https://raw.githubusercontent.com/block/buzz/main/NOSTR.md).
- One relay holds one community; there is no federation —
  [integrated platforms note](../custom-storage-git-and-chat/integrated-platforms.md).

## The two pitches

|              | Buzz ("Introducing Buzz")                                                                 | Cairn (pitch v12)                                                                                           |
| ------------ | ----------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| Opens with   | A belief: work happens when humans and agents share a room, and that place should be open | The developer's day: five agents, a lost tab, a rule forgotten after compaction, "all tests pass" unchecked |
| Length       | About 710 words, a third on Nostr and open source                                         | 355 words: problem, screen, one claim per line                                                              |
| Landing line | "Your people, your agents, your project — all in one place."                              | "Multiplayer for you and your agents."                                                                      |

A blind panel of Cairn's nine persona agents ranked v12 first for
eight seats (mean rank 1.1) and Buzz third (2.8). The multi-machine
developer preferred Buzz for self-hosting today
([round 11](../../../plan/2610012322_cairn-for-agent-fleets/persona-review/round-11/README.md)).
The panel is Cairn's own, so it tests the pitch's shape, not the
market.

## Against Cairn

| Question             | Buzz                                                                             | Cairn (2.1-draft)                                                                                                                         |
| -------------------- | -------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| Unit                 | A room per branch of one repository                                              | A lane per intent: one branch in each repository it spans, several attempts beside each other (LANE-25)                                   |
| Code                 | The relay hosts git for the community; Smart HTTP clone and push                 | Each node runs a node repository per project and the lane worktrees (LANE-23, LANE-24); the forge still lands (NG8)                       |
| Several repositories | Not documented; one room per branch                                              | One lane spans them; landing and proof per repository; a partial landing is shown, naming what is open (LANE-25)                          |
| Worktrees            | Outside Buzz; agents run where `buzz-acp` starts them                            | The run component creates one worktree per project in a lane directory and starts the session there (LANE-24)                             |
| Keeping work         | Relay-side deletion; nothing about kept heads                                    | Every commit a lane worktree produced is kept, replaced heads included, until an owner purge (LANE-23)                                    |
| Topology             | One relay per community: Postgres, Redis, S3; no federation                      | No central service; each node runs its own repositories and record; peers by key next (P2)                                                |
| Network              | The relay serves clients over WebSocket and HTTP                                 | The core opens no socket; repository work uses the local file system only, and the run component never fetches or pushes (SEC-29, row 28) |
| Record               | Signed Nostr events and a hash-chained audit log; a failed audit write is silent | One hash-chained log per writer; every failed operation counted and audited (I6)                                                          |
| Agent identity       | Own keypair, bound to its human by a second signature                            | Each agent answers to one person; others' agents reach it only through endorsement or trust by key (I2)                                   |
| Untrusted content    | Membership scopes access; message content is not marked untrusted                | Everything not yours is untrusted, recalled only when asked, inside an envelope (I2)                                                      |
| Context for agents   | None stated                                                                      | Pins restored word for word after every compaction (I3); exact recall (RCL-03)                                                            |
| Merge gate           | Signed approval in the room, beside the evidence                                 | Signed approvals and required checks next (LANE-07, P2); the forge lands                                                                  |
| Data control         | TLS and storage-layer encryption; the operator reads content                     | Nothing leaves the machine by default; secrets redacted before storage (SEC-08); purge with tombstones                                    |
| Maturity             | Shipping early beta                                                              | Pre-implementation                                                                                                                        |

## Lessons for Cairn

- Running repositories no longer separates Buzz from Cairn. Where the
  repository runs, and for whom, still does. Buzz's relay is a forge
  the community pushes to. A Cairn node repository is a bare
  repository on disk for the person's own lanes: no server, no inbound
  push, no traffic of its own. Row 28 of the register keeps it that
  way.
- A lane over several repositories is something Buzz's branch-as-room
  cannot express: one intent, a branch in each repository, landing
  tracked per repository. It belongs in the pitch once LANE-25 holds.
- Buzz's silent audit write is the failure I6 forbids. LANE-23 counts
  and audits every failed operation on a node repository; the lane
  view should show those failures as well as count them.
- Buzz keeps the merge decision in the room; Cairn defers signed
  approvals to P2 and lands on the forge. Until LANE-07 ships, Buzz
  leads on the gate.
- Buzz's operating cost (Postgres, Redis, object storage) is the
  argument for keeping Cairn's repositories server-free: git over the
  local file system, started only by the run component.
- Open: how a lane's branches reach the person's own clone (OQ-33),
  and whether the run component drives the git program or a library,
  and copies the clone's objects or borrows them (OQ-34).
