# Cairn's pitch and direction

The working pitch, the stakeholder's decisions behind it, and how it
got here. The SRS change this plan ends in turns it into the contract;
until then it is direction, not requirement.

## Draft v10: the developer pitch (4 October 2026)

Written for developers: their problem in their words first, then what
Cairn changes. Items marked "next" are P2 in the SRS.

### Cairn: from intent to outcome

State what you want. Judge what you got. Correct course. Then run more
agents than you could before.

**You know this.** Five agents on five things is already a lot: Claude
Code in five worktrees, some spawning subagents. One waits on a
permission prompt in a tab you cannot find. Another lost your rule to
compaction ("never push to main"). A third reports "all tests pass",
and to judge it you open the app, the diff and the logs in separate
windows, still unsure how it got there or whether it did what you
asked. Two of them edited the same file. When one is wrong, you start
a new session and explain it all again. You would run more agents, if
you could keep up with five.

**With Cairn.** Install the plugin, keep your harness, and open Cairn
in your browser. One screen holds the work:

- **Left, your intents.** Each with its agents and what needs you,
  ranked: a prompt to answer, an outcome ready to judge, a failure, two
  agents touching the same file.
- **Middle, the chat.** The live conversation with the agent you
  picked, every edit and command in place. Answer its prompt or reply
  right there.
- **Right, the outcome, live.** The running app from that worktree's
  dev server, the diff, the files it produced, the test runs, changing
  as the agent works.
- **On top, the intent.** The goal and its criteria, each with its
  evidence: a test run the harness recorded, or only the agent's word.

1. **Say it once.** Write the intent in Cairn, or type `/intent` in
   your harness: the goal and what done means. Pin your rules beside
   it. Both come back word for word after every compaction and go with
   every task an agent hands to a subagent.
2. **Judge where you see it.** Click through the app, read the diff,
   check each criterion against its evidence. Mark it met or needs
   changes; Cairn never decides for you.
3. **Correct course in place.** Type the correction beside the
   criterion, and it reaches the agents on that intent; watch the
   outcome change. Or sharpen the intent, or retry from an earlier
   checkpoint in a fresh worktree, with the first attempt kept beside
   it.
4. **Catch up.** Back after lunch or the weekend? The intents show
   what changed, what failed and what waits, each line linked to the
   record. If capture failed or part of the record is missing, Cairn
   says so.

The plugin shows every settings change before making it, removes
cleanly, and never slows or blocks your harness. Every agent's work is
recorded on your machine, secrets redacted. Prefer the terminal?
Everything in Cairn works there too.

**Built to grow.** When five agents feel easy, run more: hand an agent
a budget and let it delegate, and keep judging by intent rather than
by agent. Next, bring your team in: share an intent peer to peer, with
no server; teammates see the same screen, judge the same outcome under
their own names, and their corrections reach your agents only when you
pass them on.

**Yours alone.** Until you turn sharing on, Cairn connects to nothing.
Its screen is served only on localhost, for your browser. Your agent
gets its history only when it asks, marked untrusted, never as a
command.

## Pitch v9 (3 October 2026, after three persona reviews)

> Cairn: know exactly what your agents did, and work with them live.
>
> Agents now work around the clock, in parallel, across machines. Cairn
> keeps the full, hash-chained record of each lane of work (every
> message, edit, tool run and result) on your own machines, with
> secrets redacted before anything is written, and tells you when any
> of it is missing or changed. Your standing rules are
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
> when the network splits. Also next: hand a maintainer a lane as a
> reviewed, signed bundle, a plain file with no account or peering.
>
> Others can then post to your lane; only you decide what becomes an
> instruction to your agents. Endorsing a post sends exactly what you
> saw, signed as yours. Otherwise their words, like tool output and web
> pages, reach your agents only when the agents ask, marked untrusted.

What the requirement traces changed in the pitch, and why:

| Pitch change                                                                                      | Reason from the SRS                                                                                                                       |
| ------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| Added pinned rules restored word for word, and exact recall                                       | I3 and RCL-03 are the SRS's core value and its strongest measured result; the pitch had dropped them                                      |
| Added redaction before anything is written                                                        | SEC-08 redacts before storage; the security officer and maintainer personas need it                                                       |
| "Reach agents only when they ask" instead of "reach them as untrusted data"                       | I2 has two halves, the label and the asking; the pitch kept only the label                                                                |
| "Others can post; only you decide what becomes an instruction" instead of "only you can instruct" | PRV-04 makes automation the default, where even the owner is untrusted; a principal endorses a post to their own agents as a recorded act |
| Tool output and web pages named as untrusted too                                                  | I2 covers all content outside the trusted boundary, not only other people                                                                 |
| "Nothing leaves your machine except what your agent recalls"                                      | I4 itself lets recalled content travel to the model provider                                                                              |
| "What verified it: the agent's claim or its own run; re-runs and CI next"                         | No requirement covered verification; LANE-05 gives each result one evidence class                                                         |
| "Tells you when any of it is missing"; re-runs and CI marked next                                 | Persona reviews: VIEW-08 and REC-19 report gaps; witness runs and CI attestation are P2 (LANE-05, SEC-28)                                 |
| "Endorsing a post sends exactly what you saw"                                                     | D2 and OWN-08: a principal's endorsement is the one path from another person's words to an agent                                          |
| Peer to peer is "next"                                                                            | Peering is P2 work after the standalone release; NG4 excludes it from v1                                                                  |

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
   port. Standalone means Cairn sends nothing off the machine.
10. Network boundaries are defined explicitly: process, machine, peer
    and public. See
    [plan 2610022338](../2610022338_cairn-network-side/plan.md).
11. Judging happens beside the chat: the lane view shows the outcome
    live (the running app from the dev server, the diff, the files the
    harness produced, the test runs). The outcome pane is table stakes:
    all of it, the running-app preview included, ships in the first
    release (4 October 2026). What sets Cairn apart is judging against
    the intent and correcting in place. This informs OQ-30.

## Direction compared with Zed Delta

Zed Delta is the closest product: public beta since 16 September 2026,
"Replacing pull requests". Sources are in the
[review](../../research/notes/live-pr-pitch-review/competitors.md). T3
Code competes with the standalone lane view today: a local GUI that
drives several harnesses, with diffs, checkpoints and in-app review, but
one agent per thread and one person per environment. It is read from its
source in the [T3 Code note](../../research/notes/t3code/t3code.md). Amp's
orbs run agents on its remote machines and add agent-to-agent messaging
and multiplayer threads, all on Amp's service, with no documented trust
boundary ([Amp orbs note](../../research/notes/amp-orbs/amp-orbs.md)).

|                  | Zed Delta                                   | Cairn (vision; not built)                                                      | T3 Code                                                                 | Amp orbs                                                        |
| ---------------- | ------------------------------------------- | ------------------------------------------------------------------------------ | ----------------------------------------------------------------------- | --------------------------------------------------------------- |
| Unit             | Thread: conversation and edits side by side | Lane: conversation, edits, tool runs and results in one record; approvals next | Thread per agent session; worktrees and checkpoints                     | Thread per agent; agents message each other across threads      |
| Topology         | Central: Cloudflare Durable Objects backend | Standalone first; peer to peer, no central service                             | Local server; remote access via Tailscale, SSH or a T3 relay            | Central: Amp's servers hold threads and run orbs                |
| Partitions       | Not documented                              | Each side keeps working; logs merge on reconnect                               | Not applicable: one server per environment                              | Not documented                                                  |
| Live co-editing  | CRDT worktrees                              | Planned; agents in separate worktrees need none                                | None; one person, several devices; one agent per thread                 | Workspace members join a thread and instruct the agent directly |
| Editor           | Zed's own app, agents over ACP              | Any editor; Claude Code first                                                  | Its own web, desktop and mobile GUI; Codex app-server, Claude Agent SDK | Its own terminal, web and mobile apps; agents in remote orbs    |
| Record integrity | No signing documented                       | Hash-chained log per writer, sealed by the writer's key (P1)                   | Event store in SQLite; no chain or signatures                           | Server-side threads; no signing documented                      |
| Co-author trust  | None documented                             | Each agent answers to one person; others reach it by endorsement (I2)          | None; history recalled raw, no untrusted envelope                       | None documented; teammates' messages read as instructions       |
| Merge gate       | Land step; PRs off, pushes to main          | Next: signed approvals and required checks on the lane                         | In-app PR review and merge, sent to the forge                           | Changes view and diffs; not documented further                  |
| Data control     | Cloudflare-managed keys, deletion by email  | Removal on your own nodes, no hash left to confirm it; peers asked too         | Local SQLite, soft deletes, no redaction; telemetry on by default       | Amp's service; workspace threads shared by default              |
| Maturity         | Shipping                                    | Pre-implementation                                                             | Shipping, MIT, fast-moving                                              | Shipping                                                        |

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
8. After the persona reviews ([persona-review](persona-review/README.md)):
   say what is missing, mark re-runs, CI and the gate as next, and name
   endorsement as the way another person's words reach an agent.
9. For developers: open with the day they already have (tabs,
   compaction, the unexplained diff, the review without its story),
   then what Cairn changes; add reviewing the outcome, delegation and
   lane sharing; half the words of v9. The stakeholder named the
   process Cairn must excel at: intent to outcome. The pitch now
   follows that loop: state the intent, watch, judge the outcome,
   correct course, with several judges on top. A person judges; Cairn
   never does.
10. The developer's experience step by step, in the harness and the
    lane view; "harness" where the harness is meant; "git keeps the
    code" dropped.
11. The lane view leads: chat and live outcome side by side (the
    running app, the diff, the files, the test runs), and the person
    judges and corrects right there.
12. After the blind review ([round 8](persona-review/round-8/README.md)):
    the pitch is for the developer with a lot of agents. Fleet framing,
    pinned rules beside the intent, catch-up, the install contract, the
    terminal as an equal surface, overlap warnings, a clear retry, and a
    plain statement of what Cairn connects to.
13. The tooling must let one developer run a hundred agents: the view
    shows only what needs the person, ranked, and delegation runs
    within a granted budget.
14. A hundred agents work on about ten intents. The intent, not the
    agent or the lane, is what the person sees, judges and corrects; an
    intent may hold several lanes and parallel attempts.
15. The target is a team: four people, ten intents, a hundred agents.
    Teammates judge, correct and collide inside the main story, not in
    a footnote; sharing still arrives after the single-developer
    release.
16. Start from today: nobody runs a hundred agents, because the
    tooling does not exist; five agents on five intents is already a
    lot. The pitch opens there and promises growth: delegation, judging
    by intent, then the team. The hundred agents and the team of four
    stay the direction, not the opening.
17. Cairn's screen leads: intents on the left, the chat in the middle,
    the live outcome on the right, the intent and its criteria on top.
    The terminal is the alternative, not the frame.

## Open before the SRS change

The reconciled [proposal](proposal.md) answers the items once listed
here: I4 by boundary and SEC-01 (§3), NG5 (§3.3), endorsement (D2,
OWN-08) and keyed commitments (REC-17). What stays open is in its §12.
