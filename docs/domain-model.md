---
summary: >-
  Cairn's domain model: the closed set of concepts, how they relate,
  the terms that are not Cairn concepts, and how names in code, docs
  and UI follow the model. The domain-model agent reviews against it.
---
# Domain model

Cairn speaks in a small, closed set of concepts. The requirements, the
scenarios, the code, the documentation and every screen use them, and
only them. The glossary in [§1.6](srs/01-introduction.md) holds each
term's full definition; this document holds the closed list, the
relations between the concepts, and the terms that are not Cairn
concepts.

## Concepts

| Concept                                  | Meaning                                                                                                                 |
| ---------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| Person                                   | A human, known by an owner key that certifies their device keys (PRV-10).                                               |
| Node                                     | One machine running Cairn for one person (I8).                                                                          |
| Harness                                  | The external program that runs agents and reports what it sees through hooks. Cairn never owns or manages it.           |
| Agent                                    | A worker a harness runs for one person, its principal.                                                                  |
| Bot                                      | A non-human participant, such as the facilitator.                                                                       |
| Seat                                     | One device of a person, one agent run or one bot, in one room. Its participant id comes from the room and the seat key. |
| Writer                                   | One seat's append-only log on one node. An event's address is (writer, seq).                                            |
| Room                                     | An intent, a conversation, seats, pins and branches. It links branches in any number of repositories.                   |
| Personal room                            | Every person's private room. Every agent is in it from its first event, with no join.                                   |
| Pin                                      | Text that belongs to exactly one room, with one author and numbered versions. The intent is the room's lead pin.        |
| Stamp                                    | A person's act that makes one pin version restore to that person's own agents.                                          |
| Room act, owner act, expire act          | The three signed act kinds that room state derives from (LANE-31).                                                      |
| Repository, branch, commit, pull request | Git and forge facts a room links to, never containers of Cairn state.                                                   |

## Relations

- Every pin belongs to exactly one room; no pin exists without one.
- Every seat belongs to one person and one room; every writer belongs
  to one seat.
- A person's seats are shown grouped under that person.
- Only a person widens trust; an agent never does.
- A room links branches; a branch with a pull request links to it.
- Room state is a function of the recorded acts, never of a clock.

## Not Cairn concepts

| Term                       | Why                                                        | Where it may still appear                                         |
| -------------------------- | ---------------------------------------------------------- | ----------------------------------------------------------------- |
| session                    | It belongs to the harness.                                 | When naming a harness interface, such as the `SessionStart` hook. |
| project                    | A room links repositories; nothing is scoped to a project. | Nowhere.                                                          |
| lane                       | Renamed to room.                                           | The LANE and VIEW requirement ids and file names.                 |
| judge, approval gate, hide | Removed by the stakeholder.                                | Nowhere.                                                          |

## Names follow the model

A function, type, module, crate, CLI verb, MCP tool, configuration key
or event is named after the concept it handles, such as `room_post`,
`seat`, `writer` or `stamp`. Documentation, UX and UI copy, error and
help text, logs and developer setup use the same words the SRS uses.

## Changing the model

A new concept, a renamed one or a new relation is a stakeholder
decision. It lands in this document and in the glossary before any
requirement, name or screen uses it. The domain-model agent reports
every use that runs ahead of the model.
