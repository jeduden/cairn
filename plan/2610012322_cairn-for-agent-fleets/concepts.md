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
second bundle: a conversation, an intent, members and pins. Five
agents on one intent then forced a choice between the two, which is
the sign they were not independent.

## The concepts

Each stands alone: it can exist, be created and be removed without
any other, and it says nothing about the others.

| Concept   | Kind    | What it is, and nothing more                                                        | Owner      |
| --------- | ------- | ----------------------------------------------------------------------------------- | ---------- |
| Player    | compute | A person, or an agent session in a harness                                          | itself     |
| Workspace | data    | Where code changes: a branch and its worktrees (the SRS lane, narrowed)             | a person   |
| Room      | data    | A conversation, and the pins every agent in it follows; its first pin is its intent | a person   |
| Rule set  | data    | A person's rules for their agents: all of them, or one                              | a person   |
| Grant     | data    | Authority to delegate: which agents, which targets, how much, until when            | a person   |
| Link      | data    | An address plus a range: text, lines or an image region                             | its writer |

Pins are rules: for agents to follow, for people to read and adjust.
There are two kinds, set by the stakeholder: a person's rules for
their agents, and a room's pins for every agent inside it. The intent
is not a concept of its own: it is the room's first pin, the goal and
what done means.

## Bindings compose them

A binding is a recorded, versioned fact that relates two concepts. It
is data, written by a person or derived from structural events.
Bindings are the only way concepts affect each other.

| Binding      | Relates            | Means                                                       | Written by                         |
| ------------ | ------------------ | ----------------------------------------------------------- | ---------------------------------- |
| works in     | player → workspace | This agent session edits this workspace                     | structural (session start)         |
| member of    | player → room      | This player reads, posts and, if an agent, follows the pins | the player's person, or structural |
| serves       | workspace → room   | Work here is presented and judged against the room's intent | a person                           |
| applies to   | rule set → agents  | All of the person's agents, or one agent                    | the rule set's person              |
| delegated by | player → player    | This agent works for that one, under a grant                | structural (delegation record)     |
| linked from  | link → message     | This message points at that range                           | the message's writer               |

## Three composition rules

Everything a player gets is derived from the bindings, deterministically
(I10). No concept carries its own delivery logic.

1. **An agent follows** its own person's rules for agents, then the
   pins of every room it is a member of, each room's intent first. Room
   pins reach every member agent, whoever wrote them: access to a room
   is access to its pins. Adding an agent to a room is its own person's
   act, so it is that person accepting the room's pins for that agent.
2. **Work is judged against an intent** when its workspace serves the
   room whose first pin it is. Agents present their outcome against it;
   people judge.
3. **Messages reach a player** only through membership, and only when
   it reads them: data, never instructions. A cross-room message is a
   message whose writer is a member of the sending room, addressed to
   another room.

Restore, join, leave and change notices are then not concepts of their
own: they are the moments the derived result of rule 1 changes for an
agent, delivered at its next hook point.

## Use cases as configuration

| Use case                                   | Configuration                                                                                                  |
| ------------------------------------------ | -------------------------------------------------------------------------------------------------------------- |
| One agent, one task (today's Cairn)        | One workspace; the agent works in it; your rule set applies to your agents. No room needed.                    |
| One agent with a stated goal               | Add a room whose first pin is the goal; the workspace serves it; the agent is a member.                        |
| Five agents, parallel attempts at one goal | Five workspaces serving one room; the five agents are members.                                                 |
| One agent on two goals                     | The agent is a member of two rooms, each with its own intent and pins.                                         |
| A chat with no goal                        | A room with members and no pins.                                                                               |
| A goal with no chat (batch)                | A room with pins and no messages; workspaces serve it.                                                         |
| Orchestrator with subagents                | Subagents delegated by the orchestrator; members of its room if you add them, otherwise your rules for agents. |
| Rooms talking                              | A message in room A addressed to room B.                                                                       |
| Handing work to another room               | A grant naming room B's agents as targets; the task arrives as data marked as from the delegating agent.       |
| A teammate joins (next)                    | The teammate adds themselves and their agents to the room; their agents follow its pins, as yours do.          |
| A reviewer                                 | A member of the room, with no workspace and no agents.                                                         |
| A rule only for one agent                  | A rule set that applies to that agent.                                                                         |

## Stakeholder decisions of 4 October 2026

| #   | Question                                   | Decision                                                                                              |
| --- | ------------------------------------------ | ----------------------------------------------------------------------------------------------------- |
| Q1  | Is a room a lane or a group of lanes?      | Neither: independent concepts, composed by bindings                                                   |
| Q2  | Does a change notice carry the text?       | Decided later                                                                                         |
| Q3  | What does leaving do to a running agent?   | The agent decides. Cairn stops restoring the room's pins and tells the agent it left                  |
| Q6  | Does the intent reach a teammate's agents? | Yes. Access to a room is access to its pins, and the intent is the room's first pin                   |
| Q8  | A quarantined pin still in context?        | The agents figure it out, possibly with their people. Cairn tells member agents the pin was withdrawn |
| Q9  | Can the owner leave their own room?        | Yes                                                                                                   |
| Q10 | Who writes room pins and adds players?     | The room's owner writes its pins; only a player's own person adds it to a room                        |
| Q11 | A room whose owner left?                   | Nobody changes its pins until the owner hands the room over; the pins stay as they were               |

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
- **I2 changes for room pins.** Today only an agent's own person's
  words reach it. With Q6, a room's pins reach every member agent,
  including pins another person wrote. The path stays bounded: the
  agent's own person admits it to the room, and that admission is the
  act that accepts the room's pins. This is an invariant change for
  the security review (ADR-2610032155).

## Open points

- **Room pins and rules for agents together.** Both apply, the
  person's rules for agents first, each by priority, never merged
  (I3). Contradictions are shown to people, not resolved (CMP-09).
- **Q2:** whether a change notice carries the changed text.
- **Security:** see the [room security note](room-security.md).
