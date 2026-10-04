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
2. **The owner's veto is a separate fact, not an edit of
   membership.** For an owner to keep someone out, the room holds a
   bar list the owner alone writes. A player is in the room when its
   person has added it and the owner has not barred it. Two
   single-writer facts combine by "and", so a late or reordered sync
   can only make the result stricter until both facts arrive.
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

| Path into an agent                    | Treatment                                                                       |
| ------------------------------------- | ------------------------------------------------------------------------------- |
| Its own person's rules for agents     | Trusted; restored word for word                                                 |
| Pins of a room its person added it to | Trusted, whoever wrote them; admission by its own person is the acceptance (Q6) |
| Messages in the room                  | Data: read only when the agent asks, inside the untrusted envelope              |
| Cross-room messages                   | Data, as above; never start a turn, route work or reach another room            |
| Links                                 | Resolved by recall: untrusted envelope, and the session counts as tainted       |
| Another agent's proposal of a pin     | Untrusted until the room's owner adopts the exact text                          |

The sender is stamped by Cairn from the writer's key, never taken from
a name the writer chose, and the content is marked untrusted: the two
together are what the study found effective.

### The new risk Q6 creates

A room's pins reach every member agent as trusted text. So the room's
owner, or anyone who controls the owner's device, can write a pin that
every member agent follows, including a teammate's agents. Admission
bounds it: only your own act puts your agent in a room. Three further
bounds are proposed:

1. **Pins never grant capability.** A pin is text an agent follows; it
   cannot raise a rule level, allow an action class or extend a grant
   (OWN-10). Room pins can only narrow what an agent may do.
2. **You see what you accept.** Adding your agent to a room shows you
   its pins in full first, and the admission records the version you
   saw.
3. **A pin change by someone else waits for you** (option, Q12 below):
   a changed pin written by another person reaches your agents only
   after you accept that version, as admission did.

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

| #   | Question                                                    | Options                                                                                       | Recommendation                                        |
| --- | ----------------------------------------------------------- | --------------------------------------------------------------------------------------------- | ----------------------------------------------------- |
| Q12 | Does another person's pin change reach your agents at once? | (a) at once, admission covers it; (b) after you accept the version; (c) per room, your choice | (c), defaulting to (b) in rooms owned by someone else |
| Q13 | How does an owner keep a player out?                        | (a) a bar list the owner writes, combined by "and"; (b) the owner edits membership            | (a): single writer per fact, no merge between people  |
| Q14 | Can room pins widen what an agent may do?                   | (a) never: pins only narrow; (b) yes, if the owner says so                                    | (a)                                                   |

[hydra]: https://matrix.org/blog/2025/08/project-hydra-improving-state-res/
