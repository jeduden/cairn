---
summary: >-
  Cairn's domain model: the closed set of concepts with their
  definitions, how they relate, the terms that are not Cairn concepts,
  and how names in code, docs and UI follow the model. The SRS links
  here for every term, and the domain-model agent reviews against it.
---
# Domain model

Cairn speaks in a small, closed set of concepts. The requirements, the
scenarios, the code, the documentation and every screen use them, and only them.
This document is their one source; the [specification](srs/index.md) defines no
terms and links here. Every Cairn concept has one name and every word one
meaning. Where English uses one word for several things, each sense gets a
qualified name, such as a branch link or a held request. An outside thing keeps
its own name when its domain qualifies it: a network or email address, a
command-line flag, a code owner or a persona name.

These verbs each have one job:

- A principal *owns* a room.
- A seat *is a member of* a room while its add stands and no bar covers it; a
  run's or a paired phone's personal-room seat always is.
- A principal *has a seat in* a room through any of its seats that is a
  member, its agents' run seats included; an agent has a seat in a room
  through its run's seats.
- A node *holds* writer logs and rooms. "Owns" is said of rooms and "holds" of
  nodes holding writer logs and rooms ("held request" is a name); "holds a
  room" never means owning a room or being a member of it.

## Concepts

### Principals and agents

- **Principal**: A person or a service account. It has a principal key, signs
  principal acts with a device key at a principal surface, each recorded on a
  device seat, and is the principal of every agent its nodes start. Cairn
  counts principals by principal key: every device key, token key and seat key
  that chains to one principal key belongs to that principal. A service
  account its **certifier** (a person, another service account, or managed
  policy) certified is still its own principal, counted by its own principal
  key. An uncertified principal key counts as a person's, which nothing can
  prove. Only an agent's own principal widens what reaches that agent (I2).
- **Person**: A human principal. No one certifies a person's principal key.
  Only a person records a verdict.
- **Service account**: A non-human principal with its own principal key,
  certified, and revocable, by a person, another service account or managed
  policy; none is uncertified. It can own rooms, have seats and be the
  principal of its own agents, such as CI or runner agents.
- **Managed policy**: Configuration belonging to root that an organisation
  sets on a machine. Cairn never overrides it (I7). Among its powers, it may
  turn off boundaries B1 to B3, forbid risk acceptance and certify service
  accounts by listing their principal keys. The harness's own managed
  settings, which Cairn also never writes (ADM-03), are harness configuration.
  The only rule-setter that is not a principal.
- **Agent**: A worker a harness runs for exactly one principal: the principal
  of the node that started it (OWN-01). It receives restore blocks and recalls
  history; an agent is never a principal.
- **Run**: One agent run as the harness reports it, on one node. It has run
  seats and carries its recall taint (SEC-13), sandbox state (OWN-22), seat
  keys (SEC-10), kernel namespace (CMP-02) and spans (LMK-01). "Run" has no
  other meaning.
- **Subagent**: An agent another agent's harness started for it. It has its
  own run, tied to its parent's run by a parent link, with its own seats and
  writers. It takes a delegated task without a delegation grant.
- **Ingested run**: A run ingested from a transcript the hook handlers did not
  watch, with origin `ingested`, and untrusted (REC-22). A principal's own
  transcripts from outside `transcript.roots` land in its personal room.
- **Facilitator**: The one service account per room whose device seat, on its
  own node, the owner appoints as the room's facilitator, an appointed
  moderator (SEC-32); the program that writes its findings is not a Cairn
  component and acts through that node's CLI, signed with that device seat
  (`cairn room-summary write`). It may post findings in its own words and
  write room summaries, answering summary requests (LANE-33). Its posts reach
  another principal's agent as trusted text only through a trust grant, and
  its room summaries never.
- **Author**: The seat that wrote an event or a pin. The author's principal
  follows from the seat.
- **Member**: A seat whose add stands and that no bar covers, or a run's or a
  paired phone's personal-room seat (Seat, Join); the seat's role says what it
  may do. A kicked, departed or barred seat stays a seat but is no longer a
  member. A principal is never a member; **the room's principals** are those
  with a member seat: it has a seat in the room through any of its seats, and
  text for people speaks of the room's principals.
- **Owner**: The one principal who owns a room: its intent, roles, admission,
  appointments, successor and handover. Ownership changes only by handover or
  succession (LANE-11); it stays with the owner after every seat of the owner
  has left the room. The owner stands beside the roles rather than having one:
  a room act signed by its device seat in the room has every room capability
  but writing a room summary, while its agents' run seats have only what a
  role assignment or an appointment gives them. "Owner" means nothing else,
  except in the persona name "Returning owner".
- **Pull-request author**: The outside party whose commits a foreign room's
  bundle describes, matched through their commit-signing identity (LANE-15).
  May have no seat at all.
- **Delegate**: Any agent that receives a delegation, a subagent included
  (OWN-24). It keeps its own principal.

### Harness facts

The harness's own facts, which Cairn records and names but never keeps or
controls.

- **Harness**: The external program that runs agents, such as Claude Code or
  the Agent SDK. It keeps their transcripts, compacts their context and
  reports to Cairn through hooks. Cairn records it and never manages it.
- **Harness session**: The harness's own unit, which yields one transcript.
  Named only when describing the harness, never as a Cairn unit.
- **Transcript**: The harness's file of what a harness session did. A
  **transcript source** is a transcript Cairn ingests (**ingest**, `cairn
  ingest`); a rewritten prefix starts a new **transcript generation**
  (REC-07). As a Cairn term, "source" is a transcript source or a trusted
  source; the harness's `source` field and §6.1's threat sources keep their
  own sense.
- **Hook**: The harness's callback into Cairn, carrying a JSON payload (§9.1).
  Cairn's hook handlers, in the core, answer it.
- **Turn**: One exchange between the harness and the model, from an input to
  the reply that ends it. A **user turn** is the turn a person's message to an
  agent starts. In interactive deployment mode it is a trusted source on the
  node that recorded it (I2, PRV-02).
- **Compaction**: The harness replacing earlier context with a compaction
  summary when the context window fills. Cairn neither performs nor controls
  it (NG1); it records it and restores pins after it (I3).

### Places

- **Home**: The directory containing all of one principal's Cairn state on a
  node (`CAIRN_HOME`, default `~/.cairn`), belonging to one OS user and
  optionally bound to a home id the runner environment supplies (SEC-03). The
  unit of isolation (I8).
- **Node**: One home on one machine, container or sandbox, for one principal.
  Its device key signs its principal acts and expire acts; a node in an
  ephemeral sandbox may have only a token key. The node's principal is the
  principal whose home it is.
- **Device**: A node or a paired phone. A device key certifies its device
  seats; a token-key-only node's token key does so in its place.
- **Paired phone**: A device with no home, limited to reading and to allowing
  or denying held permission requests within its scope, reaching its node over
  B2 (or, in stage one, the room view through the principal's tunnel). It
  signs with its own device key and seals its device seat's writer with that
  seat's key; the node it pairs with holds the writer.
- **Sandbox**: A confinement around a run that blocks some residual risks
  (OWN-22). A node running inside a sandbox is still a node.
- **Room**: Where an intent is worked on: at most one intent, a
  **conversation** (its ordered posts), seats, pins, and branches in any
  repositories, each named by a branch link. The unit Cairn shows and shares.
- **Personal room**: A principal's private room on each of its nodes, created
  by the principal's `cairn install` as the first act of its device seat
  there. Its create room act is the device seat's add there. Every run sits in
  it from its first event, without a join, and every event or pin that belongs
  to no other room goes there.
- **Foreign room**: A room this node holds that its principal neither owns nor
  has a seat in, such as the room of an imported bundle or a peer's room.
  Untrusted, and outside every extended recall scope unless named in the call.
  A room the principal owns or has a seat in is never foreign.
- **Principal's rooms**: The rooms a principal owns or has a seat in.
- **Visibility**: Whether a room is private, shared with the room's
  principals, published or stored on blind peers (LANE-17). Changing it is a
  widening principal act of the owner.
- **Peer**: Another node this node enrolled by key and exchanges sealed
  segments with through the peer component (B2). Never trusted for content.
- **Blind peer**: A peer that holds a room's segments without the device key
  of any of the room's principals, so it stores and serves them encrypted and
  reads none of them (PEER-12).

### Record

- **Record**: The set of writer logs a node holds. Every derived artifact
  derives from it and the node's own key set (I10); the audit log, counters
  and configuration sit beside it. The **store** is the home's files holding
  them, payloads in the payload store.
- **Writer**: One seat's append-only log on one node, named by the seat's
  first key. Its events are hash-chained; the **chain head** at a seq is the
  hash over every event up to it.
- **Event**: One immutable entry in a writer: a message (a person's input or
  the model's reply), tool call, tool result, **hook observation** (what a
  hook reported), worktree checkpoint, key rotation, tombstone, or an act (a
  room act, a principal act or an expire act).
- **seq**: An event's position in its writer: strictly increasing, gap-free
  and never reused (REC-06).
- **Address**: (writer, seq), shown as short `A2·4812`, range `A2·4812–5025`
  or full `cairn:room/<room>/w/<writer>/<seq>`. Every shown form is accepted
  wherever an address is taken, and so is one ASCII input form,
  `<writer>:<seq>` or `<writer>:<from>-<to>` (for example `w-1:12`,
  `w-1:30-40`) (RCL-08). An address range lies in one writer. The only meaning
  of "address".
- **Payload**: The full content of a large event, stored outside the event
  under a name that confirms no guess at its content (REC-09).
- **Segment**: A range of one writer, sealed as a unit when it closes; what
  peers exchange. A unit of the record, not a storage layout. The **open
  segment** is a writer's newest, still growing; it closes at the points
  REC-19 names, and peers exchange its sealed prefix.
- **Seal**: A seat key's signature over its writer id, a seq and the chain
  head at that seq (REC-18), made where that key lives: a run seat's by its
  run's MCP server, covering what the hook handlers appended. Events after the
  newest seal are unsigned.
- **Commitment**: A keyed commitment to an event's content under a per-event
  random key kept with the content and erased with it; the only way the chain,
  seals and tombstones refer to content (REC-17).
- **Provenance**: An event's class from a closed set, saying what produced its
  content (§5.2). `operator` is the class of principal acts, expire acts and
  device-seat pins; posts are `post`, run-seat pins `assistant`, and a
  retention purge's tombstone is structural.
- **Origin**: How an event reached this node's record: `witnessed` (captured
  live by this node's hook handlers), `ingested` (read later from a
  transcript), `bundle` (read by import) or `peer` (received from a peer)
  (RCL-09). Independent of provenance.
- **Span**: A contiguous range of one run's events in one writer. A new span
  starts at every user turn, compaction, subagent start or end, and whenever
  the run's events move to another seat's writer (LMK-01).
- **Landmark**: A structural headline over one span, bound to its address
  range (LMK-02).
- **Structural field**: A field Cairn derives from an event's shape, never
  from its text: ids, kinds, counts, tool names, sanitized paths and exit
  status, sanitized under LMK-03.
- **Structural event**: An event whose meaning lies only in structural fields,
  such as a hook observation. An act is structural in its structural fields
  only, never in the text it carries.
- **Capture**: This node's hook handlers recording its runs' events; turning
  it off is widening (OWN-11).
- **Ingest marker**: What a hook handler leaves when its deadline cuts it
  short, so the next hook handler or `cairn ingest` resumes it (NFR-02).
- **Worktree checkpoint**: An event recording a worktree's commit, branch and
  redacted diff since the previous worktree checkpoint (REC-20). Always
  written in full, so it is never confused with the harness's rewind points.
- **Derived artifact**: Anything computed from the record (I10): indexes,
  landmarks, active pins, the quarantine set, statuses, queues, evidence and
  proof classes, and stats (I10's statistics).
- **Redaction**: Removing secrets from content before it is stored or on
  import, recorded (I1, SEC-08).
- **Retention policy**: A rule (`retention_policy.*`), set by the node's
  principal or managed policy, that purges content per room and provenance
  class after a time (I1, SEC-22).
- **Flag**: A mark Cairn sets on an event whose text matches an injection
  pattern (PRV-07); flagged text contributes only counts.
- **Quarantine**: Recorded, reversible exclusion of an event, span, run,
  writer or derived artifact from recall and from restore blocks, without
  deletion (I5).
- **Purge**: Deletion of content, leaving a **tombstone** in its place
  (ADM-07): by a principal act, or by the node under a retention policy,
  recorded naming the policy. The only way content is destroyed (I1).
- **Gap marker**: What stands where content is missing: a tombstone for a
  purged range, a quarantine marker for a quarantined address, or a truncation
  marker on capped kernel output (RCL-06, CMP-06).
- **Integrity status**: What a room or writer shows about its chain and seals,
  one of §9.7.5's values: `verified`, `unsigned`, `incomplete`, `unverified`,
  `broken`, `refused` and `equivocated` (VIEW-10). The UI never says "secure".
- **Receipt**: A signed statement about a node's record, made to be kept apart
  from the home and checked with no network. Always qualified: a **head
  receipt** lists every writer's chain head at a moment, the tamper evidence
  of VIEW-10 and SEC-27; a **purge receipt** states what a purge erased and
  what it could not (SEC-31).
- **Backup**: A copy of a home's record and keys (`cairn backup create`);
  reading it back is a backup restore.
- **At-rest key**: The key that encrypts the store when encryption at rest is
  on (SEC-09).
- **Bundle**: A signed, reviewed export of a room, carried as a file or a git
  ref.
- **Rendering**: A room rendered for people to read, with no keys or
  commitments (`cairn export --rendering`, SEC-26).
- **Import**: Reading a bundle into this node's record (`cairn import`,
  REC-23). A transcript is ingested and a peer's segments are received;
  neither is imported.
- **Kernel**: The hermetic compute environment in which Claude runs code over
  the record (CMP-01); each run's **kernel variables** live in its own
  namespace.

### Pins and context

- **Pin**: Verbatim text in exactly one room, with one author (a seat), a pin
  type and numbered pin versions. Information, never an instruction. It
  restores only under PIN-10, as a **qualifying pin**: a pin written from a
  device seat restores to its author's principal's agents, and to agents whose
  principal's trust grant covers its author; any version of a type that
  restores restores to the agents of a principal who stamped it. Adding,
  editing or unpinning a device-seat pin of a type that restores is a widening
  principal act of its author's principal, and a verdict is its own principal
  act (OWN-27); every other pin is changed by room acts; a run seat's restores
  only once stamped. A room act never changes any restore block (LANE-32).
- **Pin version**: One immutable text of a pin; each edit adds one, unstamped.
  What a stamp covers.
- **Pin candidate**: Proposed pin text an agent suggested or Cairn detected;
  not yet a pin, and with no author. Its principal's confirmation, a widening
  act, makes it a new pin its device seat authors (the owner's for an intent).
  A pin an agent's run seat wrote restores only once stamped (LANE-32).
- **Configuration pin**: A pin the principal's configuration declares,
  authored by its device seat on that node (PIN-01).
- **Pin priority**: The order in which pins fill the restore budget (PIN-03,
  PIN-08).
- **Pin type**: One of `constraint`, `preference`, `decision`, `fact`,
  `episode`, `intent`, `verdict` and `stake`. Only `constraint`, `preference`
  and `intent` pins restore.
- **Unpin**: The act that ends a pin: it takes the pin off the room's pin
  list, and its versions stay in the record (I1). It stops the pin restoring,
  with two exceptions, each raising a Needs you item: a stamped version keeps
  restoring to its stamper's agents until the stamper unstamps it, and after a
  moderator's unpin a device-seat pin keeps restoring to every agent it
  restored to until its author's principal unpins it.
- **Intent**: A room's lead pin, of type `intent`: a goal, its criteria and
  optionally the paths it is meant to change (LANE-20). Only the owner's
  principal act changes it.
- **Criterion**: One acceptance condition of an intent, with a stable id.
- **Stake**: A pin of type `stake` stating what its author works on. Only its
  author writes it, and it locks nothing.
- **Title, labels**: Room state the owner's device seat or a moderator sets: a
  display name and tags. They never reach a model.
- **Assignment**: Room state asking a seat to work on a branch. It locks
  nothing.
- **Post**: Text a seat writes to a room's conversation, with provenance
  `post`, reaching an agent only by recall, endorsement or a trust grant
  (OWN-29). A post from a device seat is trusted for its author's principal's
  own agents, and for agents whose principal's trust grant covers its author;
  every other post is untrusted. A post may carry range links; a **comment**
  is a post on a **marked range**, the address range a range link names. A
  seat's unsent post is a **draft**, ephemeral like a presence hint (PEER-09).
  A **cross-room post** stays in the sending seat's writer; the target room
  shows it, and its seats pull it by address, enveloped (LANE-29).
- **Directed post**: A post addressed to one agent. It waits in that agent's
  principal's Needs you queue for an endorsement (LANE-12).
- **Room summary**: A facilitator's summary of a room, always untrusted,
  linked by address to the events it covers, read only through
  `room_summary_get`, which never writes; a summary request is
  `room_summary_request` (LANE-33).
- **Compaction summary**: The harness's summary at compaction, recorded as
  untrusted `harness_text`, never restored.
- **Envelope**: The wrapper that marks recalled content as untrusted
  historical data; the only way recalled content reaches a model.
- **Envelope warning**: The fixed sentence at the head of every envelope
  (field `warning`).
- **Restore block**: Deterministic trusted text Cairn injects after
  compaction: qualifying pins, a landmark index and a recall hint.
- **Landmark index**: The current run's landmarks, each with its address
  range, as a restore block lists them (INJ-01).
- **Recall hint**: The one fixed line in a restore block saying that the
  recall tools reach the full history (INJ-01).
- **Opt-in notice**: Trusted text built only from Cairn's own ids, version
  numbers, counts, key fingerprints and addresses, saying posts wait or a room
  changed. Sent only while the owner's notice allowance and the agent's
  principal's notice opt-in both stand (INJ-10, LANE-30). The only meaning of
  "notice".
- **Model**: The language model behind an agent, such as Claude; "the model"
  means it wherever this document is not speaking of itself. A **model token**
  is its unit of text, in which budgets are counted.
- **Working view**: Whatever is currently in the model's context window. A
  derived view, never authoritative.
- **Held request**: A permission request, question or hand-off with a stable
  id, answerable from any principal surface (OWN-05), held no longer than its
  **hold window** before its away policy applies. Never shortened to
  "request".
- **Qualified requests**: A permission request (the harness's, held as a held
  request), a role request (a viewer's room act asking for a wider role), a
  join request (a room act of a run's personal-room seat naming the room,
  accepted by its principal), an erasure request (a node sends its purge to
  its peers, PEER-11), a purge request (a neutral principal act asking the
  room's owner to purge what its seats wrote, SEC-30), a quarantine request
  (to peers), and a summary request (to the facilitator). "Request" never
  stands alone.
- **Notification**: A signal to a person: in-page, desktop or terminal on the
  same device, or through the notification bridge to a service the node's
  principal names. It carries only the room's petname, the queue class and a
  count, never room text; it never reaches a model and never accepts answers.

### Seats and keys

- **Seat**: One run or one device in one room. The unit of membership, signing
  and authorship. A run's personal-room seat exists from its first event, and
  a paired phone's device seat there from its pairing, each with no add; every
  other seat keeps its place while its **add** stands: the act that placed it,
  its create room act or a join accepted under the room's admission (LANE-25).
- **Run seat**: A run's seat. Its key lives only in the memory of that run's
  MCP server, which seals its writer (SEC-10); an ingested run's seat key is
  kept like any other non-run seat key.
- **Device seat**: A principal's seat for one device, a node or a paired
  phone, whether the principal is a person or a service account. The one seat
  kind for acting without a run: events that belong to no run go here, and so
  do the principal acts its device signs. A token-key-only node's device seat
  is certified by its token key and signs no principal or expire acts.
- **Seat id**: A seat's id, derived from the room id and the seat's first key.
  Nobody chooses it.
- **Seat key**: A seat's one current key. It signs the seat's room acts and
  seals its writer. A rotation is a signed event in the seat's writer, signed
  by the old key and the new key; the seat keeps its id and writer, and what
  the old key sealed stays verifiable (SEC-27). A key minted because the node
  changed, a clone or a backup restore (REC-24, ADM-06), starts a new seat and
  writer, which names the old one.
- **Device key**: A device's key, certified by a principal key with a **device
  scope** (the principal-act kinds it may sign) and a maximum rule level. It
  signs principal acts and expire acts.
- **Principal key**: A principal's root key, kept offline or in a
  platform-protected key store, which certifies its device keys (PRV-10).
- **Token key**: A key a device key certifies, limited to an access token's
  rooms and expiry, which may stand between a device key and a seat key for an
  ephemeral sandbox (PRV-10). A node with only a token key signs no principal
  acts; every pin it writes restores only once a principal stamps it from one
  of its devices.
- **Seat certificate**: A device key's or token key's signature over a seat
  key, scoped to the seat's room, so every seat key chains to a principal key.
- **Access token**: A short-lived credential a principal mints to enroll a
  device or peer, seat an ephemeral sandbox or carry an invite link (PEER-05,
  PRV-10, LANE-18). "Token" is always an access token, a model token or the
  access token's **token key**.
- **Authenticator**: A hardware-backed key that gives presence proofs
  (OWN-11).
- **CI key**: A key a principal enrolled to sign CI check results.

### Acts and roles

Room state derives from three kinds of act only, and each act belongs to exactly
one kind (LANE-31).

- **Room act**: An act signed by a seat key. It is checked against the
  capabilities of the seat's role and the room state. The room acts are create
  room, join, a join request, leave, a role request, post, link, pin, edit,
  unpin, present, pick, kick, bar, unbar, mute and unmute; set title, labels
  or an assignment; and write a room summary or a summary request. A pin, edit
  or unpin room act never changes any restore block: an author's own pin, edit
  and unpin act only on pins that do not restore unstamped, and a moderator's
  unpin of any other pin only takes it off the room's pin list. A link act
  adds one range link, branch link or criterion link. Create room is the first
  act of the creating seat's writer. Room governance is not I2 trust. So room
  acts never widen what reaches an agent, and OWN-11's classes do not cover
  them.
- **Principal act**: An act signed by a device key at a principal surface, in
  one of three classes (OWN-11).
  - **Cut:** deny, interrupt, pause, stop, cancel a delegation, end a
    permission grant, reject a foreign room, record a `needs changes` verdict,
    quarantine untrusted content, withdraw a risk acceptance, revoke a trust
    grant, end a delegation grant or an acceptance grant, turn an away policy
    off, withdraw a named successor, revoke an appointment, unstamp a pin
    version, turn notices off, decline a handover, withdraw as successor, and
    dismiss a directed post.
  - **Neutral:** mark a room ready or abandoned, acknowledge an overlap,
    record a `met` or `not met` verdict, accept or ask for a join, open the
    forensic view, make a purge request, choose a branch to compare,
    acknowledge counters, and add a room to or remove it from the focus set.
  - **Widening:** allow, answer a hand-off, reply, steer, send a correction,
    retry from a worktree checkpoint, set or revise an intent, resume, record
    a delegation grant or an acceptance grant, add, edit or unpin a
    device-seat pin of a type that restores, confirm a pin candidate, change a
    room's visibility, invite a key, issue an invite link, choose a fork,
    accept a handover or succession, endorse, change a rule level or an away
    policy, quarantine that removes a pin or a trusted event from a restore
    block, release a quarantine, purge or answer an erasure request, export,
    bind a repository identity, accept open residual risks (OWN-22), assign a
    role, set a room's admission, appoint a moderator, stamp a pin version,
    name a successor, hand over a room, record a trust grant, allow notices
    for a room or opt in to them, enable a bridge or the git carrier, set a
    room setting, publish, answer a purge request, accept configuration
    (recording its digest), enroll a CI key, rotate a device key, retire a
    writer, turn capture off or pause ingestion, enroll or revoke a device,
    peer or authenticator, mint or rotate an access token, start a witness
    check, and a backup restore.

  Any principal act that removes a pin from a restore block, or stops this node
  recording its own runs' events, is widening whatever verb carries it, except
  an unstamp or a trust-grant revocation, which withdraws only the acting
  principal's own trust (LANE-32, OWN-29). An unlisted principal act is
  widening. OWN-11 and OWN-12 follow this list, and a gate fails when a
  requirement names a principal act this entry does not classify.
- **Expire act**: An act a node with a device key records, signed with that
  key, ending only an expiry its original act set: on a bar, a mute or a
  handover offer (LANE-25). The setter's node records it first; if that fails,
  a moderator's node; then the owner's node.
- **Role**: A named set of room capabilities: viewer, contributor or
  moderator. The owner gives a seat its role by a **role assignment**. Only
  the facilitator's device seat writes a room summary, beside its appointment.
  - **Viewer:** read, a role request and a summary request.
  - **Contributor:** read and a summary request; post, link and present; pin,
    edit and unpin its own pins; and work.
  - **Moderator:** a contributor's capabilities, plus unpin any pin but the
    intent, kick, bar, unbar, mute, unmute, pick, and set title, labels and
    assignments.
- **Work**: A capability, not an act: a run's events go to its run seat while
  the run works on any branch the room names; an assignment only asks. "Work"
  means nothing else; what one agent hands another is a delegated task.
- **Appointment**: A principal act that makes a run seat or another
  principal's device seat an **appointed moderator**, a moderator within
  SEC-32's limits. The owner may appoint, and so may a principal whose device
  seat has the moderator role by role assignment. The appointer or the owner
  may revoke it. Under those limits it changes no pin and does not mute the
  whole room. It neither unbars nor unmutes. It acts on neither the owner nor
  another moderator, nor outside its room, and it never appoints.
- **Join**: The act that adds a seat to a room under its admission. A run
  joins only when its principal asks for the join or accepts it (LANE-23); a
  device seat joins by a join room act its principal takes at a principal
  surface, without admission when its principal owns the room or has a member
  seat there. A paired phone never joins: its acts go to its device seat in
  the personal room. **Leave** is a seat's room act ending its own add.
- **Retire a writer**: Seal a writer for the last time, by a principal act or
  when the access token behind its seat key expires (PEER-05).
- **Kick**: Revokes a seat's current add. Only that seat's principal may add
  it again (LANE-25).
- **Bar**: Names a principal key and keeps every key that chains to it out,
  until an unbar or an expire act (LANE-25).
- **Mute**: Withdraws every capability but read from one seat or from the
  whole room; a whole-room mute leaves posting to the roles the owner names
  (LANE-16).
- **Present**: Puts a presentation in the outcome window.
- **Pick**: Chooses which presentation the outcome window shows.
- **Admission**: Whether a room is invite only or admits a list of keys. An
  invite (a key and a role) and an invite link are widening principal acts of
  the owner.
- **Successor**: A principal the owner names in advance, who accepts ownership
  once every seat of the owner has left the room. Until a handover, a
  succession or the owner's rejoin, a room whose owner left keeps its pins as
  they were (LANE-11).
- **Handover**: Transfers ownership by an offer and an acceptance. Succession
  is the other path to ownership.
- **Stamp**: A principal's act on one pin version, after being shown its exact
  text, author and key fingerprint, so that version restores word for word to
  that principal's own agents only. A new version needs a new stamp. An edit
  or an unpin leaves a stamped version restoring until its stamper unstamps
  it, a cut principal act (LANE-32).
- **Focus set**: The rooms a principal marks to come first in Needs you,
  changed by a recorded neutral act, so every device shows one order.
- **Active pin**: A pin on its room's pin list, not unpinned (I10). The
  qualifying pins (Pin) include those an unpin leaves restoring.
- **Room merge**: The one rule deriving all room state from the three act
  kinds, in causal order (LANE-31). It covers membership, roles and
  appointments. It covers pins, pin versions and stamps. It covers mutes,
  presentations, picks, bars, handovers and their offers. It covers the
  successor, title, labels, assignment, visibility, admission, the notice
  allowance and the **room settings** (live drafts, typing, the appointment
  rate). When acts conflict, the more restrictive act wins, then the lower
  commitment; concurrent picks resolve by **pick order** (the facilitator's
  seat, then any other moderator, then the owner, VIEW-22), and of two branch
  links naming one branch, the first in causal order stands, concurrent ones
  by the lower commitment (LANE-01).
- **Concurrent**: Said of two acts or events when neither is causally after
  the other.
- **Room state**: Everything the room merge derives (LANE-31).
- **Room status**: A derived view of a room: Running, Quiet, Ready for review
  and the rest of §9.7.2. Never set directly; OWN-21's ready and abandoned
  marks feed it.

### Git and forge

- **Repository**: A git repository, identified by its **repository identity**,
  a bound root commit, the same on every node holding a clone, shallow clones
  included (LANE-02), together with its remotes. A repository may carry Cairn
  data: room trailers in commits (LANE-28), bundles as git refs, and encrypted
  segments in a namespaced location the node's principal enabled for rooms
  whose owner opted in (PEER-08).
- **Branch**: A git branch, identified by repository identity, remote URL and
  branch name. A branch with no remote has a provisional node-local identity,
  rebound when it is pushed.
- **Commit**: A git commit.
- **Worktree**: A git working tree on a node, where a run edits a branch. Not
  a room.
- **Forge**: The service that keeps remotes, pull requests, reviews and branch
  protection. It approves and lands; Cairn does neither, and its reports count
  as `asserted` (LANE-08).
- **Pull request**: The forge's review object for a branch, separate from the
  room.
- **Check**: A command and its exit status, bound to a tree; its **check
  state** is one of §9.7.2's.
- **Result**: What a room's runs established: a check passing or failing on a
  **tree** (git's snapshot of a commit's files), or a stated outcome. It names
  the intent version and carries one evidence class.
- **Evidence**: The checks, attestations or text a result rests on.
- **Evidence class**: Of a result, ranked: `claim` (text only) < `own check` <
  `witness check` < `CI attested` (LANE-05).
- **Own check**: A check the hook handlers recorded on the node of the run
  that made the edits, run on the latest worktree checkpoint plus the recorded
  edits; otherwise it is marked `unbound` and counts as a `claim`.
- **Witness check**: A check re-run through the launcher on a fresh checkout
  of the exact commit, by a node whose principal authored no commit in the
  range (a **commit author** is git's author of a commit, never a seat).
- **CI attested**: A check result for the exact commit, signed by a CI key.
- **Landing**: Git or the forge merging commits into a protected branch. Cairn
  never lands anything, and a landing is never a verdict.
- **Landing link**: The link from a landed commit to a room, carrying a proof
  class, derived by Cairn.
- **Proof class**: Of a landing link. Proven: `same commit`, `same patch`,
  `same tree`. Not proven: `likely`, `asserted`, and `not proven` with a
  reason (LANE-06).
- **Room trailer**: The `Cairn-Room:` line on a commit made on a room's
  branch. Each `Cairn-Link:` line is a trailer link. It counts as `asserted`
  until proven (LANE-28).
- **Qualified links**: Every link is named by what it connects: a branch link
  (room to branch), a pull-request link (branch to its pull request), a
  criterion link (result to criterion), a range link (post or pin to an
  address range), a parent link (subagent run to parent run), a delegation
  link (delegating run to the delegate's run, under a delegation grant), an
  invite link (carrying an access token), a landing link and a trailer link.
  "Link" never stands alone, except as the name of the room act that adds one
  qualified link.
- **Comparison**: A side-by-side view of two branches: each one's exposure,
  results and evidence. It picks no winner.
- **Outcome**: What a room's runs have produced so far: the heads of the
  branches it names, their results and evidence, as verdicts assess them.
- **Presentation**: What a seat put in the outcome window, such as a dev
  server, an artifact, a file or a diff, with the seat and branch.
- **Verdict**: A person's `met`, `not met` or `needs changes` on one
  criterion: a `verdict` pin, recorded by any person with a seat in the room
  as a principal act (OWN-27), bound to the intent version, the heads of every
  branch the room names, and the results and evidence shown. It goes stale
  when any of them changes. Cairn never derives one; service accounts
  contribute evidence instead.

### Trust and flow

- **Trust level**: `trusted` or `untrusted`, per event for one principal's
  agents on one node: derived by the trust policy from the event's provenance
  and writer, the deployment mode recorded with the event, and that
  principal's stamps and trust grants as its writer logs carry them (I10).
  What the principal signed with a device key it certified derives alike on
  all its nodes; this node's unsigned trusted sources are trusted only here.
- **Trusted sources**: What I2 trusts: this node's `operator`, `harness_meta`
  and structural events and its `user` turns while the deployment mode is
  `interactive`, all trusted only on this node; principal acts, posts and pins
  signed through a device key the agent's principal certified, within that
  key's scope; for that agent, the posts and pins a trust grant of its
  principal covers; and a pin version its principal stamped. Everything else
  is untrusted.
- **Deployment mode**: `interactive` (a person types at the harness) or
  `automation` (a pipeline does), set per node (PRV-02,
  `node.deployment_mode`).
- **Trust policy**: The rule that derives each event's trust level (PRV-02).
- **Trust mark**: The sign beside an item saying where it came from and
  whether it is trusted, from §9.7.6's closed set (VIEW-07).
- **Taint**: The trust level a derived artifact inherits: untrusted when any
  event it derives from is untrusted.
- **Recall taint**: A run's mark after it recalls untrusted content, which
  tightens its rule levels (SEC-13).
- **Principal surface**: An authenticated surface for principal acts: the
  browser room view under SEC-20, the CLI at a terminal, or a paired phone
  within its scope.
- **Presence proof**: A hardware-backed, user-verified proof of a person's
  presence, bound to one widening act (OWN-11).
- **Recall**: An agent's tool call that returns enveloped content. Pull-only,
  and it defaults to the agent's current run.
- **Recall scope**: `run`, `room` or `rooms`, or a foreign room named in the
  call (RCL-05). A wider scope is said to extend recall; "widening" belongs to
  acts.
- **Endorsement**: A principal act sending a post's displayed text to one of
  the principal's own agents, inside a fixed template naming the post's author
  and address (OWN-08).
- **Trust grant**: A principal's widening act trusting another principal's key
  for its own agents, in one room or everywhere. It covers that principal's
  device-seat posts and pins only, never run seats, room summaries, nor a
  service account that relays text others wrote (OWN-29); its revocation is
  cut.
- **Delegation**: One agent handing another a delegated task: its subagent,
  or, under a delegation grant, another agent of the same principal, an agent
  on another of its nodes, or another principal's agent (OWN-23 to OWN-26).
- **Delegated task**: The text of a delegation: a subagent's from its harness,
  any other in OWN-24's template.
- **Delegate report**: What a delegate returns, enveloped: through
  `delegation_get`, except a subagent's, which its harness returns (OWN-25).
- **Delegation grant**: The delegating principal's widening act naming who may
  delegate, to which targets, at what rule level, within what budget and until
  when (OWN-23).
- **Acceptance grant**: The receiving principal's widening act on its own
  node, naming the delegating principal, the targets, the maximum rule level,
  the budget and the expiry (OWN-26). Delegation to another principal's agent
  needs both a delegation grant and an acceptance grant.
- **Permission grant**: The grant that lets an approved action pass again when
  the harness next raises it at its permission prompt (OWN-07). Separate from
  trust and delegation grants.
- **Notice allowance, notice opt-in**: The owner's per-room allowance and the
  agent's principal's opt-in. An opt-in notice needs both.
- **Rule level**: For each action class (a kind of tool action, such as edits,
  commands or network use), one of: act without asking, act when told, ask
  first, hand off (OWN-10).
- **Quota**: A limit on storage, events or **spend** (what runs cost in model
  tokens or money), per room, node, writer received from a peer, peer or
  worktree checkpoint, that the node's principal or managed policy sets
  (ADM-15).
- **Away policy**: A principal's opt-in choice of what an unanswered held
  request does while it is on: keep going, pause or stop (OWN-07). Turning one
  on offers to write a head receipt (VIEW-10).
- **Residual risk**: One of the risks §6.1 lists for an unconfined run.
- **Sandbox state**: What confines a run, its policy digest, and which
  residual risks it blocks, recorded from outside the sandbox, where the agent
  cannot write (OWN-22).
- **Risk acceptance**: The principal's act accepting the residual risks a run
  leaves open (OWN-22).
- **Hand-off, hand-back**: An agent passes control to its principal as a held
  request; the principal returns control with a note and a worktree checkpoint
  (OWN-20).
- **Run controls**: Steer, interrupt, pause, resume and stop: principal acts
  on a run.
- **Correction, retry**: After a verdict: principal-typed text in a fixed
  template, or a new run from a worktree checkpoint (OWN-28).
- **Counter**: A count of dropped, rejected, redacted, truncated, coalesced,
  timed-out or failed operations of one kind, each also written to the **audit
  log**, the node's append-only log of Cairn's own operations (OPS-01), shown
  on Health until acknowledged (`cairn counter ack`, I6, OPS-03).
- **Stat**: A count derived from the record within a recall scope, such as
  events, runs, compactions or recalls, read through `stat_list` (RCL-01).
- **Run status**: A run's one status from §9.7.1's closed set (one waiting on
  its principal is Asking, never Needs you), with its **freshness mark**,
  which says how current the run's events are on this node (VIEW-04).
- **Queue class**: One of Needs you's classes Q1 to Q4, which set its order
  (§9.7.4, VIEW-05).
- **Open room**: A room not marked ready or abandoned whose branches have not
  all landed.
- **Exposure**: Of a branch, the untrusted and flagged items its runs read and
  their recall taint (VIEW-13).
- **Overlap**: Runs in two open rooms editing one file, which raises a Needs
  you item on both rooms until the owner acknowledges it (LANE-13).
- **Fork**: Either of two events one writer sealed at one seq, both kept as
  evidence; the node's principal chooses which fork to keep (PEER-10).
- **Presence hint**: An ephemeral sign that a seat is connected, or typing,
  never stored in the record (PEER-09).
- **Watchdog observation**: Cairn's derived note that a run looks stuck, shown
  as the `stuck?` freshness mark; it never starts or resumes a turn (OWN-09).
- **Petname**: A name the person viewing chose for a key or a room, never one
  another principal sent (VIEW-07). Notifications carry a room's petname, else
  its id.

### Components and surfaces

- **Component**: A role Cairn plays, inside exactly one network boundary (I4).
  The set is closed, listed here and in §6.3; how the components ship is open
  (OQ-32), and a new one is a model change and a §6.3 row.
  - **Core (B0):** the hook handlers, each harness adapter's transcript and
    hook part, the MCP server, the kernel worker, the CLI and the TUI, and
    everything that builds what reaches the model.
  - The **harness adapter** is no component: Cairn's code for one harness,
    split between them, its transcript and hook part in the core (it parses,
    opens no socket, starts no process); its run part in the launcher, which
    starts, hosts and controls runs, pauses them at the harness prompt,
    records sandbox state and carries in only text the core built.
  - **Room-view component (B1):** serves the room view on loopback, or on a
    local endpoint only the same OS user can reach.
  - **Launcher (B1):** `cairn launch`, which starts, hosts and controls runs
    through each harness adapter's run part, carrying the core's text into the
    harness input.
  - **Peer component (B2):** replicates segments with peers and serves paired
    phones.
  - **Publish component (B3):** read-only publishing and the git carrier.
  - **Bridge component (B3):** outbound exchange with hosts the node's
    principal names: the forge bridge, the CI carrier and the notification
    bridge.
- **Boundary**: One of B0 core, B1 machine, B2 peer and B3 public (I4).
  Unqualified, "boundary" means a network boundary; any other boundary is
  qualified, such as a span boundary or a crate boundary.
- **Room view**: What shows rooms to a person: the browser, through the
  room-view component, and the TUI, the CLI, the paired phone and the
  **harness strip** (a status line the harness shows) as reduced clients that
  say what they leave out (VIEW-14). A client of the record. Its surfaces are
  a closed set:
  - **Fleet:** every live and recorded run of the principal's rooms, grouped
    by room.
  - **Room page:** one room, with the tabs Timeline, Review and Replay, whose
    **context lens** shows what the model's context window held at an event;
    the verify and why panels, the comparison and the quarantine list, with
    its **forensic view** of quarantined content, open from it, and a foreign
    room opens in it, marked foreign.
  - **Catch up:** the one surface answering "what happened since a starting
    point" the principal picks (VIEW-08).
  - **Needs you:** the one queue of items waiting on a principal (VIEW-05).
  - **Health:** counters, failures and store locations (I6).
  - **Setup:** configuration, every change shown as a diff (I7).
  - **Peers:** enrolled peers and their state.
- **Outcome window**: The pane beside a room's conversation that shows one
  presentation (VIEW-22).

## Relations

- A principal owns any number of rooms and is the principal of every agent its
  nodes start. An agent's principal follows from its node.
- A run has its personal-room seat from its first event, plus a further run
  seat per room it joined or created. Joining is explicit, never automatic.
- Ingest splits one transcript by agent.
- Every seat belongs to one principal and one room; every writer belongs to
  one seat, on one node.
- Each event goes to exactly one seat's writer: the run seat of the room the
  run works in at that moment (a room it joined or created that names its
  current branch), else its personal-room seat. An event with no run goes to
  the principal's device seat. A room act goes to the writer of the seat that
  signs it; a principal act or an expire act, to the device seat of the device
  that signs it, in the room it acts on, which a device of a principal that
  owns the room or has a member seat there first joins without admission. One
  that acts on no room, or on a room its principal has no seat in, goes to
  that device seat in the personal room, naming the room.
- A run's history spans its seats' writers, tied together by the run. Peers
  exchange segments, so a room's seats see only events routed to them.
- Principals and agents create rooms; an agent's room is owned by its
  principal. Cairn never creates a room on its own initiative; a node's
  personal room comes from the principal's `cairn install`, and Cairn may
  suggest other rooms.
- Every pin belongs to exactly one room. A pin with no other room belongs to
  the personal room of its author's principal on the node that wrote it.
- A principal's seats are shown grouped under that principal and, for a
  person, under each device.
- Recall defaults to the agent's current run; it extends only to the
  principal's rooms the agent has a seat in and the cross-room posts they
  show, never to a foreign room unless the call names it.
- Room state is a function of the recorded acts, never of a clock.

## Names follow the model

Code, CLI verbs, MCP tools, configuration keys and events take the name of the
concept they handle. Documentation, UI copy, errors, logs and setup use the
model's words.

- An MCP tool for a room act is `room_<act>`. Every other MCP tool is
  `<concept>_<verb>`, with the reading verbs `get`, `list`, `search` and
  `expand`.
- A CLI command that acts on a concept is `cairn <concept> <verb>`. Node-wide
  utilities keep one verb: `install`, `uninstall`, `status`, `doctor`,
  `verify`, `rebuild`, `migrate`, `ingest`, `import`, `export`, `purge`,
  `audit`, `canary`, `ui`, `why`, `open`, `launch`, `hook`, `mcp` and
  `kernel-worker`.
- Command-line options name concepts: `--author`, `--seat`, `--writer`,
  `--run`, `--room`.
- Configuration keys are `<concept>.<setting>`, or
  `<concept>.<room>.<setting>` per room, the concept singular and in snake
  case (`node.deployment_mode`).
- Layout words (tab, panel, pane, gutter, sheet, stack) name parts of a
  screen, never concepts.
- The attested seat kinds are `run` and `device`; a device seat's principal is
  attested as a person or a service account.
- Cairn defines no slash commands. A harness skill may call MCP tools or the
  CLI; a skill's call is the agent's own tool call, never a principal act.

## Not Cairn concepts

| Term                                                                              | Why                                                                                     | Where it may still appear                                                                                                                                                                                        |
| --------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| session                                                                           | It belongs to the harness; Cairn speaks of runs.                                        | As "harness session", or inside a harness name (the `SessionStart` and `SessionEnd` hooks, the `session_id` field, session-title records, "allow for session" and `allow-session`), when describing the harness. |
| project                                                                           | A room links repositories; nothing is scoped to a project.                              | The harness's `~/.claude/projects` path, the harness settings scope `--scope project`, and Cairn's own software project.                                                                                         |
| tenant                                                                            | Replaced by principal.                                                                  | Nowhere outside history.                                                                                                                                                                                         |
| operator, as a role or a person                                                   | The role is moderator; the person running a node is the node's principal.               | Only as the `operator` provenance class.                                                                                                                                                                         |
| owner act, owner key, owner surface                                               | Replaced by principal act, principal key and principal surface.                         | Nowhere outside history.                                                                                                                                                                                         |
| bot, co-author, actor, poster, bound human                                        | Replaced by service account, pull-request author, author and principal.                 | Nowhere outside history.                                                                                                                                                                                         |
| writer key, writer certificate                                                    | Replaced by seat key and seat certificate.                                              | Nowhere outside history.                                                                                                                                                                                         |
| attempt, claim of work, read only (a seat state)                                  | Replaced by branch, `stake` pin and mute.                                               | Nowhere outside history; "read-only" as an ordinary adjective (read-only publishing) stays.                                                                                                                      |
| run component; own run and witness run as evidence classes                        | Replaced by launcher, own check and witness check.                                      | Nowhere outside history.                                                                                                                                                                                         |
| away mode, room alias, child agent                                                | Replaced by away policy, petname and "a subagent or a delegate".                        | Nowhere outside history.                                                                                                                                                                                         |
| trusted boundary                                                                  | Replaced by trusted sources; "boundary" means a network boundary.                       | Nowhere outside history.                                                                                                                                                                                         |
| room board                                                                        | Replaced by conversation.                                                               | Nowhere outside history.                                                                                                                                                                                         |
| participant, player                                                               | Replaced by seat and member.                                                            | Nowhere outside history.                                                                                                                                                                                         |
| lane                                                                              | Renamed to room.                                                                        | The LANE and VIEW requirement ids and file names.                                                                                                                                                                |
| judge, approval gate                                                              | Removed by the stakeholder; a person records a verdict.                                 | Nowhere outside history.                                                                                                                                                                                         |
| service-account seat                                                              | Replaced by device seat, the one seat kind for acting without a run.                    | Nowhere outside history.                                                                                                                                                                                         |
| import of a transcript or a peer's segments; imported run                         | Only a bundle is imported; a transcript is ingested and a peer's segments are received. | Nowhere outside history.                                                                                                                                                                                         |
| presence check; unqualified token; Needs you as a run or room status; work marker | Replaced by presence proof, access token or model token, Asking, and ingest marker.     | Nowhere outside history.                                                                                                                                                                                         |
| hide                                                                              | Removed by the stakeholder; text the UI does not show is invisible.                     | The cryptographic term "hiding commitment" (REC-17).                                                                                                                                                             |

"History" means historical records. They are the SRS change log, accepted ADRs
and plan records, which keep the words of their time. Excluded words may also
appear in files an outside tool writes and maintains, in that tool's meaning,
such as frit's plan skills and the plan files in frit's format; the domain-model
agent skips them.

## Changing the model

A new concept, a renamed one or a new relation is a stakeholder decision. It
lands here before anything uses it. The domain-model agent reviews every change
to this document and every proposal to change it. It names the invariants whose
wording would change, and every use the change makes stale or that runs ahead.
