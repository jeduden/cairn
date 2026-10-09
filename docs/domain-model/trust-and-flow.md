---
title: "Trust and flow"
order: "09"
summary: >-
  Trust and how content reaches an agent: trust levels and sources, recall, endorsement, trust grants, delegation, rule levels, away policies, fixed templates, sandbox state and risk acceptance, quotas and spend, counters, canary and stats, run status and room status, the Needs you queue and focus set, overlap, forks, presence hints and petnames.
---
# Trust and flow

- **Trust level**: `trusted` or `untrusted`, per event for one principal's
  agents on one node: derived by the trust policy from the event's provenance,
  origin (whether recorded on this node, ingested or received), recorder and
  writer, the deployment mode recorded with the event, the node's key set,
  whether the event's room is foreign to that principal, and that principal's
  stamps and trust grants as its writer logs carry them (I10). The free text a
  cut or neutral principal act carries, such as a stated reason, is untrusted
  whatever its event's trust level (OWN-11).
- **Trusted sources**: I2's trusted sources as PRV-02 applies them, which RCL-10
  narrows in a foreign room: this node's `operator` and structural events, the
  `harness_meta` events its hook handlers recorded, and the `user` events they
  recorded while the deployment mode is `interactive`, all trusted only on this
  node; and, once PRV-10 ships, principal acts signed by a device key the
  agent's principal certified, posts and pins written from a device seat such a
  key certified, within that key's scope, a pin version its principal stamped,
  and for that agent the posts and pins a trust grant of its principal covers,
  none of them in a foreign room but the pins and stamps Foreign room names.
  Everything else is untrusted, as is a cut or neutral act's free text (OWN-11).
- **Deployment mode**: `interactive` (a person types at the harness) or
  `automation` (a pipeline does, the default), set per node (PRV-02, PRV-04,
  `node.deployment_mode`);
  changing it, loosening redaction or turning flags off applies only once
  its configuration is accepted (ADM-04).
- **Trust policy**: The rule that derives each event's trust level (PRV-02); no
  settings layer changes it.
- **Trust mark**: The sign beside an item saying where it came from and whether
  it is trusted, from §9.7.6's closed set (VIEW-07).
- **Taint**: The trust level a derived artifact, or an output built from events
  (a recall result, kernel output, an exported bundle, rendering or trusted-only
  export), inherits: untrusted when any event it derives from is untrusted or
  it holds a cut or neutral act's free text (OWN-11), except for its sanitized
  structural fields.
- **Recall taint**: A run's mark after it recalls untrusted content, which
  tightens the rule levels of the action classes SEC-13 configures as
  sensitive (OWN-10). A delegate inherits the delegating run's (OWN-24),
  and carries recall taint from its first event where its node cannot derive
  that run's from the writers it holds (OWN-26).
  A delegating run takes on a subagent's recall taint once that subagent's
  `subagent_result` reaches it, as pulling any other delegate report through
  `delegation_get` taints it (OWN-25). A run
  a harness resume starts carries the recall taint of the run it resumes, and
  carries recall taint from its first event where Cairn cannot tell that run
  (SEC-13).
- **Principal surface**: An authenticated client where principal acts are taken:
  the browser room view under SEC-20, the CLI or TUI at a terminal, or a paired
  phone within its scope; without the peer component, a browser on the
  principal's phone reaches the room view only as the browser room view, through
  the principal's own tunnel (§6.3 row 11). An unsandboxed agent can still pass
  the CLI's or TUI's terminal test, through a terminal it opens or a process it
  did not start: that is §6.1's residual risk R7, and its writing the store or
  signing outside Cairn's code is R3, which only a sandbox removes (Sandbox
  state).
- **Presence proof**: A hardware-backed, user-verified proof of a person's
  presence, bound to one widening act (OWN-11).
- **Recall**: An agent's call to a **recall tool**: an MCP tool that returns
  enveloped content (§9.2), such as `event_search`, `room_get`,
  `room_summary_get`, `delegation_get` or `kernel_exec` with its built-ins
  (CMP-03), or a read-only CLI verb the agent runs whose output is not a
  terminal (OWN-12); each records a recall event (RCL-07). Pull-only,
  defaulting to the agent's current run. A recall tool that stops at its
  per-call cap returns a **continuation cursor** (`next_cursor`); passed back as
  `cursor`, it continues without gap or overlap (RCL-03). Outside an envelope
  and the closed paths, whatever Cairn returns to a tool call, its errors and a
  CLI verb's output that is not a terminal included, carries only fixed text
  Cairn ships, ids, counts and sanitized structural fields (I2).
- **Closed path**: One of the ways I2 lists by which Cairn writes to an agent
  without the agent's recall: restore blocks, opt-in notices, compaction
  guidance and OWN-04's and OWN-07's templates; on a principal act recorded at
  that time, through the harness's own input, principal-typed text, a fixed
  template referencing ids or an endorsed post; a delegated task under its
  grants; and what a trust grant covers. No other write to an agent exists (I2),
  and data leaves the machine through them only as the harness sends them to its
  model (I4).
- **Recall scope**: `run`, `room` or `rooms`, or a foreign room named in the
  call (RCL-05). A wider scope is said to extend recall; "widening" belongs to
  acts.
- **Endorsement**: The principal act of a principal with a seat in the room,
  sending a post's text, exactly as the principal confirmed it after any edit,
  to one of the principal's own agents, inside a fixed template naming the
  author's seat key fingerprint and the post's address (OWN-08).
- **Trust grant**: A principal's widening act trusting another principal's key
  for its own agents, in one room or everywhere. It covers that principal's
  posts and pins written from device seats a device key of that principal
  certified, within that key's device scope (PRV-10), never a cross-room post
  shown in another room, run seats,
  token-key-only nodes or room summaries; never in a foreign room, but for the
  pins a leave, kick or bar keeps there (Foreign room); and a post it
  covers reaches only the grantor's agents whose run has a seat in the post's
  room. A grant naming a service account whose certificate or managed-policy
  listing marks it as relaying text others wrote, or whose listing is unmarked,
  is refused; one naming any other service account shows a warning, for a
  facilitator that it reads untrusted room text, which its posts never quote
  and its pins never carry (Facilitator, OWN-29). Its revocation is cut.
  A recorded grant covers nothing once the node's key set holds a certificate
  or listing marking the key it names as relaying, or a listing of it without
  the mark, whatever order they arrived in (I10); no revocation or removal
  lifts that, and a revocation of any certificate of that key, or a removal of
  any listing of it, leaves every grant naming it that does not causally follow
  that act covering nothing (Service account).
- **Delegation**: One agent handing another a delegated task: its subagent, or,
  under a delegation grant, another agent of the same principal, an agent on
  another of its nodes, or another principal's agent (OWN-23 to OWN-26). How
  a delegated task and its delegate report travel between two principals'
  nodes rests on OQ-14.
- **Delegated task**: The text of a delegation: a subagent's from its harness,
  any other in OWN-24's template.
- **Delegate report**: What a delegate returns, recorded on the delegating side
  so it stays in the delegating run's recall scope (OWN-24), enveloped: through
  `delegation_get`, except a subagent's, which its harness returns as untrusted
  `subagent_result` (OWN-25).
- **Delegation grant**: The delegating principal's widening act naming who may
  delegate, to which targets, at what rule level, within what **delegation
  budget** (its cap in spend), how many delegates at once, how deep, and until
  when (OWN-23); past that expiry the delegating node refuses a delegation under
  it, and its expire act ends it.
- **Acceptance grant**: The receiving principal's widening act on its own node,
  naming the delegating principal, the targets, the maximum rule level, the
  delegation budget and the expiry (OWN-26), which ends it as a delegation
  grant's does. Delegation to another principal's agent needs both a delegation
  grant and an acceptance grant.
- **Permission grant**: The grant that lets an approved action pass again when
  the harness next raises it at its permission prompt (OWN-07). Separate from
  trust and delegation grants.
- **Notice allowance, notice opt-in**: The owner's per-room allowance and the
  agent's principal's opt-in. An opt-in notice needs both.
- **Action class**: A kind of tool action, such as edits, commands or network
  use (OWN-10).
- **Rule level**: For each action class, one of, loosest to tightest: act
  without asking, act when told, ask first, hand off (OWN-10). A maximum rule
  level is the loosest allowed.
- **Quota**: A limit on storage or events, per room, node, writer received from
  a peer, peer or worktree checkpoint, that the node's principal or managed
  policy sets (ADM-15). Reaching one never drops or deletes an event: what is
  received over it is refused and audited, and this node's own recording over it
  goes on, counted and raised as a Needs you item (I1, I6). Its first crossing
  by this node's own recording, at each limit set for it, is recorded as a
  structural event in this node's device seat's personal-room writer, as a
  refused segment is, and that item derives from it (I10).
- **Spend**: What runs cost in model tokens or money; no quota limits it until
  OQ-26 closes.
- **Away policy**: A principal's opt-in choice of what an unanswered held
  request does while it is on: keep going, pause or stop (OWN-07). Only that
  principal act sets it, never a settings key.
- **Residual risk**: One of the risks §6.1 lists for an unconfined run.
- **Sandbox state**: What confines a run, its policy digest, and which residual
  risks it blocks, recorded from outside the sandbox, where the agent cannot
  write; a run with none recorded counts as unsandboxed (OWN-22).
- **Risk acceptance**: The principal's act accepting the residual risks a run
  leaves open (OWN-22).
- **Hand-off, hand-back**: An agent passes control to its principal as a held
  request; the principal returns control with a note and a worktree checkpoint
  (OWN-20).
- **Run controls**: Steer, interrupt, pause, resume and stop: principal acts on
  a run. **Terminal takeover**, the principal typing in the harness's terminal
  the launcher hosts, is the harness's own channel, never a principal act
  (OWN-02, OWN-19): the person's keystrokes are the harness's own input from
  that terminal, which the launcher never carries, so what the launcher
  carries into the harness's input stays text the core built (I4).
- **Fixed template**: Wording Cairn ships, filled only with ids, counts, version
  numbers, key fingerprints, addresses and the principal-typed, endorsed or
  delegated text, or the text of a post a trust grant covers, its requirement
  names; **principal-typed text** is text a principal typed at a principal
  surface for that act.
- **Correction, retry**: After a verdict: principal-typed text in a fixed
  template, or a new run from a worktree checkpoint, whose join to the room the
  retry itself asks for (OWN-28, LANE-23).
- **Counter**: A count of dropped, rejected, redacted, truncated, coalesced,
  timed-out or failed operations of one kind, each written to the node's
  append-only **audit log** of Cairn's own operations (OPS-01), shown by
  `cairn status` (ADM-11), failing `cairn doctor` until acknowledged
  (`cairn counter ack`, I6, OPS-03).
- **Canary**: `cairn canary`'s end-to-end test: it writes a **canary event**
  through the hook path and recalls it through the MCP server (OPS-04).
- **Stat**: A count derived from the record within a recall scope, such as
  events, runs, compactions or recalls, read through `stat_list` (RCL-01).
- **Run status**: A run's one status from §9.7.1's closed set (one waiting on
  its principal is Asking, never Needs you), with its **freshness mark**, which
  says how current the run's events are on this node, whether hooks still arrive
  (`unrecorded`), or that it is an ingested run (`ingested`); `stuck?` is a
  watchdog observation (VIEW-04); time-relative freshness marks are computed
  where shown, never derived artifacts.
- **Room status**: A room's one status from §9.7.2's closed set, such as
  Running, Quiet or Ready for review. It is computed where shown from room
  state, the heads, landings and check states of the branches it names, its
  runs' statuses and their freshness marks. It is never recorded or set
  directly; OWN-21's ready and abandoned marks feed it.
- **Queue class**: One of Needs you's classes Q1 to Q4, which set its order
  (§9.7.4, VIEW-05).
- **Focus set**: The rooms a principal marks to come first in Needs you, changed
  by a recorded neutral principal act, so every device shows one order.
- **Open room**: A room with no ready mark whose branch heads still stand
  (OWN-21) and no abandoned mark, that names no branch or names one that has not
  landed.
- **Exposure**: Of a branch, the untrusted and flagged items its runs read and
  their recall taint (VIEW-13).
- **Overlap**: Runs in two open rooms editing one file, which raises a Needs you
  item for each room's owner until that owner acknowledges it for that room
  (LANE-13).
- **Fork**: Either of two events one writer sealed at one seq, both kept for
  forensics; the node's principal chooses which fork to keep (PEER-10).
- **Presence hint**: An ephemeral sign that a seat is connected, or typing (a
  **typing hint**), never stored in the record (PEER-09).
- **Watchdog observation**: Cairn's note, computed where shown and never
  recorded, that a run looks stuck, shown as the `stuck?` freshness mark; it
  never starts or resumes a turn (OWN-09).
- **Petname**: A name the person viewing chose for a key or a room, never one
  another principal sent (VIEW-07). Notifications carry a room's petname, else
  its id.
