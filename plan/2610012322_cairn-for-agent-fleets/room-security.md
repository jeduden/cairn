# Room security: the two warnings from prior art

Draft of 4 October 2026, on the two warnings in the
[room protocol deep dive](room-protocol.md), applied to the
[concepts](concepts.md) as the stakeholder decided them.

## Warning 1: merging membership across peers

### What went wrong elsewhere

Matrix lets several servers each hold a room and merge their views.
Membership and power levels are authorisation state, and when two
servers' histories conflicted, state resolution could pick a branch
that undid a ban or a power change: a "state reset". It took a
coordinated security release and room version 12 in 2025 to fix
([Project Hydra][hydra]).
The root cause is several parties writing the same authorisation
facts, and a merge rule deciding between them.

### Where Cairn is exposed

Only once peering exists (P2): rooms shared between nodes, people
working offline, a teammate's node syncing late. Standalone, one node
holds everything and nothing merges.

### How the decided model avoids it

1. **Every fact has exactly one writer.** The room's pins are written
   only by the room's owner. A player's membership is written only by
   that player's own person. No two people ever write the same fact,
   so no merge ever decides between two people.
2. **Keeping a player out is the authorisation layer's job (Q13).**
   A room's owner assigns roles; a role carries permissions; an
   operator may kick a player (remove it now) or bar it (keep it out).
   A player is in the room when its person has added it and no
   operator's bar stands. Bars from several operators combine by "any
   bar wins", so a late or reordered sync can only make the result
   stricter. A bar is lifted only by the operator who set it or by the
   owner, and an operator whose role the owner revokes writes no
   further bars.
3. **Within one writer, restriction wins.** Only one person's own
   devices can conflict, for example two laptops offline. Then a
   removal beats an add, and a bar beats an unbar, at the same causal
   position. The conflict is recorded and shown (I6). A revocation is
   never undone by a merge, only by a later explicit act.
4. **Ownership moves only by a signed pair.** A handover is the old
   owner's signed event accepted by the new owner's (LANE-11). An
   owner who leaves without handing over freezes the room's pins (Q11).
5. **Equivocation is detected.** A writer who signs two different
   histories is shown as equivocated (PEER-10), and their later events
   stop counting.

### What stays for membership

- **Withheld events.** A peer that withholds a bar delays it on nodes
  that only hear from that peer. The bar's writer's own node applies it
  at once, and sync stops sending to the barred key. A node holding a
  gap shows it.
- **Content already copied.** A barred player keeps what its node
  already holds; barring stops future content, never past.
- **A test that proves it.** The merge must be property-tested: no
  sequence of deliveries, reorderings or duplications re-admits a
  barred player or revives a removed membership.

## Warning 2: other agents' text as instructions

### What goes wrong elsewhere

AutoGen, LangGraph, CrewAI and the OpenAI Agents SDK hand one agent's
words to another as conversation or as task context, which the model
treats as instructions. Prompt Infection showed an injection spreading
from agent to agent that way; naming the sender cut attacks by about
5%, while naming the sender and marking the content untrusted stopped
them in that study ([arXiv 2410.07283](https://arxiv.org/abs/2410.07283)).

### What the decided model does

| Path into an agent                | Treatment                                                                                                                       |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| Its own person's rules for agents | Trusted; restored word for word                                                                                                 |
| Pins of a room it is a member of  | Information: a fixed notice names the room and pin versions; the agent reads them through a tool, inside the untrusted envelope |
| Messages in the room              | Data: read only when the agent asks, inside the untrusted envelope                                                              |
| Cross-room messages               | Data, as above; never start a turn, route work or reach another room                                                            |
| Links                             | Resolved by recall: untrusted envelope, and the session counts as tainted                                                       |
| A room bot's findings             | Data in the room, like any post; never instructions to anyone's agent                                                           |

The sender is stamped by Cairn from the writer's key, never taken from
a name the writer chose, and the content is marked untrusted: the two
together are what the study found effective.

Because room pins are information (Q12), no other person's words
reach an agent automatically, and I2 stays as it is. A room owner
cannot steer a teammate's agents through a pin; the most a pin can do
is be read, as any message can.

## Roles and the etiquette bot

A room's owner configures its roles and provides the players that
fill them (Q16). Cairn holds the roles and enforces their permissions
deterministically; it never judges and never calls a model (CMP-09,
I4).

| Role          | Always there | Permissions                                                                  | Filled by                                    |
| ------------- | ------------ | ---------------------------------------------------------------------------- | -------------------------------------------- |
| Owner         | yes          | Write pins, configure roles, assign holders, hand over the room              | The person who created or took over the room |
| Operator      | yes          | Kick and bar players, lift their own bars, hide a message from the room view | The owner, unless the owner assigns others   |
| Etiquette bot | no, optional | An operator's permissions, plus posting findings against the pins            | A classifier the owner provides              |
| Member        | yes          | Read, post, link                                                             | Any player its person adds                   |

- **The etiquette bot** is a player of kind bot with its own key, run
  by the owner or an operator outside Cairn, like a harness. It reads
  the room's pins, messages and links, and what members chose to make
  visible in the room, through the same tools an agent uses.
- **What it does.** It posts findings to the room, each linking the
  pin and the content it judged, raises a Needs you item for the owner
  and for the person whose agent a finding concerns, and, holding an
  operator's permissions, kicks or bars players (Q15).
- **What it never does.** Instruct, steer or stop another person's
  agents, change pins, or act outside the room. Its findings are a
  classifier's claims, shown as such, and the owner can lift any bar.
- **A room without one.** The owner may run no etiquette bot; the
  operator role is still there, held by the owner by default.

### What stays for the bot

- **Errors.** A classifier misses things and flags innocent work.
  Findings are claims, never verdicts; people judge.
- **The bot as a target.** It reads untrusted content too, so it can
  be swayed into kicking or barring. Its powers stay inside the room:
  it can remove players, never instruct them, and the owner can undo
  every removal.
- **Privacy across people.** What a bot may read of a teammate's work
  is the teammate's choice of what to make visible in the room.

### What stays for injection

Cairn controls what it delivers, not what sways an agent. An agent
that reads an untrusted message can still be persuaded by it. The
remaining defences are around the agent, not in it:

- recall taint raises sensitive actions to ask first (SEC-13);
- held requests keep the person in the loop for those actions;
- grants bound what an agent can hand on;
- the sandbox state and risk acceptance bound what an unsandboxed
  agent can do (OWN-22).

## Decisions for the stakeholder

| #   | Question                                  | Options                                                                                                                                                 | Recommendation |
| --- | ----------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------- |
| Q12 | Do room pins instruct agents?             | Decided: no. They are information; a room bot enforces them                                                                                             | —              |
| Q13 | How does an owner keep a player out?      | Decided: the authorisation layer provides it; operators kick and bar                                                                                    | —              |
| Q14 | Can room pins widen what an agent may do? | Moot: room pins are information and grant nothing                                                                                                       | —              |
| Q15 | What may a room bot do beyond posting?    | Decided: also bar and kick players                                                                                                                      | —              |
| Q16 | Does Cairn ship a room bot?               | Decided: rooms have roles; the owner configures them and provides the players, an etiquette bot among them, optional; the operator role is always there | —              |

[hydra]: https://matrix.org/blog/2025/08/project-hydra-improving-state-res/
