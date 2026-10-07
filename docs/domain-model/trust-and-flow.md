---
title: "Trust and flow"
order: "09"
summary: >-
  Trust and how content reaches an agent: trust levels and sources, recall, endorsement, trust grants, delegation, rule levels, away policies, fixed templates and run status.
---
# Trust and flow

- **Trust level**: `trusted` or `untrusted`, per event for one principal's
  agents on one node: derived by the trust policy from the event's provenance,
  origin (whether recorded on this node, ingested or received), recorder and
  writer, the deployment mode recorded with the event, the node's key set,
  whether the event's room is foreign to that principal, and that principal's
  stamps and trust grants as its writer logs carry them (I10).
- **Trusted sources**: I2's trusted sources as PRV-02 applies them, which RCL-10
  narrows in a foreign room: this node's `operator` and structural events, the
  `harness_meta` events its hook handlers recorded, and the `user` events they
  recorded while the deployment mode is `interactive`, all trusted only on this
  node; and, once PRV-10 ships, principal acts signed by a device key the
  agent's principal certified, posts and pins written from a device seat such a
  key certified, within that key's scope, a pin version its principal stamped,
  and for that agent the posts and pins a trust grant of its principal covers,
  none of them in a foreign room but the pins and stamps Foreign room names.
  Everything else is untrusted.
- **Deployment mode**: `interactive` (a person types at the harness) or
  `automation` (a pipeline does), set per node (PRV-02, `node.deployment_mode`).
- **Trust policy**: The rule that derives each event's trust level (PRV-02).
- **Trust mark**: The sign beside an item saying where it came from and whether
  it is trusted, from §9.7.6's closed set (VIEW-07).
- **Taint**: The trust level a derived artifact, or an output built from events
  (a recall result, kernel output, an exported bundle, rendering or trusted-only
  export), inherits: untrusted when any event it derives from is untrusted,
  except for its sanitized structural fields.
- **Recall taint**: A run's mark after it recalls untrusted content, or
  inherited from its delegating agent (OWN-24), which tightens its rule levels
  (OWN-10, SEC-13).
- **Principal surface**: An authenticated surface for principal acts: the
  browser room view under SEC-20, the CLI or TUI at a terminal, or a paired
  phone within its scope; without the peer component, a phone reaches the room
  view only as the browser room view, through the principal's own tunnel (§6.3
  row 11).
- **Presence proof**: A hardware-backed, user-verified proof of a person's
  presence, bound to one widening act (OWN-11).
- **Recall**: An agent's call to a **recall tool**: an MCP tool that returns
  enveloped content (§9.2), such as `event_search`, `room_get`,
  `room_summary_get`, `delegation_get` or `kernel_exec` with its built-ins
  (CMP-03), or a read-only CLI verb the agent runs whose output is not a
  terminal (OWN-12); each records a recall event (RCL-07). Pull-only, and it
  defaults to the agent's current run.
- **Closed path**: One of the ways I2 lists by which Cairn writes to an agent
  without the agent's recall: restore blocks, opt-in notices, compaction
  guidance and OWN-04's and OWN-07's templates; on a principal act,
  principal-typed text, a fixed template referencing ids or an endorsed post; a
  delegated task under its grants; and what a trust grant covers. No other write
  to an agent exists (I2), and data leaves the machine through them only as the
  harness sends them to its model (I4).
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
  certified, never a cross-room post shown in another room, run seats,
  token-key-only nodes, room summaries, nor a service account its certificate or
  managed-policy listing marks as relaying text others wrote, an unmarked
  listing counting so, refused, while a grant naming any other service account
  shows a warning (OWN-29); never in a foreign room; and a post it covers
  reaches only the grantor's agents whose run has a seat in the post's room. Its
  revocation is cut.
- **Delegation**: One agent handing another a delegated task: its subagent, or,
  under a delegation grant, another agent of the same principal, an agent on
  another of its nodes, or another principal's agent (OWN-23 to OWN-26).
- **Delegated task**: The text of a delegation: a subagent's from its harness,
  any other in OWN-24's template.
- **Delegate report**: What a delegate returns, recorded on the delegating side
  so it stays in the delegating run's recall scope (OWN-24), enveloped: through
  `delegation_get`, except a subagent's, which its harness returns as untrusted
  `subagent_result` (OWN-25).
- **Delegation grant**: The delegating principal's widening act naming who may
  delegate, to which targets, at what rule level, within what **delegation
  budget** (its cap in spend), how many delegates at once, how deep, and until
  when (OWN-23).
- **Acceptance grant**: The receiving principal's widening act on its own node,
  naming the delegating principal, the targets, the maximum rule level, the
  delegation budget and the expiry (OWN-26). Delegation to another principal's
  agent needs both a delegation grant and an acceptance grant.
- **Permission grant**: The grant that lets an approved action pass again when
  the harness next raises it at its permission prompt (OWN-07). Separate from
  trust and delegation grants.
- **Notice allowance, notice opt-in**: The owner's per-room allowance and the
  agent's principal's opt-in. An opt-in notice needs both.
- **Rule level**: For each action class (a kind of tool action, such as edits,
  commands or network use), one of, loosest to tightest: act without asking, act
  when told, ask first, hand off (OWN-10). A maximum rule level is the loosest
  allowed.
- **Quota**: A limit on storage, events or **spend** (what runs cost in model
  tokens or money), per room, node, writer received from a peer, peer or
  worktree checkpoint, that the node's principal or managed policy sets
  (ADM-15).
- **Away policy**: A principal's opt-in choice of what an unanswered held
  request does while it is on: keep going, pause or stop (OWN-07). Only that
  principal act sets it, never a settings key.
- **Residual risk**: One of the risks §6.1 lists for an unconfined run.
- **Sandbox state**: What confines a run, its policy digest, and which residual
  risks it blocks, recorded from outside the sandbox, where the agent cannot
  write (OWN-22).
- **Risk acceptance**: The principal's act accepting the residual risks a run
  leaves open (OWN-22).
- **Hand-off, hand-back**: An agent passes control to its principal as a held
  request; the principal returns control with a note and a worktree checkpoint
  (OWN-20).
- **Run controls**: Steer, interrupt, pause, resume and stop: principal acts on
  a run. **Terminal takeover**, the principal typing in the harness's terminal
  the launcher hosts, is the harness's own channel, never a principal act
  (OWN-02, OWN-19): the person's keystrokes are the harness's own input from
  that terminal, which the launcher hosts but never carries, so what the
  launcher carries into the harness's input stays text the core built (I4).
- **Fixed template**: Wording Cairn ships, filled only with ids, counts, version
  numbers, key fingerprints, addresses and the principal-typed, endorsed or
  delegated text, or the text of a post a trust grant covers, its requirement
  names; **principal-typed text** is text a principal typed at a principal
  surface for that act.
- **Correction, retry**: After a verdict: principal-typed text in a fixed
  template, or a new run from a worktree checkpoint (OWN-28).
- **Counter**: A count of dropped, rejected, redacted, truncated, coalesced,
  timed-out or failed operations of one kind, each also written to the **audit
  log**, the node's append-only log of Cairn's own operations (OPS-01), shown on
  Health until acknowledged (`cairn counter ack`, I6, OPS-03).
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
- **Queue class**: One of Needs you's classes Q1 to Q4, which set its order
  (§9.7.4, VIEW-05).
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
