---
summary: >-
  Cairn's domain model: the closed set of concepts with their
  definitions, how they relate, the terms that are not Cairn concepts,
  and how names in code, docs and UI follow the model. The SRS links
  here for every term, and the domain-model agent reviews against it.
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

Every word has one meaning. Where everyday English uses one word for
several things, each sense gets its own qualified name: "link" is
always a branch link, a range link or another named kind, and
"request" is always a held request, a role request or another named
kind.

These verbs each have one job:

- A principal *owns* a room.
- A seat *is a member of* a room while its add stands.
- A principal *has a seat in* a room through any of its seats, including its
  agents' run seats.
- A node *holds* writer logs and rooms. "Owns" is said only of rooms and
  "holds" only of nodes; "holds a room" never means owning a room or being a
  member of it.

## Concepts

### Principals and agents

- **Principal**: A person or a service account. It has a principal key, signs
  principal acts at a principal surface through a device seat, and is the
  principal of every agent its nodes start. Cairn counts principals by
  principal key: every device key, token key and seat key that chains to one
  principal key belongs to that principal. A service account its **certifier**
  (another service account, or managed policy) certified is still its own
  principal, counted by its own principal key. Only an agent's own principal
  widens what reaches that agent (I2).
- **Person**: A human principal. No one certifies a person's principal key.
  Only a person records a verdict.
- **Service account**: A non-human principal with its own principal key,
  certified, and revocable, by another service account or by managed policy.
  It can own rooms, have seats and be the principal of its own agents, such as
  CI or runner agents.
- **Managed policy**: Root-owned configuration an organisation sets on a
  machine. Cairn never overrides it (I7). It may turn off boundaries B1 to B3,
  forbid risk acceptance and certify service accounts. The only authority that
  is not a principal.
- **Agent**: A worker a harness runs for exactly one principal: the principal
  of the node that started it (OWN-01). It receives restore blocks and recalls
  history; an agent is never a principal.
- **Run**: One agent run as the harness reports it, on one node. It has run
  seats and carries its recall taint (SEC-13), sandbox state (OWN-22), seat
  keys (SEC-10), kernel namespace (CMP-02) and spans (LMK-01). "Run" has no
  other meaning.
- **Subagent**: An agent another agent's harness started for it. It has its
  own run, tied to its parent's run by a parent link, with its own seats and
  writers. It takes work without a delegation grant.
- **Ingested run**: A run ingested from a transcript the hook handlers did not
  watch, with origin `ingested`, and untrusted (REC-22). A principal's own
  transcripts from outside `transcript.roots` land in its personal room.
- **Facilitator**: A service account whose device seat in a room is an
  appointed moderator (SEC-32). It may post findings in its own words and
  write room summaries, answering summary requests (LANE-33). Its posts reach
  an agent as trusted text only through a trust grant.
- **Author**: The seat that wrote an event or a pin. The author's principal
  follows from the seat.
- **Member**: A seat whose add stands, or a run's personal-room seat; the
  seat's role says what it may do. A kicked, departed or barred seat stays a
  seat but is no longer a member. A principal is never a member: it has a seat
  in the room through any of its seats, and text for people speaks of the
  room's principals.
- **Owner**: The one principal who owns a room: its intent, roles, admission,
  appointments, successor and handover. Ownership changes only by handover or
  succession (LANE-11). The owner stands beside the roles rather than having
  one. "Owner" means nothing else.
- **Pull-request author**: The outside party whose commits a foreign room's
  bundle describes, matched through their commit-signing identity (LANE-15).
  May have no seat at all.
- **Delegate**: The agent that receives a delegation (OWN-24). It keeps its
  own principal.

### Harness facts

The harness's own facts, which Cairn records and names but never
keeps or controls.

- **Harness**: The external program that runs agents, such as Claude Code or
  the Agent SDK. It keeps their transcripts, compacts their context and
  reports to Cairn through hooks. Cairn records it and never manages it.
- **Harness session**: The harness's own unit of work, which yields one
  transcript. Named only when describing the harness, never as a Cairn unit.
- **Transcript**: The harness's file of what a harness session did. A
  **transcript source** is a transcript Cairn ingests. "Source" never stands
  alone: it is a transcript source or a trusted source.
- **Hook**: The harness's callback into Cairn, carrying a JSON payload (§9.1).
  Cairn's hook handlers, in the core, answer it.
- **User turn**: A message the person types to an agent through the harness.
  In interactive deployment mode it is a trusted source (I2, PRV-02).
- **Harness adapter**: Cairn's code for one harness. Its transcript and hook
  part sits in the core: it parses, opens no socket and starts no process. Its
  run part starts, hosts and controls runs, holds them at the harness prompt
  and records sandbox state, inside the launcher (B1).
- **Compaction**: The harness replacing earlier context with a compaction
  summary when the context window fills. Cairn neither performs nor controls
  it (NG1); it records it and restores pins after it (I3).

### Places

- **Home**: The directory holding all of one principal's Cairn state on a node
  (`CAIRN_HOME`, default `~/.cairn`), belonging to one OS user and optionally
  bound to a home id the runner environment supplies (SEC-03). The unit of
  isolation (I8).
- **Node**: One home on one machine, container or sandbox, for one principal.
  Its device key signs what it records; a node in an ephemeral sandbox may
  have only a token key. The node's principal is the principal whose home it
  is.
- **Device**: Anything with a device key: a node, or a paired phone.
- **Paired phone**: A device with no home, limited to reading and to allowing
  or denying held permission requests within its scope. It signs with its own
  device key, and the node it pairs with holds and seals its device seat's
  writer.
- **Sandbox**: A confinement around a run that blocks some residual risks
  (OWN-22). A node running inside a sandbox is still a node.
- **Room**: Where an intent is worked on: at most one intent, a conversation,
  seats, pins, and branches in any repositories, each named by a branch link.
  The unit Cairn records, shows and shares. Principals and agents create
  rooms; Cairn never creates a room on its own initiative.
- **Conversation**: A room's ordered posts.
- **Personal room**: A principal's private room on each of its nodes, created
  by the principal's `cairn install` as the first act of its device seat
  there. Every run sits in it from its first event, without a join, and every
  event or pin that belongs to no other room goes there.
- **Foreign room**: A room this node holds that its principal neither owns nor
  has a seat in, such as the room of an imported bundle or a peer's room.
  Untrusted, and outside every widened recall scope unless named in the call.
  A room the principal owns or has a seat in is never foreign.
- **Principal's rooms**: The rooms a principal owns or has a seat in.
- **Visibility**: Whether a room is private, shared with members, published or
  stored on blind peers (LANE-17). Changing it is a widening principal act of
  the owner.
- **Peer**: Another node this node enrolled by key and exchanges sealed
  segments with through the peer component (B2). Never trusted for content.
- **Blind peer**: A peer that holds a room's segments without the key of any
  seat in the room, so it stores and serves them encrypted and reads none of
  them (PEER-12).

### Record

- **Record**: The set of writer logs a node holds. The source of truth;
  everything else is derived from it (I10).
- **Writer**: One seat's append-only log on one node, named by the seat's
  first key. Its events are hash-chained; the **chain head** at a seq is the
  hash over every event up to it.
- **Event**: One immutable entry in a writer: a message, tool call, tool
  result, hook observation, or an act (a room act, a principal act or an
  expire act).
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
- **Segment**: A closed, signed range of one writer, sealed as a unit; what
  peers exchange. A unit of the record, not a storage layout.
- **Seal**: A seat key's signature over its writer id, a seq and the chain
  head at that seq (REC-18). Events after the newest seal are unsigned.
- **Commitment**: A keyed commitment to an event's content under a per-event
  random key kept with the content and erased with it; the only way the chain,
  seals and tombstones refer to content (REC-17).
- **Provenance**: An event's class from a closed set, saying what produced its
  content (§5.2). `operator` is the class of the events this node's principal
  writes through a principal surface.
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
- **Worktree checkpoint**: An event recording a worktree's commit, branch and
  redacted diff since the previous worktree checkpoint (REC-20). Always
  written in full, so it is never confused with the harness's rewind points.
- **Derived artifact**: Anything computed from the record (I10): indexes,
  landmarks, active pins, the quarantine set, statuses, queues, evidence and
  proof classes, stats, and room summaries' range links.
- **Redaction**: Removing secrets from content before it is stored or on
  import, recorded (I1, SEC-08).
- **Retention policy**: A rule, set by the node's principal or managed policy,
  that purges content of a room and provenance after a time (I1, SEC-22).
- **Flag**: A mark Cairn sets on an event whose text matches an injection
  pattern (PRV-07); flagged text contributes only counts.
- **Quarantine**: Recorded, reversible exclusion of an event, span, run,
  writer or derived artifact from recall and from restore blocks, without
  deletion (I5).
- **Purge**: Deletion of content, leaving a tombstone (ADM-07). The only act
  that destroys content (I1).
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
- **Bundle**: A signed, reviewed export of a room, carried as a file or a git
  ref.
- **Import**: Reading a bundle into this node's record (`cairn import`,
  REC-23). A transcript is ingested and a peer's segments are received;
  neither is imported.
- **Kernel**: The hermetic compute environment in which Claude runs code over
  the record (CMP-01).

### Pins and context

- **Pin**: Verbatim text in exactly one room, with one author (a seat), a pin
  type and numbered pin versions. Information, never an instruction. It
  restores only under PIN-10, as a **qualifying pin**: a pin written from a
  device seat restores to its author's principal's agents, and to agents whose
  principal's trust grant covers its author; any version restores to the
  agents of a principal who stamped it. Adding, editing or unpinning a
  device-seat pin is a widening principal act of its author's principal; a run
  seat's pin, which restores only once stamped, is changed by room acts. A
  room act never changes any restore block (LANE-32).
- **Pin version**: One immutable text of a pin; each edit adds one, unstamped.
  What a stamp covers.
- **Pin candidate**: An inactive pin an agent proposed or Cairn detected. It
  becomes active only through its principal's confirmation, which makes a pin
  the principal's device seat authors, or a stamp. A pin an agent's run seat
  wrote becomes active only by a stamp (LANE-32).
- **Pin type**: One of `constraint`, `preference`, `decision`, `fact`,
  `episode`, `intent`, `verdict` and `stake`. Only `constraint`, `preference`
  and `intent` pins restore.
- **Unpin**: The act that ends a pin: it takes the pin off the room's pin
  list, and its versions stay in the record (I1). It stops the pin restoring,
  with two exceptions, each raising a Needs you item: a stamped version keeps
  restoring to its stamper's agents until the stamper unstamps it, and after a
  moderator's unpin a device-seat pin keeps restoring to its author's
  principal's agents until that principal unpins it.
- **Intent**: A room's lead pin, of type `intent`: a goal, its criteria and
  optionally the paths it is meant to change (LANE-20). Only the owner's
  principal act changes it.
- **Criterion**: One acceptance condition of an intent, with a stable id.
- **Stake**: A pin of type `stake` stating what its author works on. Only its
  author writes it, and it locks nothing.
- **Title, labels**: Room state the owner chooses: a display name and tags.
  They never reach a model.
- **Assignment**: Room state asking a seat to work on a branch. It locks
  nothing.
- **Post**: A message a seat writes to a room's conversation, with provenance
  `post`, untrusted unless a trust grant of the agent's principal covers its
  author. A post may carry range links; a **comment** is a post on a marked
  range. A **cross-room post** stays in the sending seat's writer; the target
  room shows it, and its seats pull it by address, enveloped (LANE-29).
- **Directed post**: A post addressed to one agent. It waits in that agent's
  principal's Needs you queue for an endorsement (LANE-12).
- **Room summary**: A facilitator's summary of a room, linked by address to
  the events it covers, read only through `room_summary_get`, which never
  writes; a summary request is `room_summary_request` (LANE-33).
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
- **Working view**: Whatever is currently in the model's context window. A
  projection, never the source of truth.
- **Held request**: A permission request, question or hand-off with a stable
  id, answerable from any principal surface (OWN-05). Never shortened to
  "request".
- **Qualified requests**: A permission request (the harness's, held as a held
  request), a role request (a viewer asks for a wider role), a join request
  (for a run, asked or accepted by its principal), an erasure request (a node
  sends its purge to its peers, PEER-11), a purge request (a principal with a
  seat asks the room's owner to purge what its seats wrote, SEC-30), a
  quarantine request (to peers), and a summary request (to the facilitator).
  "Request" never stands alone.
- **Notification**: A signal to a person: in-page, desktop or terminal on the
  same device, or through the notification bridge to a service the node's
  principal names. It carries only the room's petname, the queue class and a
  count, never room text; it never reaches a model and never accepts answers.

### Seats and keys

- **Seat**: One run or one device in one room. The unit of membership, signing
  and authorship. A run's personal-room seat exists from its first event, with
  no add; every other seat keeps its place while its **add** stands: the join
  that placed it, accepted under the room's admission (LANE-25).
- **Run seat**: A run's seat. Its key lives only in the memory of that run's
  MCP server (SEC-10).
- **Device seat**: A principal's seat for one device, a node or a paired
  phone, whether the principal is a person or a service account. The one seat
  kind for acting without a run: events that belong to no run go here, and so
  do the principal acts its device signs.
- **Seat id**: A seat's id, derived from the room id and the seat's first key.
  Nobody chooses it.
- **Seat key**: A seat's one current key. It signs the seat's room acts and
  seals its writer. A rotation is a signed event in the seat's writer, signed
  by the old key and the new key; the seat keeps its id and writer, and what
  the old key sealed stays verifiable (SEC-27). A key minted because the node
  changed, a clone or a restore (REC-24, ADM-06), starts a new seat and
  writer, linked to the old one.
- **Device key**: A device's key, certified by a principal key with a scope
  and a maximum rule level. It signs principal acts and expire acts.
- **Principal key**: A principal's root key, kept offline or in a
  platform-protected key store, which certifies its device keys (PRV-10).
- **Token key**: A key a device key certifies, limited to a token's rooms and
  expiry, which may stand between a device key and a seat key for an ephemeral
  sandbox (PRV-10). A node with only a token key signs no principal acts; its
  runs' pins restore only once a principal stamps them from one of its
  devices.
- **Seat certificate**: A device key's or token key's signature over a seat
  key, scoped to the seat's room, so every seat key chains to a principal key.
- **Token**: A short-lived credential a principal mints to enroll a device or
  peer, or to seat an ephemeral sandbox (PEER-05, PRV-10).
- **Authenticator**: A hardware-backed key that gives presence checks
  (OWN-11).
- **CI key**: A key a principal enrolled to sign CI check results.

### Acts and roles

Room state derives from three kinds of act only, and each act belongs
to exactly one kind (LANE-31).

- **Room act**: An act signed by a seat key. It is checked against the
  capabilities of the seat's role. The room acts are create room, join, leave,
  post, link, pin, edit, unpin, present, pick, kick, bar, unbar, mute and
  unmute; set title, labels or an assignment; and write a room summary or a
  summary request. A pin, edit or unpin room act never changes any restore
  block: an author's own pin, edit and unpin act only on run seats' pins, and
  a moderator's unpin of any other pin only takes it off the room's pin list.
  A link act adds one qualified link. Create room is the first act of the
  creating seat's writer. Room governance is not I2 trust. So room acts never
  widen what reaches an agent, and OWN-11's classes do not cover them.
- **Principal act**: An act signed by a device key at a principal surface, in
  one of three classes (OWN-11).
  - **Cut:** deny, interrupt, pause, stop, cancel a delegation, end a grant,
    reject a foreign room, record a `needs changes` verdict, quarantine
    untrusted content, withdraw a risk acceptance, revoke a trust grant,
    revoke an appointment, unstamp a pin version, turn notices off, decline a
    handover, withdraw as successor, and dismiss a directed post.
  - **Neutral:** mark a room ready or abandoned, acknowledge an overlap,
    record a `met` or `not met` verdict, link a result to a criterion, accept
    a join, choose a branch to compare, acknowledge counters, and add a room
    to or remove it from the focus set.
  - **Widening:** allow, answer a hand-off, reply, steer, send a correction,
    retry from a worktree checkpoint, set or revise an intent, resume, record
    a delegation grant or an acceptance grant, add, edit or unpin a
    device-seat pin, confirm a pin candidate, change a room's visibility,
    invite a key, issue an invite link, choose a fork, accept a handover or
    succession, endorse, change a rule level or an away policy, quarantine
    that removes a pin or a trusted event from a restore block, release a
    quarantine, purge or answer an erasure request, export, bind a repository
    identity, accept open residual risks (OWN-22), assign a role, set a room's
    admission, appoint a moderator, stamp a pin version, name a successor,
    hand over a room, record a trust grant, allow notices for a room or opt in
    to them, enable a bridge, retire a writer, turn capture off or pause
    ingestion, enroll or revoke a device, peer or authenticator, mint or
    rotate a token, start a witness check, and restore a backup.

  Any principal act that removes a pin from a restore block, or stops this
  node recording its own runs' events, is widening whatever verb carries it,
  except an unstamp, which withdraws only the unstamper's own trust (LANE-32).
  An unlisted principal act is widening. OWN-11 and OWN-12 follow this list,
  and a gate fails when a requirement names a principal act this entry does
  not classify.
- **Expire act**: An act a node records, signed with its device key, ending
  only an expiry its original act set: on a bar, a mute or a handover offer
  (LANE-25). The setter's node records it first; if that fails, a moderator's
  node; then the owner's node.
- **Role**: A named set of room capabilities: viewer, contributor or
  moderator. The owner gives a seat its role by a **role assignment**. Only
  the facilitator's device seat writes a room summary, beside its moderator
  role.
  - **Viewer:** read, a role request and a summary request.
  - **Contributor:** read and a summary request; post, link (a branch link
    included) and present; pin, edit and unpin its own pins; work on the
    room's branches it is given.
  - **Moderator:** a contributor's capabilities, plus unpin any pin but the
    intent, kick, bar, unbar, mute, unmute, pick, and set title, labels and
    assignments.
- **Work**: A capability, not an act: a run's events go to its run seat while
  the run works on one of the room's branches.
- **Appointment**: A principal act that makes a run seat or another
  principal's device seat a moderator. The owner may appoint, and so may a
  principal whose device seat has the moderator role by role assignment. The
  appointer or the owner may revoke it. An appointed moderator stays within
  SEC-32's limits. It changes no pin and does not mute the whole room. It
  neither unbars nor unmutes. It acts on neither the owner nor another
  moderator, nor outside its room, and it never appoints.
- **Join**: A run joins a room only when its principal asks for the join or
  accepts it (LANE-23).
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
  after the owner leaves.
- **Handover**: Transfers ownership by an offer and an acceptance. Succession
  is the other path to ownership.
- **Stamp**: A principal's act on one pin version, after being shown its exact
  text, author and key fingerprint, so that version restores word for word to
  that principal's own agents only. A new version needs a new stamp. An edit
  or an unpin leaves a stamped version restoring until its stamper unstamps
  it, a cut principal act (LANE-32).
- **Focus set**: The rooms a principal marks to come first in Needs you,
  changed by a recorded neutral act, so every device shows one order.
- **Room merge**: The one rule deriving all room state from the three act
  kinds, in causal order (LANE-31). It covers membership, roles, appointments,
  pins and their versions, and stamps. It also covers mutes, presentations,
  picks, bars, handover offers, handovers, title, labels and assignment. When
  acts conflict, the more restrictive act wins, then the lower commitment;
  concurrent picks resolve by **holder order** (facilitator, then moderator,
  then owner, VIEW-22), and of two branch links naming one branch, the first
  in causal order stands (LANE-01).
- **Concurrent**: Said of two acts or events when neither is causally after
  the other. Defined without any clock.
- **Room state**: Everything the room merge derives (LANE-31). It is never set
  directly.
- **Room status**: A derived view of a room: Running, Quiet, Ready for review
  and the rest of §9.7.2. Not state: no act sets it directly, though OWN-21's
  ready and abandoned marks feed it.

### Git and forge

- **Repository**: A git repository, identified by a bound root commit, the
  same on every node holding a clone, shallow clones included (LANE-02),
  together with its remotes. A repository may carry Cairn data: room trailers
  in commits (LANE-28), bundles as git refs, and encrypted segments in a
  namespaced location the node's principal enabled (PEER-08).
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
- **Check**: A command and its exit status, bound to a tree.
- **Result**: An outcome of a room's work: a check passing or failing on a
  tree, or a stated outcome. It names the intent version and carries one
  evidence class.
- **Evidence**: The checks, attestations or text a result rests on.
- **Evidence class**: Of a result, ranked: `claim` (text only) < `own check` <
  `witness check` < `CI attested` (LANE-05).
- **Own check**: A check a hook recorded on the room's own node, run on the
  latest worktree checkpoint plus the recorded edits; otherwise it is marked
  `unbound` and counts as a `claim`.
- **Witness check**: A check re-run through the launcher on a fresh checkout
  of the exact commit, by a node whose principal authored no change in the
  range.
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
  link, an invite link, a landing link and a trailer link. "Link" never stands
  alone, except as the name of the room act that adds one qualified link.
- **Comparison**: A side-by-side view of two branches: their exposure, outcome
  and evidence. It picks no winner.
- **Outcome**: What a room's work has produced so far: the heads of the
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
  agents: derived by the trust policy from the event's provenance and writer,
  the deployment mode recorded with the event, and that principal's stamps and
  trust grants as its writer logs carry them. Every node of that principal
  holding the same logs derives the same level (I10).
- **Trusted sources**: What I2 trusts: this node's `operator`, `harness_meta`
  and structural events; its `user` turns while the deployment mode is
  `interactive`; principal acts signed by a device key the agent's principal
  certified, within that key's scope; for that agent, the posts and pins a
  trust grant of its principal covers; and a pin version its principal
  stamped. Everything else is untrusted.
- **Deployment mode**: `interactive` (a person types at the harness) or
  `automation` (a pipeline does), set per node (PRV-02, `deployment.mode`).
- **Trust policy**: The rule that derives each event's trust level (PRV-02).
- **Trust mark**: The sign beside an item saying where it came from and
  whether it is trusted, from §9.7.6's closed set (VIEW-07).
- **Taint**: The trust level a derived artifact inherits: untrusted when any
  event it derives from is untrusted.
- **Recall taint**: A run's mark after it recalls untrusted content, which
  tightens its rule levels (SEC-13).
- **Principal surface**: An authenticated surface for principal acts: the room
  view under SEC-20, the CLI at a terminal, or a paired phone within its
  scope.
- **Presence check**: A hardware-backed, user-verified proof of a person's
  presence, bound to one widening act (OWN-11).
- **Recall**: An agent's tool call that returns enveloped content. Pull-only,
  and it defaults to the agent's current run.
- **Recall scope**: `run`, `room` or `rooms`, or a foreign room named in the
  call (RCL-05).
- **Endorsement**: A principal act sending a post's displayed text to one of
  the principal's own agents, inside a fixed template naming the post's author
  and address (OWN-08).
- **Trust grant**: A principal's widening act trusting another principal's key
  for its own agents, in one room or everywhere. It covers that principal's
  device-seat posts and pins only, never run seats, nor a service account that
  relays text others wrote (OWN-29).
- **Delegation**: Work one agent hands another: its subagent, or, under a
  delegation grant, another run of the same principal, an agent on another of
  its nodes, or another principal's agent (OWN-23 to OWN-26).
- **Delegated task**: The text of a delegation, delivered in OWN-24's
  template.
- **Delegate report**: What a delegate returns, read through `delegation_get`,
  enveloped.
- **Delegation grant**: The delegating principal's widening act naming who may
  delegate, to which targets, at what rule level, within what budget and until
  when (OWN-23).
- **Acceptance grant**: The receiving principal's widening act on its own
  node, naming the delegating principal, the targets, the maximum rule level,
  the budget and the expiry (OWN-26). Delegation to another principal's agent
  needs both a delegation grant and an acceptance grant.
- **Permission grant**: The grant that lets an approved, parked action pass
  again (OWN-07). Separate from trust and delegation grants.
- **Notice allowance, notice opt-in**: The owner's per-room allowance and the
  agent's principal's opt-in. An opt-in notice needs both.
- **Rule level**: For each action class (a kind of tool action, such as edits,
  commands or network use), one of: act without asking, act when told, ask
  first, hand off (OWN-10).
- **Quota**: A limit on storage, events or spend, per room, node, writer
  received from a peer, peer or worktree checkpoint, that the node's principal
  or managed policy sets (ADM-15).
- **Away policy**: A principal's opt-in choice of what an unanswered held
  request does while it is on: keep going, pause or stop (OWN-07). Turning one
  on offers to write a head receipt (VIEW-10).
- **Residual risk**: One of the risks §6.1 lists for an unconfined run.
- **Sandbox state**: What confines a run, its policy digest, and which
  residual risks it blocks, recorded from a source the agent cannot write
  (OWN-22).
- **Risk acceptance**: The principal's act accepting the residual risks a run
  leaves open (OWN-22).
- **Hand-off, hand-back**: An agent passes control to its principal as a held
  request; the principal returns control with a note and a worktree checkpoint
  (OWN-20).
- **Run controls**: Steer, interrupt, pause, resume and stop: principal acts
  on a run.
- **Correction, retry**: After a verdict: principal-typed text in a fixed
  template, or a new run from a worktree checkpoint (OWN-28).
- **Counter**: A count of dropped, rejected, redacted, timed-out or failed
  operations of one kind, shown on Health until acknowledged (`cairn counter
  ack`, I6, OPS-03).
- **Stat**: A count derived from the record within a recall scope, such as
  events, runs, compactions or recalls, read through `stat_list` (RCL-01).
- **Run status**: A run's one status from §9.7.1's closed set, with its
  **freshness mark**, which says how current the run's events are on this node
  (VIEW-04).
- **Queue class**: One of Needs you's classes Q1 to Q4, which set its order
  (§9.7.4, VIEW-05).
- **Overlap**: Two open rooms' work editing one file, which raises a Needs you
  item on both rooms until the owner acknowledges it (LANE-13).
- **Fork**: The two events one writer sealed at one seq, kept as evidence; the
  node's principal chooses which to keep (PEER-10).
- **Marked range**: An address range a seat marks so posts and pins can point
  to it through range links.
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
  - **Room-view component (B1):** serves the room view on loopback.
  - **Launcher (B1):** `cairn launch`, which starts, hosts and controls runs
    through each harness adapter's run part.
  - **Peer component (B2):** replicates segments with peers.
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
  say what they leave out (VIEW-14). A client of the record, never its source.
  Its surfaces are a closed set:
  - **Fleet:** every live and recorded run of the principal's rooms, grouped
    by room.
  - **Room page:** one room, with the tabs Timeline, Review and Replay, whose
    **context lens** shows what the model's context window held at an event;
    the verify and why panels, the comparison and the quarantine list open
    from it, and a foreign room opens in it, marked foreign.
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
- A run has its personal-room seat from its first event, plus one run seat per
  room it joined. Joining is explicit, never automatic.
- A subagent's run is tied to its parent's run by a parent link. Ingest splits
  one transcript by agent.
- Every seat belongs to one principal and one room; every writer belongs to
  one seat, on one node.
- Each event goes to exactly one seat's writer: the run seat of the room the
  run works in at that moment (a room it joined that names its current
  branch), else its personal-room seat. An event with no run goes to the
  principal's device seat. A room act goes to the writer of the seat that
  signs it; a principal act or an expire act, to the device seat of the device
  that signs it, in the room it acts on. One that acts on no room, or on a
  room where its principal has no seat, goes to that device seat in the
  personal room, naming the room.
- A run's history spans its seats' writers, joined through the run. Peers
  exchange writer logs, so a room's seats see only work routed to them.
- Principals and agents create rooms; an agent's room is owned by its
  principal. Cairn never creates a room on its own initiative; a node's
  personal room comes from the principal's `cairn install`, and Cairn may
  suggest other rooms.
- Every pin belongs to exactly one room. A pin with no other room belongs to
  the personal room of its author's principal on the node that wrote it.
- A principal's seats are shown grouped under that principal and, for a
  person, under each device.
- Only a principal widens what reaches its own agents; an agent never does,
  and room acts never do.
- A cross-room post belongs to the sending seat's room; the target room shows
  it by address and never copies it into a writer of its own.
- A room names branches in any number of repositories through branch links; a
  branch with a pull request links to it.
- Recall defaults to the agent's current run; it widens only to the
  principal's rooms the agent has a seat in, never to a foreign room unless
  the call names it.
- Room state is a function of the recorded acts, never of a clock.

## Names follow the model

A function, type, module, crate, CLI verb, MCP tool, configuration key
or event is named after the concept it handles. Documentation, UX and
UI copy, error and help text, logs and developer setup use the same
words the SRS uses.

- An MCP tool for a room act is `room_<act>`. Every other MCP tool is
  `<concept>_<verb>`, with the reading verbs `get`, `list`, `search` and
  `expand`.
- A CLI command that acts on a concept is `cairn <concept> <verb>`. Node-wide
  utilities keep one verb: `install`, `uninstall`, `status`, `doctor`,
  `verify`, `rebuild`, `backup`, `restore`, `migrate`, `ingest`, `import`,
  `export`, `purge`, `audit`, `canary`, `ui`, `why`, `open`, `launch`, `hook`,
  `mcp` and `kernel-worker`.
- Flags name concepts: `--author`, `--seat`, `--writer`, `--run`, `--room`.
- Configuration keys are `<concept>.<setting>`, the concept singular.
- The attested seat kinds are `run` and `device`; a device seat's principal is
  attested as a person or a service account.
- Cairn defines no slash commands. A harness skill may call MCP tools or the
  CLI; a skill's call is the agent's own act, never a principal act.

## Not Cairn concepts

| Term                                                       | Why                                                                                     | Where it may still appear                                                                                                                                                                                        |
| ---------------------------------------------------------- | --------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| session                                                    | It belongs to the harness; Cairn speaks of runs.                                        | As "harness session", or inside a harness name (the `SessionStart` and `SessionEnd` hooks, the `session_id` field, session-title records, "allow for session" and `allow-session`), when describing the harness. |
| project                                                    | A room links repositories; nothing is scoped to a project.                              | The harness's `~/.claude/projects` path, the harness settings scope `--scope project`, and Cairn's own software project.                                                                                         |
| tenant                                                     | Replaced by principal.                                                                  | Nowhere outside history.                                                                                                                                                                                         |
| operator, as a role or a person                            | The role is moderator; the person running a node is the node's principal.               | Only as the `operator` provenance class.                                                                                                                                                                         |
| owner act, owner key, owner surface                        | Replaced by principal act, principal key and principal surface.                         | Nowhere outside history.                                                                                                                                                                                         |
| bot, co-author, actor, poster, bound human                 | Replaced by service account, pull-request author, author and principal.                 | Nowhere outside history.                                                                                                                                                                                         |
| writer key, writer certificate                             | Replaced by seat key and seat certificate.                                              | Nowhere outside history.                                                                                                                                                                                         |
| attempt, claim of work, read only (a seat state)           | Replaced by branch, `stake` pin and mute.                                               | Nowhere outside history; "read-only" as an ordinary adjective (read-only publishing) stays.                                                                                                                      |
| run component; own run and witness run as evidence classes | Replaced by launcher, own check and witness check.                                      | Nowhere outside history.                                                                                                                                                                                         |
| away mode, room alias, child agent                         | Replaced by away policy, petname and "a subagent or a delegate".                        | Nowhere outside history.                                                                                                                                                                                         |
| trusted boundary                                           | Replaced by trusted sources; "boundary" means a network boundary.                       | Nowhere outside history.                                                                                                                                                                                         |
| room board                                                 | Replaced by conversation.                                                               | Nowhere outside history.                                                                                                                                                                                         |
| participant, player                                        | Replaced by seat and member.                                                            | Nowhere outside history.                                                                                                                                                                                         |
| lane                                                       | Renamed to room.                                                                        | The LANE and VIEW requirement ids and file names.                                                                                                                                                                |
| judge, approval gate                                       | Removed by the stakeholder; a person records a verdict.                                 | Nowhere outside history.                                                                                                                                                                                         |
| service-account seat                                       | Replaced by device seat, the one seat kind for acting without a run.                    | Nowhere outside history.                                                                                                                                                                                         |
| import of a transcript or a peer's segments; imported run  | Only a bundle is imported; a transcript is ingested and a peer's segments are received. | Nowhere outside history.                                                                                                                                                                                         |
| hide                                                       | Removed by the stakeholder.                                                             | Nowhere outside history.                                                                                                                                                                                         |

"History" is the SRS change log, accepted ADRs and plan records, which
keep the words of their time. Excluded words may also appear in files
an outside tool writes and maintains, in that tool's meaning, such as
frit's plan skills and the plan files in frit's format; the
domain-model agent skips them.

## Changing the model

A new concept, a renamed one or a new relation is a stakeholder
decision. It lands in this document before any requirement, name or
screen uses it. The domain-model agent reviews every change to this
document and every proposal to change it: it checks the model against
itself and every definition elsewhere against the model, names the
invariants whose wording would change, and lists every use the change
makes stale. It also reports every use that runs ahead of the model.
