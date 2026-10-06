---
summary: >-
  Cairn's domain model: the closed set of concepts, how they relate,
  the terms that are not Cairn concepts, and how names in code, docs
  and UI follow the model. The domain-model agent reviews against it.
---
# Domain model

Cairn speaks in a small, closed set of concepts. The requirements, the
scenarios, the code, the documentation and every screen use them, and
only them. This document is the model's one source: the closed list
of concepts, the relations between them, and the terms that are not
Cairn concepts. Every other definition, the glossary in
[§1.6](srs/01-introduction.md) included, says what this document
says.

## Concepts

| Concept                                  | Meaning                                                                                                                                               |
| ---------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| Person                                   | A human, known by an owner key that certifies their device keys (PRV-10). Holds one home and one personal room on each of their nodes.                |
| Node                                     | One machine running Cairn for one person (I8).                                                                                                        |
| Harness                                  | The external program that runs agents and reports what it sees through hooks. Cairn never owns or manages it; its sessions are its own.               |
| Agent                                    | A worker a harness runs for one person, its principal.                                                                                                |
| Run                                      | One agent run as the harness reports it, on one node. Holds seats; its recall taint, sandbox state, seat keys, kernel namespace and spans live on it. |
| Subagent                                 | An agent another agent's harness started: its own run, linked to its parent's run, with its own seats and writers.                                    |
| Bot                                      | A non-human seat holder run outside Cairn with its own key, such as the facilitator.                                                                  |
| Seat                                     | One device of a person, one run or one bot, in one room. Its seat id comes from the room and the seat key.                                            |
| Writer                                   | One seat's append-only log on one node. An event's address is (writer, seq).                                                                          |
| Room                                     | An intent, a conversation, seats, pins and branches. It links branches in any number of repositories.                                                 |
| Personal room                            | Every person's private room on their node. Every run sits in it from its first event, with no join.                                                   |
| Pin                                      | Text that belongs to exactly one room, with one author and numbered versions. The intent is the room's lead pin.                                      |
| Stamp                                    | A person's act that makes one pin version restore to that person's own agents.                                                                        |
| Room act, owner act, expire act          | The three signed act kinds that room state derives from (LANE-31).                                                                                    |
| Repository, branch, commit, pull request | Git and forge facts a room links to, never containers of Cairn state.                                                                                 |

## Relations

- An agent's principal is its person: the person whose node started
  it. Principal is this relation, not a concept of its own.
- A run holds its personal-room seat from its first event, plus one
  seat per room it joined. Joining is explicit, never automatic.
- A subagent's run links to its parent's run. Ingest splits one
  harness transcript by agent.
- Every seat belongs to one person and one room; every writer belongs
  to one seat, on one node.
- Each event goes to exactly one seat's writer: the seat of the room
  the run is working in at that moment (a room it joined that holds
  its current branch), else its personal-room seat. A room act goes
  to the writer of the seat that signs it.
- A run's history spans its seats' writers, joined through the run.
  Peers share writer logs, so a room sees only work routed to its
  seats.
- People and agents create rooms; an agent's room is owned by its
  principal. Cairn never creates a room on its own; it may suggest
  one.
- Every pin belongs to exactly one room; no pin exists without one. A
  pin with no other room belongs to its author's personal room.
- A person's seats are shown grouped under that person.
- Only a person widens trust; an agent never does.
- A room links branches in any number of repositories; a branch with
  a pull request links to it. A repository's identity, its root
  commit, links branches and commits and holds no Cairn state.
- Recall defaults to the agent's own run; it widens only to rooms the
  agent holds a seat in, or to a foreign room named in the call.
- Room state is a function of the recorded acts, never of a clock.

## Not Cairn concepts

| Term                 | Why                                                        | Where it may still appear                                                                                                                  |
| -------------------- | ---------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| session              | It belongs to the harness; Cairn speaks of runs.           | When naming a harness interface: the `SessionStart` hook, the harness's "allow for session", the harness's transcript id as an ingest key. |
| project              | A room links repositories; nothing is scoped to a project. | The harness's `~/.claude/projects` path, the harness settings scope `--scope project`, and Cairn's own software project.                   |
| tenant               | Renamed to person.                                         | The invariants I4 and I8, until a security review rewords them.                                                                            |
| participant, player  | Replaced by seat.                                          | Nowhere.                                                                                                                                   |
| lane                 | Renamed to room.                                           | The LANE and VIEW requirement ids and file names.                                                                                          |
| judge, approval gate | Removed by the stakeholder; a person records a verdict.    | Nowhere.                                                                                                                                   |
| hide                 | Removed by the stakeholder.                                | OQ-34, as a deferred question.                                                                                                             |

## Names follow the model

A function, type, module, crate, CLI verb, MCP tool, configuration key
or event is named after the concept it handles, such as `room_post`,
`seat`, `writer` or `stamp`. Documentation, UX and UI copy, error and
help text, logs and developer setup use the same words the SRS uses.

## Changing the model

A new concept, a renamed one or a new relation is a stakeholder
decision. It lands in this document before any requirement, name or
screen uses it. The domain-model agent reviews every change to this
document and every proposal to change it: it checks the model against
itself and every definition elsewhere against the model, names the
invariants whose wording would change, and lists every use the change
makes stale. It also reports every use that runs ahead of the model.
