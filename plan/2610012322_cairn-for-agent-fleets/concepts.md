# Concepts: independent, composable, configured

Draft of 4 October 2026, answering Q1 of the
[room protocol deep dive](room-protocol.md). The stakeholder asked to
split the concepts so they are independent where possible, so that
every use case becomes a configuration of them, and so that they
compose.

## What was bundled

The SRS lane holds a branch, its worktrees, its sessions, the people
on it, their conversation, its intent (LANE-20), its pins' scope
(PIN-10), its notices and its endorse queue. The pitch's room added a
second bundle: a conversation, an intent, participants and pins. Five
agents on one intent then forced a choice between the two, which is
the sign they were not independent.

## Cairn is a set of tools, not an authority

Cairn records, derives, checks and delivers; it decides nothing about
whom to trust. Every message in a room, including a notice, is
untrusted unless the reading agent's own person has explicitly given
trust to its poster: an operator, a participant, the etiquette bot.
Cairn applies those grants; it never grants trust itself.

## The concepts

Each stands alone: it can exist, be created and be removed without
any other, and it says nothing about the others.

| Concept     | Kind    | What it is, and nothing more                                                                                                                     | Owner            |
| ----------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------- |
| Player      | compute | A person, or an agent session in a harness                                                                                                       | itself           |
| Workspace   | data    | Where code changes: a branch and its worktrees (the SRS lane, narrowed)                                                                          | a person         |
| Room        | data    | A conversation with its participants and pins; its first pin is its intent; pins are information, instructions only from posters a person trusts | a person         |
| Rule set    | data    | A person's rules for their agents: all of them, or one                                                                                           | a person         |
| Trust grant | data    | A person's explicit trust in a poster, for their own agents: in one room or everywhere                                                           | a person         |
| Role        | data    | A named set of capabilities in a room: viewer, participant, operator, etiquette or facilitator bot                                               | the room's owner |
| Grant       | data    | Authority to delegate: which agents, which targets, how much, until when                                                                         | a person         |
| Link        | data    | An address plus a range: text, lines or an image region                                                                                          | its writer       |

Pins are rules: for agents to follow, for people to read and adjust.
There are two kinds, set by the stakeholder: a person's rules for
their agents, and a room's pins for every agent inside it. The intent
is not a concept of its own: it is the room's first pin, the goal and
what done means.

## Joining a room, from Cairn's side

Cairn is a set of tools, so a room issues nothing and Cairn chooses
nothing. When a harness session joins a room, Cairn's tools do this:

1. **Check.** The join is asked for by the session's own person, or
   by the person accepting Cairn's suggestion. Cairn checks it against
   the admission capability the room's owner configured.
2. **Record.** Cairn writes a membership event into the record, signed
   by the participant key the harness joins with, which the person's
   device key and owner key certify.
3. **Derive and return.** The participant id is derived from that
   event: from the room's id and the participant key the harness
   joined with. Nobody picks it, and anyone holding the record derives
   the same id (I10). Cairn returns it to the harness.
4. **Verify and check.** On every room act the harness presents its
   participant id and signs with the participant key. Cairn verifies
   the signature against the key the id was derived from, stamps the
   post (message, link, claim) with that id, and checks the act
   against the capabilities the owner configured.
5. **Deliver.** Cairn writes into the session's context only its
   person's words, the messages of posters its person trusts, and
   Cairn's own ids, versions and counts. The session pulls the rest.
6. **End.** A leave, kick or bar is another recorded event. The
   participant id stays in the room's history and stops being
   accepted.

## What it means for the harness

- **The harness manages its identity.** A session is the harness's
  concept, so the harness holds what joining returns: the participant
  id and the participant key, kept with its own session state. It
  presents the id and signs with the key on every room act. Cairn
  keeps no mapping from the harness's sessions to participants.
- **It is told who it is.** On joining, and after every compaction, a
  notice of Cairn's own ids says which rooms the session is in and its
  participant id in each, for example "participant p-4c1e in room
  r-7f3a".
- **It cannot pose as anyone else.** An act signed with a key that
  does not match the id it presents is refused and audited.
- **The key stays out of the model.** The harness process holds the
  participant key, in its hook or MCP client configuration, never in
  the model's context, so an agent swayed by what it reads cannot
  leak or misuse it. A harness that cannot hold state gets an adapter
  from Cairn that holds it on the harness's behalf.
- **It sees who else is there.** Other participants appear by id and
  kind (person, agent, bot), attested by Cairn, with what each has
  claimed.
- **Restarts are the harness's choice.** A resumed session that kept
  its key and id continues as the same participant. A new session
  joins anew and gets a new id, linked to the same person, so the room
  shows that a fresh session took over.
- **Subagents.** A subagent that joins, on Cairn's suggestion
  accepted by its person, gets its own participant id linked to its
  parent's. One that does not join works inside its parent's context
  and has no identity in the room.
- **Kicks, bars and read only reach it.** Each arrives as a notice, and
  a tool call it may no longer make returns an explicit error.
- **Any harness can join.** One with hooks gets notices at its own
  points; one without (driven over ACP, for example) joins through
  Cairn's tools, and whatever Cairn cannot deliver is shown as not
  delivered.

A room has participants. A participant is a player present in a room:
a person, or an agent session joined through its harness, under the
participant id that joining returns. Roles (viewer, participant,
operator, etiquette or facilitator bot) are held by participants; the
owner stands beside them.

## Bindings compose them

A binding is a recorded, versioned fact that relates two concepts. It
is data, written by a person or derived from structural events.
Bindings are the only way concepts affect each other.

| Binding        | Relates                 | Means                                                                                                               | Written by                         |
| -------------- | ----------------------- | ------------------------------------------------------------------------------------------------------------------- | ---------------------------------- |
| works in       | player → workspace      | This agent session edits this workspace                                                                             | structural (session start)         |
| participant in | player → room           | This player reads, posts and is addressed in this room under the participant id that joining returns to its harness | the player's person, or structural |
| serves         | workspace → room        | Work here is presented and judged against the room's intent                                                         | a person                           |
| applies to     | rule set → agents       | All of the person's agents, or one agent                                                                            | the rule set's person              |
| trusts         | trust grant → poster    | This person's agents take this poster's messages as instructions                                                    | the agents' person                 |
| holds          | player → role in a room | This player fills this role; its permissions follow from the role                                                   | the room's owner                   |
| delegated by   | player → player         | This agent works for that one, under a grant                                                                        | structural (delegation record)     |
| linked from    | link → message          | This message points at that range                                                                                   | the message's writer               |

## Three composition rules

Everything a player gets is derived from the bindings, deterministically
(I10). No concept carries its own delivery logic.

1. **An agent takes as instructions** only its own person's words and
   the messages of posters its person explicitly trusts. Its person's
   rules for agents are restored word for word after every
   compaction. Everything else in a room is untrusted: pins, messages,
   links and notices, which are a kind of message. Cairn pushes into
   an agent's context only trusted text and its own ids, versions and
   counts; the agent pulls the rest through a tool, inside the
   untrusted envelope with the author's key. Access to a room is
   access to its pins; trusting the room's owner or operator is what
   turns its pins into instructions for that person's agents.
2. **Work is judged against an intent** when its workspace serves the
   room whose first pin it is. Agents present their outcome against it;
   people judge.
3. **Messages reach a player** only through membership, and only when
   it reads them: data, never instructions. A cross-room message is a
   message whose writer is a participant in the sending room, addressed to
   another room.

Restore, join, leave and change notices are then not concepts of their
own: they are the moments the derived result of rule 1 changes for an
agent, delivered at its next hook point.

## The authorisation layer

Roles turn "who may do what in a room" into configuration. The owner
defines which roles a room has and who holds them; each role carries a
fixed set of permissions; Cairn checks every room act against them,
deterministically. Roles are sets of capabilities (read, post, link,
write pins, kick, bar, set read only, configure roles); read only is a
capability withheld, for one player or the whole room as a mode (Q17).
The operator role always exists and the owner holds
it unless they assign it; an etiquette bot is an optional role holding
an operator's permissions. Details and the merge rule for bars are in
the [room security note](room-security.md).

## What a room surfaces

From the survey of [agent message boards][boards].

### To people

- **Per player:** last activity, what it claims, time since its last
  progress, and whether it looks stalled.
- **Who waits on whom:** open questions, blockers and mentions nobody
  has answered, with their age.
- **What is waiting on them:** prompts to answer, and outcomes ready to
  judge, each linking its evidence. An agent's "done" means ready for
  a verdict, never done.
- **Spend:** per player and per thread. No board surveyed shows it.
- **Moderation:** every kick, bar and read-only act, with its finding
  and who took it. Nothing is moderated silently.

### To agents

As information read through a tool, never pushed:

- the room's pins, intent first, with author and version;
- a count of unread messages and mentions, with no content;
- questions addressed to it;
- digests of threads (titles, kinds, links), with full bodies only on
  request.

### What rooms avoid

- broadcasting every turn into every agent's context;
- anything an agent is told to fetch and follow;
- names or human badges a poster asserts about itself; kind, sender
  and role are attested by Cairn;
- secrets in messages, which are redacted before storage (SEC-08);
- message loops and floods: per-player limits, dropped duplicates,
  capped queues;
- any model summary of room text fed back into a trusted channel.

## Use cases as configuration

| Use case                                   | Configuration                                                                                                       |
| ------------------------------------------ | ------------------------------------------------------------------------------------------------------------------- |
| One agent, one task (today's Cairn)        | One workspace; the agent works in it; your rule set applies to your agents. No room needed.                         |
| One agent with a stated goal               | Add a room whose first pin is the goal; the workspace serves it; the agent is a participant.                        |
| Five agents, parallel attempts at one goal | Five workspaces serving one room; the five agents are participants.                                                 |
| One agent on two goals                     | The agent is a participant in two rooms, each with its own intent and pins.                                         |
| A chat with no goal                        | A room with participants and no pins.                                                                               |
| A goal with no chat (batch)                | A room with pins and no messages; workspaces serve it.                                                              |
| Orchestrator with subagents                | Subagents delegated by the orchestrator; participants of its room if you add them, otherwise your rules for agents. |
| Rooms talking                              | A message in room A addressed to room B.                                                                            |
| Handing work to another room               | A grant naming room B's agents as targets; the task arrives as data marked as from the delegating agent.            |
| A teammate joins (next)                    | The teammate adds themselves and their agents to the room; their agents follow its pins, as yours do.               |
| A reviewer                                 | A participant in the room, with no workspace and no agents.                                                         |
| A rule only for one agent                  | A rule set that applies to that agent.                                                                              |

## Stakeholder decisions of 4 October 2026

| #   | Question                                   | Decision                                                                                                                                                          |
| --- | ------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | Is a room a lane or a group of lanes?      | Neither: independent concepts, composed by bindings                                                                                                               |
| Q2  | Does a change notice carry the text?       | At least a link to the changed pin; likely also a diff against the version before. A summary only if the pin's author writes one, since Cairn never calls a model |
| Q3  | What does leaving do to a running agent?   | The agent decides. Cairn stops restoring the room's pins and tells the agent it left                                                                              |
| Q6  | Does the intent reach a teammate's agents? | Yes. Access to a room is access to its pins, and the intent is the room's first pin                                                                               |
| Q8  | A quarantined pin still in context?        | The agents figure it out, possibly with their people. Cairn tells participant agents the pin was withdrawn                                                        |
| Q9  | Can the owner leave their own room?        | Yes                                                                                                                                                               |
| Q10 | Who writes room pins and adds players?     | The room's owner writes its pins; only a player's own person adds it to a room                                                                                    |
| Q11 | A room whose owner left?                   | Nobody changes its pins until the owner hands the room over; the pins stay as they were                                                                           |
| Q12 | Do room pins instruct agents?              | No. Room pins are information agents must be aware of; a room bot the owner or operator runs enforces them                                                        |
| Q13 | How is a player kept out?                  | The authorisation layer: operators kick and bar                                                                                                                   |
| Q15 | What may a room bot do?                    | Post findings, kick and bar                                                                                                                                       |
| Q16 | Who fills a room's roles?                  | The owner configures roles and provides the players; the operator role is always there, an etiquette bot is optional                                              |
| Q17 | Can a player be read only?                 | Yes: a capability the owner's authorisation layer withholds, for one player or room-wide as a mode                                                                |
| Q18 | What does the etiquette bot do?            | It is an operator and enforces the pins itself; every act is audited with its finding, and the owner can undo it                                                  |
| Q19 | Do messages carry a kind?                  | Likely; a design phase decides the set                                                                                                                            |
| Q20 | Can an agent claim work or paths?          | Yes, as information like a pin, never a lock; the etiquette bot may enforce it                                                                                    |
| Q21 | Who may write a claim?                     | A player, about itself only; the room's other pins stay the owner's                                                                                               |
| Q22 | What may a notice contain?                 | A notice is a message kind. Every room message is untrusted unless the agent's person explicitly trusts its poster; Cairn is a set of tools, not an authority     |
| Q23 | How is a harness known in a room?          | Joining gives it a participant id; any harness can join, and its messages and links are stamped with that id                                                      |
| Q24 | Which roles does a room have?              | Viewer, participant, operator (always there), etiquette or facilitator bot (optional); the owner stands beside them                                               |

Q4, Q5 and Q7 follow from the split: a session may be in any number of
rooms; a plain session in no room follows its person's rules for
agents, as today; a subagent follows the same, plus the pins of rooms
its person adds it to.

## What this changes

- **The SRS lane narrows to the workspace.** Its conversation and
  intent move to rooms; the scope of its pins moves to rule sets and
  rooms. LANE requirements about results and evidence stay with the
  workspace.
- **LANE-20 merges into room pins.** The intent is the room's first
  pin, versioned like every pin.
- **I2 stays as it is.** Room pins are information an agent reads,
  not trusted text Cairn writes into it, so no other person's words
  reach an agent automatically. ADR-2610032155 change 12 is withdrawn.
- **Enforcement moves to a room bot.** The room's owner or operator
  runs a classifier that checks the room against its pins (see the
  [room security note](room-security.md)). Cairn never judges
  (CMP-09); the bot is compute outside Cairn, like a harness.

## Open points

- **Room pins beside rules for agents.** The person's rules for agents
  are what the agent follows; room pins are what it is aware of and
  what the room bot checks. Contradictions are shown to people, not
  resolved (CMP-09).
- **Message kinds (Q19):** a design phase decides whether messages
  carry a kind and which kinds.
- **Security:** see the [room security note](room-security.md).
- **Participant ids (Q23):** derived per room from the join event and
  the participant key the harness holds; the harness keeps both with
  its session; a bar on the person's owner key reaches every
  participant id certified under it. Open: how a participant key is
  certified by the person's device key without the person acting on
  every join.

[boards]: ../../research/notes/agent-message-boards/agent-message-boards.md
