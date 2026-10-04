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

| Concept   | Kind    | What it is, and nothing more                                             | Owner      |
| --------- | ------- | ------------------------------------------------------------------------ | ---------- |
| Player    | compute | A person, or an agent session in a harness                               | itself     |
| Workspace | data    | Where code changes: a branch and its worktrees (the SRS lane, narrowed)  | a person   |
| Intent    | data    | A goal and the criteria for done, versioned                              | a person   |
| Room      | data    | A conversation: an ordered stream of messages                            | a person   |
| Rule set  | data    | A person's rules (pins), versioned                                       | a person   |
| Grant     | data    | Authority to delegate: which agents, which targets, how much, until when | a person   |
| Link      | data    | An address plus a range: text, lines or an image region                  | its writer |

Nothing in this table refers to another row.

## Bindings compose them

A binding is a recorded, versioned fact that relates two concepts. It
is data, written by a person (or, where marked, derived from
structural events). Bindings are the only way concepts affect each
other.

| Binding      | Relates            | Means                                                                                         | Written by                     |
| ------------ | ------------------ | --------------------------------------------------------------------------------------------- | ------------------------------ |
| works in     | player → workspace | This agent session edits this workspace                                                       | structural (session start)     |
| member of    | player → room      | This player reads and posts in this room                                                      | a person, or structural        |
| pursues      | workspace → intent | Work here is judged against this intent                                                       | a person                       |
| discusses    | room → intent      | This room's conversation is about this intent                                                 | a person                       |
| applies to   | rule set → scope   | These rules apply wherever this scope reaches (project, workspace, intent, room or one agent) | a person                       |
| delegated by | player → player    | This agent works for that one, under a grant                                                  | structural (delegation record) |
| linked from  | link → message     | This message points at that range                                                             | the message's writer           |

## Three composition rules

Everything a player gets is derived from the bindings, deterministically
(I10). No concept carries its own delivery logic.

1. **Rules reach an agent** when the rule set's owner is the agent's
   own person, and the rule set applies to a scope the agent reaches:
   its project, a workspace it works in, an intent that workspace or a
   room it is in pursues or discusses, a room it is a member of, or the
   agent itself. Reach is a closed, finite walk over bindings. This
   keeps I2 (only your rules reach your agents) and I8.
2. **The intent reaches an agent** by the same walk, as its own
   person's text or by id and version when the intent belongs to
   someone else.
3. **Messages reach a player** only through membership, and only when
   it reads them: data, never instructions. A cross-room message is a
   message whose writer is a member of the sending room, addressed to
   another room.

Restore, join, leave and change notices are then not concepts of their
own: they are the moments the derived result of rules 1 and 2 changes
for an agent, delivered at its next hook point.

## Use cases as configuration

| Use case                                   | Configuration                                                                                                                                       |
| ------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| One agent, one task (today's Cairn)        | One workspace; the agent works in it; your rule set applies to the project. No intent or room needed.                                               |
| One agent with a stated goal               | Add an intent; the workspace pursues it.                                                                                                            |
| Five agents, parallel attempts at one goal | Five workspaces, each pursuing one intent; one room discussing it, with the five agents as members.                                                 |
| One agent on two goals                     | The agent is a member of two rooms, each discussing its own intent; rule sets apply per intent.                                                     |
| A chat with no goal                        | A room and its members; no intent, no workspace.                                                                                                    |
| A goal with no chat (batch)                | An intent and the workspaces pursuing it; no room.                                                                                                  |
| Orchestrator with subagents                | Subagents delegated by the orchestrator; members of its room only if you add them, otherwise they reach rules through the orchestrator's scopes.    |
| Rooms talking                              | A message in room A addressed to room B.                                                                                                            |
| Handing work to another room               | A grant naming room B's agents as targets; the task arrives as data marked as from the delegating agent.                                            |
| A teammate joins (next)                    | The teammate becomes a member of the room; their own rule sets reach only their agents; your intent reaches their agents by id until they adopt it. |
| A reviewer                                 | A member of the room, with no workspace and no agents.                                                                                              |
| A rule only for one agent                  | A rule set that applies to that agent.                                                                                                              |

## What this changes

- **Q1 is answered by the split.** A room is neither a lane nor a group
  of lanes. It is a conversation bound to an intent, and workspaces are
  bound to the same intent independently.
- **The SRS lane narrows to the workspace.** Its conversation moves to
  rooms. Its intent and the scope of its pins move to bindings. LANE
  requirements about results and evidence stay with the workspace.
- **Pin scope becomes "applies to"** with a fixed set of scope kinds,
  replacing PIN-10's lane rules.
- **Q4 (several rooms) and Q5 (how a plain session joins) become
  configuration.** A session may be a member of any number of rooms.
  A plain session reaches rules through the workspace it works in,
  with no membership needed.
- **Q7 (private subagents)** reduces to rule 1: a subagent reaches its
  delegating agent's scopes, delivered through its own start hook.

## Open points in the split

- **Scope kinds.** Whether "applies to" needs more kinds than project,
  workspace, intent, room and agent.
- **Conflicts across scopes.** Two rule sets of the same person
  reaching one agent from two scopes: both apply, ordered by scope kind
  and priority, never merged (I3). Contradictions are shown, not
  resolved (CMP-09).
- **Who may write bindings in a shared room.** Proposed: only the
  owner of the bound concept, so a teammate cannot bind your intent or
  your rules.
