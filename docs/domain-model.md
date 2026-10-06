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
of concepts with their definitions, the relations between them, and
the terms that are not Cairn concepts. The
[specification](srs/index.md) defines no terms of its own; it links
here, and any other text that defines a term says what this document
says.

## Concepts

Each concept, with its definition.

- **Person**: A human, known by an owner key that certifies their device keys
  (PRV-10). On each of their nodes a person holds one home and one personal
  room, and is the principal of every agent their node starts.
- **Home**: Directory holding all of one person's Cairn state on a node
  (`CAIRN_HOME`, default `~/.cairn`), owned by one OS user and optionally
  bound to a home ID the runner environment supplies (SEC-03).
- **Repository**: A git repository, identified by a bound root commit, the
  same on every node holding a clone, shallow clones included (LANE-02). A git
  fact rooms use to link branches and commits, never a container of Cairn
  state.
- **Branch, commit, pull request**: Git and forge facts a room links to, never
  containers of Cairn state.
- **Harness**: The external program that runs agents, such as Claude Code or
  the Agent SDK, and reports what it sees through hooks. Cairn never owns or
  manages it; its transcripts and hook payloads are its own, and a harness's
  transcript id serves Cairn only as an ingest key.
- **Agent**: A worker a harness runs for one person, its principal (OWN-01).
- **Run**: One agent run as the harness reports it, on one node. A run holds
  seats: its personal-room seat from its first event, plus one per room it
  joined (LANE-23). Its recall taint (SEC-13), sandbox state (OWN-22), seat
  keys (SEC-10), kernel namespace (CMP-02) and spans (LMK-01) live on it. Its
  history spans its seats' writers, joined through the run (LANE-01).
- **Subagent**: An agent another agent's harness started for it: its own run,
  linked to its parent's run, with its own seats and writers. Ingest splits
  one harness transcript by agent (REC-02).
- **Source**: A transcript file that Cairn ingests.
- **Record**: The set of append-only writer logs a node holds. The source of
  truth.
- **Event**: One immutable entry in the record: a message, tool call, tool
  result, hook observation, or administrative action.
- **seq**: An event's position in its writer's log: strictly increasing,
  gap-free, never reused. With the writer it forms the event's address.
- **Payload**: The full content of a large event, stored outside the event row
  under a name that confirms no guess at its content (REC-09).
- **Provenance**: Where an event's content came from ([SRS
  §5.2](srs/05-functional-requirements.md)).
- **Trust level**: `trusted` or `untrusted`, derived from provenance and
  writer by this node's trust policy (PRV-02).
- **Taint**: The trust level inherited by a derived artifact: untrusted if any
  source event is untrusted.
- **Span**: A contiguous range of one run's events, bounded by user turns,
  compactions, or subagent boundaries.
- **Landmark**: A structural headline describing a span, bound to its exact
  `seq` range.
- **Pin**: Text that belongs to exactly one room, stored verbatim, written by
  a seat holding the pin capability, its one author, with numbered versions
  (LANE-26). A pin with no other room belongs to its author's personal room.
  Only pins an agent's own person wrote or stamped restore (PIN-10, LANE-32).
- **Quarantine**: Exclusion of events or artifacts from recall and injection,
  recorded as an event, without deletion.
- **Envelope**: The structured wrapper in which recalled content is returned
  to Claude, marking it as untrusted historical data.
- **Restore block**: The deterministic text Cairn injects after compaction:
  pins, landmark index, and a recall hint.
- **Working view**: Whatever is currently in the model's context window. A
  projection; never the source of truth.
- **Kernel**: The layer-4 compute environment in which Claude runs code over
  the record.
- **Room**: Where an intent is worked on: at most one intent (its lead pin),
  its conversation, seats and pins, and zero or more branches in any number of
  repositories, each branch one attempt; the unit Cairn records, shows and
  shares. People and agents create rooms, an agent's room owned by its
  principal; Cairn never creates one on its own (LANE-01). A branch stays in
  the room it was opened in. An event belongs to the room of the seat whose
  writer it was routed to (LANE-01). Room requirements keep the ids LANE-xx
  (room) and VIEW-xx (room view).
- **Personal room**: Every person's private room on their node. Every run sits
  in it from its first event, with no join, and an event goes to its run's
  personal-room seat whenever the run is working in no room it joined
  (LANE-01).
- **Foreign room**: A room whose owner is another person, held from a bundle
  or a peer. Untrusted, and outside every widened recall scope unless named in
  the call (RCL-05).
- **Writer**: One seat's append-only log on one node, identified by its writer
  key. Each event goes to exactly one seat's writer (LANE-01).
- **Node**: One machine or sandbox running Cairn under one person's home.
- **Address**: (writer, seq), shown as short `A2·4812`, range `A2·4812–5025`
  or full `cairn:room/<room>/w/<writer>/<seq>`. Wherever an address is taken,
  on the CLI and by the MCP tools (`get`, `expand`), every shown form is
  accepted (RCL-08), and so is one ASCII input form, `<writer>:<seq>` or
  `<writer>:<from>-<to>` (for example `w-1:12`, `w-1:30-40`), where `<writer>`
  is the writer id or its shown label; it is equivalent to the short and range
  forms.
- **Segment**: A closed, signed range of one writer's log, sealed as a unit;
  what peers exchange. A unit of the record, not a file or any other storage
  layout.
- **Seal**: A writer key's signature over its writer id, a `seq` and the chain
  head at that `seq` (REC-18). Events after the newest seal are unsigned.
- **Commitment**: A keyed commitment to the content under a per-event random
  key held with the content and erased with it; the algorithm is chosen by
  ADR. The only way the chain, seals and tombstones refer to content (REC-17).
- **Core**: The B0 component: hooks, MCP server, kernel worker, CLI and TUI. A
  role with a boundary, not a packaging unit; whether it shares an executable
  with other components is not decided (OQ-32).
- **Writer certificate**: A device key's signature over a writer key, scoped
  to its seat's room, so every writer key chains owner → device → writer
  (PRV-10).
- **Boundary**: One of B0 core, B1 machine, B2 peer, B3 public. Every
  component sits behind exactly one (SEC-19).
- **Owner**: The person who holds a room: its intent, roles, admission and
  successor. Changes only by handover (LANE-11).
- **Co-author**: A person other than the owner who joins a shared room with
  their own agents and posts there, holding a role of LANE-16.
- **Owner key**: The person's root signing key, held offline or in a
  platform-protected key store, which certifies device keys (PRV-10).
- **Device key**: A node's or device's key, certified by the owner key with a
  scope and a maximum rule level.
- **Owner act**: A recorded `operator` event an authenticated owner surface
  wrote, in one of three classes (OWN-11). Cut: deny, interrupt, pause, stop,
  cancel a delegation, end a grant, reject a foreign room, record a `needs
  changes` verdict, quarantine of untrusted content, withdraw a risk
  acceptance, revoke a trust grant, leave a room, kick, bar, set read only,
  revoke an operator appointment, unstamp a pin version, and turn notices off.
  Neutral: create a room, mark a room ready, comment, acknowledge an overlap,
  record a `met` or `not met` verdict, link a result to a criterion, join a
  room or accept a join, present, and pick what the outcome window shows.
  Widen: allow, answer a hand-off, reply, steer, send a correction, retry from
  a checkpoint, set, revise or adopt an intent, resume, record a delegation or
  acceptance grant, issue an invite link, endorse, change a rule or away
  policy, add, change or end a pin, quarantine that removes a pin or a trusted
  event from a restore block, release a quarantine, purge or answer an erasure
  request, export, bind a repository identity, force-push, choose a fork,
  accept open residual risks (OWN-22), invite or change a role, set a room's
  admission, appoint an agent or a bot to operator, unbar, lift read only,
  stamp a pin version, name a successor, hand over a room, record a trust
  grant, allow notices for a room or opt in to them, enable a bridge, retire a
  writer, enrol or revoke a device, peer or authenticator, mint or rotate a
  token, confirm a recorded command or start a witness run, restore a backup.
  Any act that removes a pin from a restore block or stops events being
  accepted is widening, whatever verb carries it, except a room's kick, bar
  and read only, which only refuse room acts (LANE-25), and an unstamp, which
  withdraws only the unstamper's own trust (LANE-32); an unlisted owner act is
  widening. An `expire` act is no owner act: a node records it, signed with
  its device key, ending only an expiry its original act set (LANE-25). OWN-11
  and OWN-12 follow this list, and a gate fails when a requirement names an
  owner act this entry does not classify.
- **Room act, expire act**: With owner act, the three signed act kinds that
  room state derives from (LANE-31).
- **Held request**: A permission request, question or hand-off recorded with a
  stable id, answerable from any owner surface (OWN-05).
- **Post**: A message written to a room board by a person or by an agent's
  `room_post` call. Provenance class `post`, always untrusted.
- **Endorsement**: A principal's signed act that sends a post's displayed text
  to one of their own agents, inside a fixed template naming its source
  (OWN-08).
- **Notice**: Opt-in trusted text saying posts wait or a room changed: Cairn's
  own ids, version numbers and counts, key fingerprints and recall addresses
  only, sent only while the room's owner allows notices and the agent's own
  person opted in (INJ-10, LANE-30).
- **Petname**: The name the viewer's own contacts bind to a key. Never a name
  a peer sends.
- **Evidence class**: Of a result: `claim`, `own run`, `witness run` or `CI
  attested` (LANE-05).
- **Proof class**: Of the link from a landed commit to a room: `same commit`,
  `same patch`, `same tree`, `likely`, `asserted` or `not proven` (LANE-06).
- **Witness run**: A command re-run through the run component on a fresh
  checkout of the exact commit by a node that authored nothing in the room.
- **Rule level**: Per action class: act without asking, act when told, ask
  first, hand off (OWN-10).
- **Away policy**: The owner's opt-in choice of what an unanswered hold does:
  keep going, pause or stop (OWN-07).
- **Presence check**: A hardware-backed, user-verified proof of the owner's
  presence, bound to the act it approves (OWN-11).
- **Room view**: The local surfaces (browser, TUI, CLI) that show rooms. A
  client of the record, never the source of truth. Distinct from **Working
  view**, the model's context window.
- **Catch up**: The one surface answering "what happened since a boundary"
  (VIEW-08).
- **Needs you**: The one queue of items waiting on a person (VIEW-05).
- **Bundle**: A signed, reviewed export of a room, carried as a file or a git
  ref.
- **Bound human**: One owner key and every device, writer and seat key that
  chains to it. Cairn counts humans by owner key, as a `witness run` does
  (LANE-05).
- **Sandbox state**: What confines a run, its policy digest, and which
  residual risks of [SRS §6.1](srs/06-security.md) it blocks, recorded from a
  source the agent cannot write (OWN-22).
- **Risk acceptance**: The owner's recorded act accepting the residual risks
  an unsandboxed or partly sandboxed run leaves open (OWN-22).
- **Blind peer**: A peer, possibly run by another entity, that stores and
  serves a room's ranges encrypted to its members' keys and reads none of them
  (PEER-12).
- **Delegation**: Work one agent hands another: its subagent, or, under a
  grant, another run, node or principal's agent (OWN-23 to OWN-26).
- **Delegation grant**: A principal's widening owner act naming who may
  delegate, to which targets, at what rule level, within what budget and until
  when (OWN-23).
- **Visibility**: Whether a room is private, shared with members, published or
  stored on blind peers (LANE-17).
- **Intent**: What a room is for: a goal and acceptance criteria, each with a
  stable id, and optionally the paths it is meant to change, written by its
  owner and stored as the room's lead pin, of type `intent`, versioned like
  every pin (LANE-20). Verdicts on the outcome are recorded against it.
- **Verdict**: A person's finding on an outcome against the intent, on each
  criterion of the room's intent: `met`, `not met` or `needs changes`,
  recorded as a pin of type `verdict` they write, an owner act under their own
  key (OWN-27). Cairn never derives one.
- **Attempt**: One branch in a room: one try at the room's intent from a base
  commit, worked by the seats given that branch. It ends when its commits land
  or its branch is closed (LANE-01).
- **Seat**: One device of a person, one run or one bot, in one room, under the
  seat id that joining returns, or, for a run's personal room, from its first
  event with no join (LANE-23). While its add stands and no kick or bar covers
  it, it holds its place in the room (LANE-25). A person on several devices
  holds several seats, shown grouped under that person through the owner →
  device → seat chain (LANE-23, PRV-10).
- **Seat id**: A seat's id in one room, derived from the room's id and the
  seat key it joined with. A run's seat key is handed to Cairn's MCP server
  for that run, which holds it in memory only (SEC-10); a run's seat ids
  follow from the joins its seats' writers record (LANE-23), and every room
  act is signed with that key (LANE-24).
- **Bot**: A non-human seat holder with its own key, run outside Cairn and
  reading a room only through the tools an agent uses, such as the facilitator
  bot (SEC-32).
- **Room role**: A named set of room capabilities: viewer, contributor or
  operator, assigned by the owner; a person holding operator may also appoint
  an agent or a bot to operator (LANE-16).
- **Operator (room role)**: The room role holding a contributor's capabilities
  plus branch, unpin of any pin but the intent, pick, kick, bar and read only.
  The owner assigns it; a person holding it may appoint an agent or a bot to
  it, revocable by the appointer or the owner, and an operator that is not a
  person holds it only within SEC-32's limits (LANE-16). Unrelated to the
  `operator` event class.
- **Facilitator bot**: A bot that a person holding operator appointed to
  operator, run outside Cairn, holding operator capabilities within SEC-32's
  limits plus posting findings in its own words against the pins and writing
  room summaries (LANE-33). A person may trust it by a grant (LANE-16, SEC-32,
  OWN-29).
- **Kick and bar**: Moderation acts: a kick revokes a seat's current add; a
  bar names a person's owner key or a bot's key and keeps every key it
  certified out until lifted (LANE-25).
- **Outcome window**: A room's shared view of the work beside its
  conversation: seats present to it, and a `pick` chooses which present it
  shows (VIEW-22).
- **Trust grant**: A person's recorded, revocable act trusting a poster's key
  for their own agents, in one room or everywhere; never an agent's key
  (OWN-29).
- **Room trailer**: The `Cairn-Room:` and `Cairn-Link:` lines added to every
  commit made in a room; an assertion until the record proves the link
  (LANE-28).
- **Stamp**: A person's signed owner act on one version of a room pin, so that
  version restores word for word to that person's own agents only. A new
  version needs a new stamp, and an unpin ends every stamp. The only way a pin
  an agent wrote becomes active (LANE-32).
- **Concurrent**: Of two room acts or events: neither is causally after the
  other, so the record gives them no order (LANE-31).
- **Room merge**: The one rule deriving all room state from signed room acts,
  owner acts and node-recorded `expire` acts in causal order, with no clock:
  of concurrent acts on one object, the more restrictive wins, then the lower
  commitment, and every resolved conflict is shown (LANE-31).

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

| Term                 | Why                                                                              | Where it may still appear                                                                                                                  |
| -------------------- | -------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| session              | It belongs to the harness; Cairn speaks of runs.                                 | When naming a harness interface: the `SessionStart` hook, the harness's "allow for session", the harness's transcript id as an ingest key. |
| project              | A room links repositories; nothing is scoped to a project.                       | The harness's `~/.claude/projects` path, the harness settings scope `--scope project`, and Cairn's own software project.                   |
| tenant               | Renamed to person: the invariants' word for the person whose home a node serves. | The invariants I4 and I8, until a security review rewords them.                                                                            |
| participant, player  | Replaced by seat.                                                                | Nowhere.                                                                                                                                   |
| lane                 | Renamed to room.                                                                 | The LANE and VIEW requirement ids and file names.                                                                                          |
| judge, approval gate | Removed by the stakeholder; a person records a verdict.                          | Nowhere.                                                                                                                                   |
| hide                 | Removed by the stakeholder.                                                      | OQ-34, as a deferred question.                                                                                                             |

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
