# Entity model: data, and compute with data

Draft of 4 October 2026, from the stakeholder's ask to sort out which
entities are data and which compute and hold data. The pitch's rooms,
links, messages between rooms and delegation rest on this split.

## The split

**Data** is inert. It can be stored, read, linked and shown. It never
acts and never instructs. Every piece of data has an address in the
record and a writer.

**Compute with data** acts: it reads data, decides or executes, and
writes new data. It holds state of its own (a context window, a
process, a running app). Only compute entities act, and only one kind
of them, a person, can instruct an agent.

## Data

| Entity           | Written by                   | What it is                                                        |
| ---------------- | ---------------------------- | ----------------------------------------------------------------- |
| Event            | the compute entity behind it | One entry of the record: a message, an edit, a command, a result  |
| Intent           | a person                     | A goal, what done means and the rules, versioned                  |
| Pin              | a person                     | A rule to restore to agents; its model for rooms is open          |
| Message          | a person or an agent         | A post in a room or to another room                               |
| Mark and link    | a person or an agent         | An address plus a range: characters, lines or an image region     |
| Snapshot         | Cairn, from a compute entity | A screenshot of the app, a diff, a file version, a test log       |
| Checkpoint       | Cairn, from the worktree     | The state of a worktree at a point                                |
| Verdict          | a person                     | Met or needs changes, on a criterion                              |
| Grant            | a person                     | A delegation budget: which agents, which targets, how much, until |
| Room             | derived by Cairn             | The view of one intent's events, players and links                |
| Lane             | derived by Cairn             | A branch and its worktrees, as the record holds them              |
| Status and queue | derived by Cairn             | What needs whom, rebuilt from the record (I10)                    |

## Compute with data

| Entity                  | Holds                     | Acts by                                     | Trust                                                          |
| ----------------------- | ------------------------- | ------------------------------------------- | -------------------------------------------------------------- |
| Person (player)         | keys, judgement           | writing intents, messages, verdicts, grants | The only source of instructions to their own agents            |
| Agent                   | a context window          | calling tools through its harness           | Swayed by what it reads; Cairn cannot stop that, only limit it |
| Harness                 | a process, its transcript | running the agent's tool calls              | Its own input channel is the person's                          |
| Subagent and delegate   | a context window          | as an agent, within its parent's grant      | No looser than its parent                                      |
| Running app, dev server | a process, its pages      | executing code an agent wrote               | Untrusted; its screens are untrusted data                      |
| Command and test run    | a process, briefly        | executing, then exiting with a status       | Its output is untrusted data; its exit status is structural    |
| Cairn                   | the record, derived state | recording, deriving, delivering, displaying | Deterministic, no model, never judges (I10, CMP-09)            |
| Peer node               | another person's record   | syncing data by key                         | Its data arrives as theirs, untrusted                          |
| Model provider          | outside Cairn             | serving the harness's model calls           | Outside every Cairn boundary                                   |

## Rules that follow

1. **Data never instructs.** Only a person's words reach an agent as an
   instruction, through the harness's input (I2). Messages, links,
   snapshots and results reach an agent only when it asks, marked
   untrusted.
2. **Compute writes data, attributed.** Everything a compute entity
   produces enters the record as data under its writer: an agent's
   message, a command's output, a screenshot of a running app.
3. **Links point at data, never at compute.** You cannot link to a
   running app; marking it takes a snapshot first, and the link points
   at the snapshot. A link resolves the same way forever.
4. **Cairn computes but never judges.** It records, derives and
   displays deterministically. Judging an outcome is a person's act;
   presenting it is the agent's.
5. **Delegation is the one bounded path from compute to compute.** An
   agent hands a task to another agent only under a grant a person
   wrote; the task arrives as data marked as from the delegating agent,
   within the grant's budget.
6. **Untrusted compute is isolated.** A running app executes code no
   person reviewed; its pages stay out of the agent's context unless an
   agent fetches them as untrusted data, and out of the lane view's own
   origin.

## Where this settles the open questions

- **Messages between rooms** are data: an agent may write one, and it
  instructs no one until a person passes it on.
- **Handing tasks to another room** is rule 5: possible only under a
  grant, not by message.
- **Pinning in a room** stays open: a pin is data, and the open part is
  which agents restore which pins and when.
