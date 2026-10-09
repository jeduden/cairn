---
title: "Record"
order: "04"
summary: >-
  The record and what derives from it: writers, events, addresses, segments, seals, provenance and origin, spans and landmarks, quarantine and purge, receipts, backups, bundles and exports.
---
# Record

- **Record**: The set of writer logs a node holds. Every derived artifact
  derives from it and the node's own key set (I10); the audit log, counters and
  **configuration** sit beside it: the principal's settings (§9.6), which a
  repository's `.cairn.toml`, its **repository configuration**, may only
  tighten: set only the keys §9.6 marks settable there, in the direction it
  names, such as lowering a limit or turning an injection path off, never
  enabling injection, lowering the pin budget or otherwise removing a pin from
  a restore block, extending recall scope, changing the trust policy or the
  deployment mode, or disabling redaction or flags (SEC-11). Repository
  configuration declares no pin, and a key of it that would make a change ADM-04
  takes only from accepted configuration is ignored, audited and counted
  (ADM-04). Managed policy, configuration and repository configuration are the
  three **settings layers**. The **store** is the home's files containing the
  record, its derived artifacts and payloads (§8.1).
- **Writer**: One seat's append-only log, written on one node or paired phone
  and held by any node, named by its **writer id**, derived from the seat's
  first key. Its events are hash-chained; the **chain head** at a seq is the
  hash over every event up to it.
- **Retire a writer**: Mark a writer retired, naming its last accepted seq, by
  a principal act of the principal its seat key chains to (any other is void
  and shown) or the expire act ending the access token whose token key
  certified its seat key. A peer still accepts segments past it that continue
  its chain without a fork, marked delivered after retirement (open for the
  room merge: OQ-41); only a revocation refuses them (PEER-05).
- **Event**: One immutable entry in a writer, such as a message (the harness's
  user input or the model's reply), tool call, tool result, **hook observation**
  (what a hook reported), worktree checkpoint, key rotation, tombstone, or an
  act (a room act, a principal act or an expire act), a **recall event** (one
  recall, RCL-07), a canary event, a backup event, an agent's pin candidate, an
  erasure or quarantine request, or an unparsed line. Its **event kind** is
  which kind of entry it is, recorded apart from its provenance.
- **seq**: An event's position in its writer: strictly increasing, gap-free and
  never reused (REC-06).
- **Address**: (writer, seq), shown as short `A2·4812` (its writer label and
  seq), range `A2·4812–5025` or full `cairn:room/<room>/w/<writer>/<seq>`. Every
  shown form is accepted wherever an address is taken, and so is one ASCII input
  form, `<label>:<seq>` or `<label>:<from>-<to>` (for example `A2:12`,
  `A2:30-40`), with the writer id accepted in place of the label (RCL-08). A
  **writer label** is a writer's short display alias, unique per node, each
  **writer-label assignment** recorded as a structural event in the assigning
  device seat's personal-room writer. An address range lies in one writer;
  "range" on its own means an address range. The only meaning of "address".
- **Payload**: The full content of a large event, stored outside the event in
  the store's **payload store**, under a name that confirms no guess at its
  content (REC-09).
- **Segment**: A range of one writer, sealed as a unit when it closes; what
  peers exchange. A unit of the record, not a storage layout, though a writer's
  closed segments may be merged (REC-25) or rewritten by a purge (ADM-07)
  without changing any address. The **open segment** is a writer's newest,
  still growing; it closes at the points REC-19 names, and peers exchange its
  sealed prefix. A closed segment or an open segment's sealed prefix is a
  **sealed range**. A refused segment is recorded as a structural event in
  this node's device seat's personal-room writer once per writer and reason,
  when first refused, beside its audit entry, so its refusal derives from the
  record and a rebuild appends nothing (REC-21, I10).
- **Seal**: A seat key's signature over its writer id, a seq and the chain head
  at that seq (REC-18), made where that key lives: a witnessed run's run seat's,
  but the seat ingest starts, by its run's MCP server, covering what the hook
  handlers appended, an ingested run's by the core, a paired phone's device
  seat's by the phone (OQ-39), and every other writer's by the core; how the
  core signs and seals what the room-view component, the launcher and the bridge
  component record rests on OQ-40. Events after the newest seal are unsigned.
- **Commitment**: A keyed commitment to an event's content under a per-event
  random key, its **commitment key**, kept with the content and erased with it;
  the only way the chain, seals and tombstones refer to content, but for what a
  tombstone keeps (REC-17, Purge).
- **Provenance**: An event's class from a closed set, saying what produced its
  content (PRV-01): `user`, `assistant`, `tool_call`, `tool_result:<tool>`,
  `web`, `mcp:<server>`, `file`, `subagent_result`, `harness_meta`,
  `harness_text`, `operator`, `post`, `summary`, `structural` and `unparsed`.
  `harness_meta` is what the harness reports, or the launcher records, about the
  harness's operation, never free text: turn triggers, model tokens, metadata
  lines that carry no free text and sandbox state (PRV-08, OWN-22); `operator`
  is the class of principal acts, expire acts and device-seat pins; posts are
  `post` and run-seat pins `assistant`; every other room act but a room summary
  takes its seat's **pin class**, `operator` for a device seat and `assistant`
  for a run seat; a room summary is `summary`, never trusted; an erasure or
  quarantine request a principal act sends is `operator`, an erasure request a
  retention policy sends `structural`; a worktree checkpoint is `file` (REC-20)
  and a recall event `assistant` (RCL-07); hook observations, key rotations,
  tombstones, canary and backup events and a witness check's record (the
  command by commitment, its exit status and the tree hash, OWN-18) are
  `structural`.
- **Origin**: How an event reached this node's record: `witnessed` (recorded
  live on this node: by its hook handlers, its CLI, TUI, MCP server, launcher,
  room-view component or bridge component), `ingested` (appended by `cairn
  ingest`, whatever transcript it reads, past an ingest marker included),
  `bundle` (read by import) or `peer` (received from a peer or a paired phone,
  or recorded `witnessed` before this node's node identity changed, Node)
  (RCL-09); the trust policy reads only whether an event was recorded on this
  node, ingested or received, and `bundle` beside `peer` is a display mark. A
  **witnessed run** is one this node's hook handlers watched. Independent of
  provenance. Each event also records its **recorder**, the part that recorded
  it (one of those `witnessed` names: the bridge component's events take
  provenance `web`, always untrusted; the room-view component records a
  principal act taken in the browser room view, marked with its principal
  surface, and how it is signed rests on OQ-40), which the trust policy reads;
  an event from a peer or a bundle records no recorder, only its origin.
- **Span**: A contiguous range of one run's events in one writer. A new span
  starts at every user turn, compaction, subagent start or end, and whenever the
  run's events move to another seat's writer (LMK-01).
- **Landmark**: A structural headline over one span, bound to its address range
  (LMK-02). LMK-06 may add a **natural-language headline** to a span whose every
  event is trusted, made only by a deterministic template over trusted fields;
  it is no structural field and never reaches a restore block. Landmarks roll up
  into **tiers** of at most `k` **landmark blocks**, each a sequence of
  landmarks; a full tier collapses its older landmark blocks to one line each,
  listing one address range per writer, into the next tier (LMK-05).
- **Structural field**: A field Cairn derives from an event's shape, never from
  its text: ids, kinds, counts, tool names, key fingerprints, addresses, version
  numbers, sanitized paths and exit status, sanitized under LMK-03. Sanitized,
  it is trusted whatever the event's trust level (PRV-06, LMK-03).
- **Structural event**: An event of provenance `structural`, whose meaning lies
  only in structural fields. A hook observation keeps only a hook's structural
  fields; content a hook carries is recorded under its own class. An act's
  structural fields count as structural fields, never the text it carries; the
  act keeps its own class.
- **Capture**: This node's hook handlers recording its runs' events; turning it
  off is widening (OWN-11), so `cairn uninstall` records that act before it
  removes the hook registrations (ADM-02).
- **Ingest marker**: What a hook handler leaves when its hook budget cuts it
  short, so the next hook handler or `cairn ingest` resumes it (NFR-02).
  **Ingest** is reading a transcript into the record, by the hook handlers
  incrementally (REC-13) or by `cairn ingest`; what `cairn ingest` appends takes
  origin `ingested` and is never recorded by the hook handlers, even past an
  ingest marker. A transcript source's **ingest position** is how far ingest has
  read it: a byte offset and a hash of the consumed prefix (REC-03).
- **Worktree checkpoint**: An event recording a worktree's commit, branch and
  redacted diff since the previous worktree checkpoint (REC-20).
- **Derived artifact**: Anything computed from the record and the node's key
  set, such as those I10 lists: the **search index** over event text among its
  indexes and the node's principal's Needs you queue among its queues. Results
  with their evidence
  classes are derived artifacts; room status, a check's state and a run status's
  time-relative freshness marks are computed where shown; the **quarantine set**
  is what a node holds quarantined.
- **Redaction**: Removing secrets from content before it is stored, on import,
  or from what an export's, a publish's or an invite's review step sends,
  recorded (I1, SEC-08).
- **Retention policy**: A rule (`retention_policy.*`), set by the node's
  principal or managed policy, that purges content per room and provenance class
  after a time (I1, SEC-22); the principal's change to one is widening and
  applies only once its configuration is accepted (ADM-04).
- **Flag**: A mark Cairn sets on an event whose text matches an injection
  pattern (PRV-07); flagged text contributes only counts to landmarks (LMK-04).
- **Quarantine**: Recorded, reversible exclusion of an event, span, run, writer
  or derived artifact from recall, restore blocks, landmark text (LMK-04 keeps
  its counts) and trusted-only exports, on the node that records it, and still
  after its node identity changes (Node), without deletion (I5). On that node,
  no quarantined post is endorsed or reaches an agent under a trust grant,
  and no quarantined delegated task reaches a delegate in OWN-24's template
  (SEC-12). A quarantined pin version never restores and counts as absent when
  PIN-10 chooses which version of a pin restores, the intent's included; a pin
  left with no qualifying version leaves the restore block.
- **Purge**: Deletion of content, appending a **tombstone** naming the purged
  range, which views show in its place (ADM-07): by a principal act, or by the
  node under a retention policy, recorded naming the policy. The only way stored
  content is destroyed (I1); it never undoes an act's effect, as Relations
  says. A purged pin version counts as absent when PIN-10
  chooses a version, as a quarantined one does, and never restores, not even as
  its tombstone.
- **Gap marker**: What stands where content is missing: a tombstone for a purged
  range, a **quarantine marker** for a quarantined address, or a **truncation
  marker** on capped kernel output. A **missing range** is part of a writer this
  node does not hold, such as a peer's lost tail; a **capture gap** is part of a
  run its capture did not record.
- **Integrity status**: What a room or writer shows about its chain and seals,
  one of §9.7.5's values (VIEW-10); `refused` shows a refused segment
  (Segment). The UI never says "secure".
- **Receipt**: A signed statement about a node's record, made to be kept apart
  from the home and checked with no network. Always qualified: a **head
  receipt** lists every writer's chain head at a moment, the tamper evidence of
  VIEW-10 and SEC-27; a **purge receipt** states what a purge erased and what it
  could not (SEC-31).
- **Backup**: A copy of a home's store and audit log, with no seat, device,
  token or at-rest key (`cairn backup create`, ADM-06, SEC-10); reading
  it back is a backup restore. Creating one records no principal act, only an
  audited event (§9.5), and a copy the principal takes off the machine is its
  own tool, outside Cairn's components (I4).
- **At-rest key**: The key that encrypts the store when encryption at rest is on
  (SEC-09).
- **Bundle**: A reviewed export of a room, signed by the device key of the
  device that exported it, its **exporter**, which chains to the **bundle's
  principal key**, carried as a file or a git ref.
- **Trusted-only export**: The JSONL of the events trusted for the principal's
  agents, with full provenance, that downstream memory systems read (ADM-12,
  MEM-02); an export like a bundle (SEC-26). It leaves out the free text of a
  cut or neutral principal act (OWN-11), which its review step shows as
  withheld.
- **Export**: Writing a bundle, a rendering or a trusted-only export (`cairn
  export`), a widening principal act under SEC-26. To **publish** is to serve a
  room's bundle read-only through the publish component, only as the room's
  visibility allows; a widening principal act under SEC-26, and stopping it is
  cut.
- **Rendering**: A room rendered for people to read, with no keys or commitments
  (`cairn export --rendering`, SEC-26).
- **Import**: Reading a bundle into this node's record (`cairn import`, REC-23).
  A transcript is ingested and a peer's segments are received; neither is
  imported.
- **Kernel**: The hermetic compute environment in which an agent runs code over
  the record (CMP-01); each run's **kernel variables** live in its own
  namespace.
