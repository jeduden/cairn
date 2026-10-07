---
summary: >-
  Cairn's domain model: the closed set of concepts with their
  definitions, how they relate, the terms that are not Cairn concepts,
  and how names in code, docs and UI follow the model. The SRS links
  here for every term, and the domain-model agent reviews against it.
---
# Domain model

Cairn speaks in a small, closed set of concepts. Requirements, scenarios, code,
documentation and every screen use only them. These files are their one source;
the [specification](../srs/index.md) defines no terms and links here. Every
Cairn concept has one name and every word one meaning. Where English uses one
word for several things, each sense gets a qualified name, such as a branch link
or a held request. An outside thing keeps its own name when its domain qualifies
it: a network or email address, a command-line flag, a code owner or a persona
name.

These verbs each have one job:

- A principal *owns* a room.
- A seat *is a member of* a room as Member defines.
- A principal, an agent or a run *has a seat in* a room through any of its seats
  that is a member, its agents' run seats included; an agent has a seat in a
  room through its runs' seats that are members.
- A node *holds* writer logs, rooms and whatever else it stores. "Owns" is said
  of rooms and "holds" of nodes ("held request" and "hold window" are names);
  "holds a room" never means owning a room or being a member of it.

## Concepts

The concepts live in one file per group. Read this hub first, then the
group a term belongs to.

<?catalog
glob: "*.md"
where: 'order: string'
sort: order
header: ""
row: "- [{title}]({filename}) — {summary}"
?>
- [Principals and agents](principals-and-agents.md) — Who acts in Cairn: principals, persons and service accounts, managed policy, agents, runs and subagents, the facilitator, authors, members, the owner and delegates.
- [Harness facts](harness-facts.md) — The harness's own facts that Cairn records and names: the harness, harness sessions and transcripts, hooks, turns and compaction.
- [Places](places.md) — Where Cairn's state lives and is shared: homes, nodes, devices and paired phones, sandboxes, rooms and their kinds, visibility, peers and blind peers.
- [Record](record.md) — The record and what derives from it: writers, events, addresses, segments, seals, provenance and origin, spans and landmarks, quarantine and purge, receipts, backups, bundles and exports.
- [Pins and context](pins-and-context.md) — What may reach a model: pins and their versions, types and budget, the intent, stakes, posts, room summaries, envelopes, restore blocks, opt-in notices and held requests.
- [Seats and keys](seats-and-keys.md) — Seats and the keys behind them: run and device seats, room and seat ids, seat, device, principal and token keys, certificates, access tokens, authenticators and CI keys.
- [Acts and roles](acts-and-roles.md) — The three act kinds and what they may do: room acts, principal acts and their classes, expire acts, roles, appointments, joins, moderation, handover, stamps and the room merge.
- [Git and forge](git-and-forge.md) — Repositories, branches and commits, the forge, results with their evidence and proof classes, qualified links, comparisons, outcomes and verdicts.
- [Trust and flow](trust-and-flow.md) — Trust and how content reaches an agent: trust levels and sources, recall, endorsement, trust grants, delegation, rule levels, away policies, fixed templates, sandbox state and risk acceptance, quotas and spend, counters, canary and stats, run status and room status, the Needs you queue and focus set, overlap, forks, presence hints and petnames.
- [Components and surfaces](components-and-surfaces.md) — Cairn's components inside their network boundaries, the room view's surfaces and the outcome window.
<?/catalog?>

## Relations

- A principal owns any number of rooms; an agent's principal follows from its
  node.
- A run has its personal-room seat from its first event, plus a further run seat
  per room it joined or created, a new one for each seat key minted anew, and
  the seat ingest starts beside any of them (Run seat).
- A restore block carries the qualifying pins of every room its run has had a
  seat in during the run, its personal room included (PIN-10); a leave, kick or
  bar keeps them, but once the room is foreign only those Foreign room names.
- Ingest splits one transcript by run.
- Every seat belongs to one principal and one room; every writer to one seat.
- Each event goes to exactly one seat's writer. A run's event goes to its run
  seat in the room it works in at that moment, one it joined or created that
  names the branch of the run's latest preceding event naming its branch, while
  that seat is a member and has work (its role, no mute), per the room state
  this node holds when it records the event (LANE-01, LANE-16); else to its
  personal-room seat; what `cairn ingest` appends for a witnessed run goes
  instead to the seat ingest starts beside that run seat (Run seat). A room act
  goes to the writer of the seat that signs it. A principal act or an expire act
  goes to the device seat of the device that signs it, or before PRV-10 ships
  records it, in the room it acts on; a node of a principal with a member seat
  there first joins it without admission. One that acts on no room, on a room
  its principal has no member seat in, or that a paired phone signs, goes to
  that device seat in the personal room, naming the room, and that room shows it
  by address as it shows a cross-room post (LANE-29). A seat key's rotation goes
  to that seat's own writer (SEC-27). A tombstone, an erasure request a
  retention policy sends, and a bridge's or the launcher's event about a room's
  branch go to the recording device's seat in the room they name, joined as for
  a principal act, else to that device seat in the personal room, naming the
  room. Any other event with no run goes to the recording device's seat in the
  personal room of its node, or for a paired phone, of the node it pairs with.
- A run's history spans its seats' writers, tied together by the run. Peers
  exchange segments, so a room contains only the events routed to its seats and
  shows others by address (LANE-01).
- Principals and agents create rooms; an agent's room is owned by its principal.
  Cairn never creates a room on its own initiative; a node's personal room comes
  from the principal installing Cairn there (Personal room), or is carried over
  to a node clone, and Cairn may suggest a room or a join.
- A pin naming no room belongs to its author's principal's personal room on the
  node that wrote it.
- Recall extends only to the principal's
  rooms the agent has a seat in and the cross-room posts they show, never to a
  foreign room unless the call names it.

## Names follow the model

Code, CLI verbs, MCP tools, settings keys, events, documentation, UI copy,
errors and logs use the model's words.

- An MCP tool for a room act is `room_<act>`, but `room_create` for create room.
  Every other MCP tool is `<concept>_<verb>`, with the reading verbs `get`,
  `list`, `search` and `expand`.
- A CLI command that acts on a concept is `cairn <concept> <verb>`. Node-wide
  utilities keep one word: `install`, `uninstall`, `status`, `doctor`, `verify`,
  `rebuild`, `migrate`, `ingest`, `import`, `export`, `purge`, `audit`,
  `canary`, `ui`, `why`, `open`, `launch`, `hook`, `mcp` and `kernel-worker`.
- An option that selects a concept is named for it: `--author`, `--seat`,
  `--writer`, `--run`, `--room`.
- A **settings key** names one setting in a settings layer:
  `<concept>.<setting>`, or `<concept>.<room>.<setting>` or
  `<concept>.<room>.<qualifier>` per room, the concept singular and in snake
  case (`node.deployment_mode`). A multi-word concept is snake case in MCP tools
  and settings keys, kebab case in CLI commands.
- Layout words (such as tab, panel, pane, gutter, sheet, stack, tile, pill,
  rail, composer and command palette) name parts of a screen, never concepts.
- Cairn defines no slash commands. A harness skill may call MCP tools or the
  CLI; a skill's call is the agent's own tool call, never a principal act.

## Not Cairn concepts

| Term                                                                              | Why                                                                                            | Where else it may appear                                                                                                 |
| --------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| session                                                                           | It belongs to the harness; Cairn speaks of runs.                                               | As "harness session", or inside a harness's own names, such as `SessionStart`, `session_id` and `allow-session`.         |
| project                                                                           | A room names branches in repositories; nothing is scoped to a project.                         | The harness's `~/.claude/projects` path, the harness settings scope `--scope project`, and Cairn's own software project. |
| tenant                                                                            | Replaced by principal.                                                                         | Nowhere else.                                                                                                            |
| operator, as a role or a person                                                   | The role is moderator; the person running a node is the node's principal.                      | Only as the `operator` provenance class.                                                                                 |
| owner act, owner key, owner surface                                               | Replaced by principal act, principal key and principal surface.                                | Nowhere else.                                                                                                            |
| bot, co-author, actor, poster, bound human                                        | Replaced by service account, pull-request author, author (for actor and poster) and principal. | Nowhere else.                                                                                                            |
| writer key, writer certificate                                                    | Replaced by seat key and seat certificate.                                                     | Nowhere else.                                                                                                            |
| attempt, claim of work, read only (a seat state)                                  | Replaced by branch, `stake` pin and mute.                                                      | Nowhere else; "read-only" as an ordinary adjective (read-only publishing) stays.                                         |
| run component; own run and witness run as evidence classes                        | Replaced by launcher, own check and witness check.                                             | Nowhere else.                                                                                                            |
| away mode, room alias, child agent                                                | Replaced by away policy, petname and "a subagent or a delegate".                               | Nowhere else.                                                                                                            |
| trusted boundary                                                                  | Replaced by trusted sources; "boundary" means a network boundary.                              | Nowhere else.                                                                                                            |
| room board                                                                        | Replaced by conversation.                                                                      | Nowhere else.                                                                                                            |
| participant, player                                                               | Replaced by seat and member.                                                                   | Nowhere else.                                                                                                            |
| lane                                                                              | Renamed to room.                                                                               | LANE requirement ids and file names such as lane-view.feature.                                                           |
| judge, approval gate                                                              | Removed by the stakeholder; a person records a verdict.                                        | Nowhere else.                                                                                                            |
| service-account seat                                                              | Replaced by device seat.                                                                       | Nowhere else.                                                                                                            |
| import of a transcript or a peer's segments; imported run                         | Only a bundle is imported.                                                                     | Nowhere else.                                                                                                            |
| presence check; unqualified token; Needs you as a run or room status; work marker | Replaced by presence proof, access token or model token, Asking, and ingest marker.            | Nowhere else.                                                                                                            |
| hide                                                                              | Removed by the stakeholder; say the UI does not show it.                                       | The cryptographic term "hiding commitment" (REC-17).                                                                     |

Every excluded term may still appear in **historical records**: the SRS change
log, accepted ADRs and plan records, which keep the words of their time.
Excluded words, and words the model defines, such as `verdict`, may also appear
in that tool's meaning in files an outside tool writes and maintains, or that
name its fields. Examples are frit's plan skills and plan files, and the
repository's own engineering tooling, such as ENG-28's review gate and the
requirement naming its fields. The domain-model agent skips them.

## Changing the model

A new concept, a renamed one or a new relation is a stakeholder decision. It
lands here before anything uses it. The domain-model agent reviews every change
to these files and every proposal to change it. It names the invariants whose
wording would change, and every use the change makes stale or that runs ahead.
