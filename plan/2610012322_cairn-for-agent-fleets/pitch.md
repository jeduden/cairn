# Cairn's pitch and direction

The working pitch, the stakeholder's decisions behind it, and how it
got here. The SRS change this plan ends in turns it into the contract;
until then it is direction, not requirement.

## The pitch (3 October 2026, after the persona reviews)

> Cairn: know exactly what your agents did, and work with them live.
>
> Agents now work around the clock, in parallel, across machines. Cairn
> keeps the full, tamper-evident record of each lane of work (every
> message, edit, tool run and result) on your own machines, with
> secrets redacted before anything is written, and tells you when any
> of it is missing. Your standing rules are
> pinned: they come back word for word after every compaction, and
> your agents recall exactly what they did whenever they ask.
>
> One view shows each agent as it works, what it produced, and what
> verified it: the agent's own claim or a run on its own machine; next,
> a reviewer's re-run or CI for that exact commit.
>
> It runs standalone: nothing leaves your machine except what your
> agent recalls into its own model call. Next, turn on peer to peer to
> work a lane live with others, with no central service and no break
> when the network splits.
>
> Others can then post to your lane; only you decide what becomes an
> instruction to your agents. Endorsing a post sends exactly what you
> saw, signed as yours. Otherwise their words, like tool output and web
> pages, reach your agents only when the agents ask, marked untrusted.

What the requirement traces changed in the pitch, and why:

| Pitch change                                                                                      | Reason from the SRS                                                                                                                         |
| ------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| Added pinned rules restored word for word, and exact recall                                       | I3 and RCL-03 are the SRS's core value and its strongest measured result; the pitch had dropped them                                        |
| Added redaction before anything is written                                                        | SEC-08 redacts before storage; the security officer and maintainer personas need it                                                         |
| "Reach agents only when they ask" instead of "reach them as untrusted data"                       | I2 has two halves, the label and the asking; the pitch kept only the label                                                                  |
| "Others can post; only you decide what becomes an instruction" instead of "only you can instruct" | PRV-04 makes automation the default, where even the owner is untrusted; the owner adopts a post as an instruction as its own recorded event |
| Tool output and web pages named as untrusted too                                                  | I2 covers all content outside the trusted boundary, not only other people                                                                   |
| "Nothing leaves your machine except what your agent recalls"                                      | I4 itself lets recalled content travel to the model provider                                                                                |
| "What verified it: claim, local run or CI"                                                        | No requirement covered verification; LANE-04 adds a level per result                                                                        |
| "Tells you when any of it is missing"; re-runs and CI marked next                                 | Persona reviews: VIEW-08 and REC-19 report gaps; witness runs and CI attestation are P2 (LANE-05, SEC-28)                                   |
| "Endorsing a post sends exactly what you saw"                                                     | D2 and OWN-08: a principal's endorsement is the one path from another person's words to an agent                                            |
| Peer to peer is "next"                                                                            | Peering is P2 work after the standalone release; NG4 excludes it from v1                                                                    |

The lane is the unit: a branch, its worktrees, its agents and humans,
and their conversations and results. That is a pull request in all but
its interface. Git keeps the code; Cairn keeps the lane.

## The stakeholder's decisions

1. Keep the name Cairn.
2. The lane, a live pull request, is the unit Cairn holds.
3. No dependence on a central service. Self-hosting is normal, as it
   is for git.
4. Standalone works with no network at all. Peer to peer extends it.
5. Multiplayer works without a central server, through partitions;
   CRDTs and other conflict resolution handle the merge.
6. The user experience comes first; making sync seamless is the second
   step.
7. Security must be good enough, measured against Zed Delta.
8. There is a UI. One view shows an agent's harness live, its work
   results and the other harnesses. Harnesses also connect directly;
   the UI is one more client, never a required hop.
9. The standalone UI is a web page in the browser on a loopback-only
   port. Standalone means nothing leaves the machine.
10. Network boundaries are defined explicitly: process, machine, peer
    and public. See
    [plan 2610022338](../2610022338_cairn-network-side/plan.md).

## Direction compared with Zed Delta

Zed Delta is the closest product: public beta since 16 September 2026,
"Replacing pull requests". Sources are in the
[review](../../research/notes/live-pr-pitch-review/competitors.md).

|                  | Zed Delta                                   | Cairn (vision; not built)                                                      |
| ---------------- | ------------------------------------------- | ------------------------------------------------------------------------------ |
| Unit             | Thread: conversation and edits side by side | Lane: conversation, edits, tool runs and results in one record; approvals next |
| Topology         | Central: Cloudflare Durable Objects backend | Standalone first; peer to peer, no central service                             |
| Partitions       | Not documented                              | Each side keeps working; logs merge on reconnect                               |
| Live co-editing  | CRDT worktrees                              | Planned; agents in separate worktrees need none                                |
| Editor           | Zed's own app, agents over ACP              | Any editor; Claude Code first                                                  |
| Record integrity | No signing documented                       | Hash-chained log per writer, sealed by the writer's key (P1)                   |
| Co-author trust  | None documented                             | Each agent answers to one person; others reach it by endorsement (I2)          |
| Merge gate       | Land step; PRs off, pushes to main          | Next: signed approvals and required checks on the lane                         |
| Data control     | Cloudflare-managed keys, deletion by email  | Removal on your own nodes, no hash left to confirm it; peers asked too         |
| Maturity         | Shipping                                    | Pre-implementation                                                             |

## Lessons from OpenAI dots

From [the dots notes](../../research/notes/openai-agent-ui/dots.md):
the question of the moment is "what did my agent do?", which Cairn's
record answers; a completed run is not a verified one, so results show
what checked them; rule levels per action shape the owner's control.

## How the pitch changed

1. A lossless, secure memory for one agent.
2. Session history next to git, shared across a fleet.
3. The live pull request: the lane as one record.
4. After a blind adversarial review
   ([notes](../../research/notes/live-pr-pitch-review/)): lead with
   security, since Delta owns the live-PR headline; add the process a
   pull request runs; stop claiming that code and talk "never drift".
5. Standalone first, peer to peer second, a UI over harnesses.
6. After OpenAI dots: lead with knowing what your agents did.
7. After the requirement traces
   ([forward](trace-forward.md), [backward](trace-backward.md)): restore
   pinning, recall and redaction; keep I2's asking half; scope owner
   trust and "nothing leaves the machine" to what the SRS can hold.
8. After two persona reviews ([persona-review](persona-review/README.md)):
   say what is missing, mark re-runs, CI and the gate as next, and name
   endorsement as the way another person's words reach an agent.

## Open before the SRS change

The reconciled [proposal](proposal.md) answers the items once listed
here: I4 by boundary and SEC-01 (§3), NG5 (§3.3), endorsement (D2,
OWN-08) and keyed commitments (REC-17). What stays open is in its §12.
