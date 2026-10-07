---
title: "Acts and roles"
order: "07"
summary: >-
  The three act kinds and what they may do: room acts, principal acts and their classes, expire acts, roles, appointments, joins, moderation, handover, stamps and the room merge.
---
# Acts and roles

Room state derives from three kinds of act only. Seats that are members with no
add, and the seat ingest starts, stand beside them (Seat). Each act belongs to
exactly one kind (LANE-31).

- **Room act**: An act signed by a seat key, wherever it is taken. It is checked
  against the capabilities of the seat's role and the room state; create room,
  join, a join request and leave are checked against admission and the add
  instead. The room acts are create room, join, a join request, leave, a role
  request, post, link, pin, edit, unpin, list removal, present, pick, kick, bar,
  unbar, mute and unmute; set title, labels or an assignment; and write a room
  summary or a summary request. A leave, kick or bar never drops a room's
  qualifying pins from a run's restore block, within the foreign-room limit of
  Relations. A pin, edit or unpin room act never changes any restore block: an
  author's own pin, edit and unpin act only on pins that do not restore
  unstamped, and a list removal only takes a pin off the pin list. A link act
  adds one range link, branch link or criterion link. Create room is the first
  act of the creating seat's writer. Room acts are governance, not I2 trust: on
  their own they never widen what reaches an agent, which only principal acts
  such as a trust grant, an endorsement or a stamp do, and OWN-11's classes do
  not cover them.
- **Principal act**: An act of a kind OWN-11 classes, taken at a principal
  surface. It is signed by a device key once PRV-10 ships (OWN-02); before then
  it is an `operator` event its device seat's seal covers, told apart from a
  room act by the principal surface it is marked with (OWN-02). An act a seat
  key signs as the act, beyond sealing its writer, is a room act. A widening act
  lets more reach an agent, or more act or leave the node; a cut act only stops,
  narrows or undoes a widening; a neutral act does neither, such as
  acknowledging counters. Every change of visibility, admission or a room
  setting is widening, whichever way it goes. Its classes:
  - **Cut:** deny, interrupt, pause or stop a run, cancel a delegation, end a
    permission grant, reject a foreign room, decline a join request, tighten a
    rule level, revoke a role assignment, revoke a CI key, stop publishing a
    room, quarantine content without removing anything from a restore block,
    withdraw a risk acceptance, revoke a trust grant, end a delegation grant or
    an acceptance grant, turn an away policy off, withdraw a named successor,
    revoke an appointment, unstamp a pin version, turn notices off, decline a
    handover, withdraw as successor, dismiss a directed post, revoke an access
    token, a seat key or a service account's certificate, turn capture on, and
    turn off the room-view component, the launcher, the peer component, the
    publish component, a bridge or the git carrier.
  - **Neutral:** mark a room ready, its **ready mark** (the owner or a principal
    whose device seat in the room is a moderator), or abandoned, its **abandoned
    mark** (the owner), acknowledge an overlap, record a `met`, `not met` or
    `needs changes` verdict, unpin a verdict, open the forensic view, make a
    purge request, refuse a quarantine request, an erasure request or a purge
    request, choose a branch to compare, acknowledge counters, dismiss a Q3 or
    Q4 item other than a directed post, unpin a pin its own agent's run seat
    wrote, and add a room to or remove it from the focus set.
  - **Widening:** allow, answer a hand-off (a hand-back), reply, steer, confirm
    a steer for a later turn (OWN-13), send a correction, retry from a worktree
    checkpoint, set or revise an intent, resume, record a delegation grant or an
    acceptance grant, add, edit or unpin a pin of a type that restores written
    from a device seat a device key certified (before PRV-10 ships, this node's
    own device seat), confirm a pin candidate, change a room's visibility,
    invite a principal key, issue an invite link, choose a fork, turn on the
    room-view component or the launcher by accepting the configuration that
    turns it on (ADM-04), turn on the peer or publish component, accept a
    handover or succession, ask for a device seat's join to a room its principal
    has no member seat in, ask for or accept a join, which sends the run's
    events to a room other principals' nodes hold, endorse, loosen a rule level,
    turn on or change an away policy other than turning it off, quarantine that
    removes a pin or a landmark from a restore block, release a quarantine,
    purge or apply an erasure request, export, bind a repository identity by
    hand or rebind it, accept open residual risks (OWN-22), certify a service
    account's principal key, assign a role, set a room's admission, appoint a
    moderator or the facilitator, stamp a pin version, name a successor, hand
    over a room, record a trust grant, allow notices for a room or opt in to
    them, enable a bridge, which turns the bridge component on while any bridge
    stands enabled, or the git carrier, set a room setting, publish, apply a
    purge request, change a retention policy, accept configuration (recording
    its digest), enroll a CI key, rotate a device key, retire a writer, turn
    capture off or pause it, enroll or revoke a device, peer or authenticator,
    mint or rotate an access token, start a witness check, which confirms the
    command taken from an untrusted event (OWN-18), and a backup restore.
    Revoking a device or a peer stays widening although it undoes an enrollment,
    since it can drop pins and stop acts arriving.

  Any principal act that removes a pin from a restore block, or stops this node
  recording its own runs' events, is widening whatever verb carries it, so
  `cairn uninstall` records the widening act turning capture off before it
  removes the hook registrations (ADM-02). On an ephemeral node, PEER-01's
  environment setting, a structural event, stands in for turning on its peer
  component and, for the git-carrier remotes its access token names, its publish
  component; minting that access token is the widening act. An unstamp and a
  trust-grant revocation are the exceptions: each withdraws only the acting
  principal's own trust (LANE-32, OWN-29). Applying a quarantine request takes
  the class of the quarantine it applies. A principal act no requirement names
  is widening. OWN-11 and OWN-12 follow this list, and every principal act a
  requirement names is classed here.
- **Expire act**: Once PRV-10 ships, an act a node with a device key records,
  signed with that key, ending only an expiry its original act set. It ends a
  bar, a mute or a handover offer (LANE-25). It ends an access token, recorded
  by the node that minted it (PEER-05). It ends a delegation grant or an
  acceptance grant, recorded by the node of the principal that recorded the
  grant (OWN-23, OWN-26). Before PRV-10 ships no expiry can be set but a
  grant's, which then takes effect only as a refused delegation (Delegation
  grant).
- **Role**: A named set of room capabilities: viewer, contributor or moderator.
  The owner gives a seat its role by a **role assignment**. An invite or invite
  link records one for every seat that chains to the invited principal key and
  joins under it, run seats included, though a run seat an invite names
  moderator is a contributor. A paired phone's personal-room seat has no role
  and reads within its device scope. The seat ingest starts takes over the role
  of the run seat it names (REC-19). A seat with none, other than the owner's
  device seats, is a viewer; a run's personal-room seat, or its seat in a room
  it created, is a contributor. A handover records a moderator role assignment
  for the former owner's device seats (LANE-11). A run seat is a moderator only
  by appointment. Only the facilitator's device seat writes a room summary,
  beside its appointment.
  - **Viewer:** read, a role request and a summary request.
  - **Contributor:** read and a summary request; post, link and present; pin,
    edit and unpin its own pins, its **pin capability**, a device seat's own
    pins being every pin its principal wrote from a device seat within that
    seat's device scope; and work.
  - **Moderator:** a contributor's capabilities, plus a list removal of any pin
    but the intent or a verdict, kick, bar, unbar, mute, unmute, pick, and set
    title, labels and assignments.
- **Capability**: What a role lets a seat do. The closed set LANE-16 lists: what
  the three roles list, plus writing a room summary, which only the
  facilitator's appointment carries.
- **Work**: A capability, not an act: to edit, execute commands, write worktree
  checkpoints and commit (LANE-16); a run's events go to its run seat as
  Relations says; an assignment only asks. As a capability, "work" means nothing
  else; what one agent hands another is a delegated task.
- **Appointment**: A principal act that makes a run seat or a device seat of a
  principal other than the owner an **appointed moderator**, a moderator within
  SEC-32's limits. The owner may appoint, and so may a principal whose device
  seat has the moderator role by role assignment, except the facilitator, whom
  only the owner appoints. The appointer or the owner may revoke it.
- **Join**: The act that adds a seat to a room under its admission, or without
  it for a device seat whose principal has a member seat there. A run joins only
  when its principal asks for the join or accepts it (LANE-23). That covers the
  branches the room names later by branch links, whose runs' events then go
  there (LANE-01), so a branch link stays a room act. A device seat joins by a
  join room act its principal takes at a principal surface. Its node also joins
  it before recording there a principal or expire act, a tombstone, an erasure
  request a retention policy sends, or a bridge's or the launcher's event about
  a room's branch, without admission when its principal has a member seat there.
  A paired phone never joins. An owner whose seats have all left rejoins under
  admission, which its principal key always satisfies (Admission). **Leave** is
  a seat's room act ending its own add.
- **Kick**: Revokes a seat's current add. Only that seat's principal may add it
  again (LANE-25).
- **Bar**: Names a principal key and keeps every key that chains to it out,
  until an unbar or an expire act (LANE-25). Its setter where SEC-32
  permits, the setter's appointer or the owner may unbar.
- **Mute**: Withdraws every capability but read from one seat or from the whole
  room; a whole-room mute leaves posting to the roles the owner names (LANE-16).
- **Present**: Puts a presentation in the outcome window.
- **Pick**: Chooses which presentation the outcome window shows.
- **Admission**: Whether a room is invite only or admits a list of principal
  keys. An invite (a principal key and a role) and an invite link are widening
  principal acts of the owner. The owner's principal key always satisfies its
  room's admission, for its device and run seats alike. An invite's, an export's
  or a publish's **review step** shows the acting principal, before it takes
  effect, what will take effect or leave, SEC-08 applied (LANE-10, SEC-26).
- **Successor**: A principal the owner names in advance, who accepts ownership
  once every seat of the owner has left the room; that acceptance is a
  **succession**. Until a handover, a succession or the owner's rejoin, a room
  whose owner left keeps its pins as they were, but a stamper may still unstamp
  (LANE-11, LANE-32).
- **Handover**: Transfers ownership by an offer and an acceptance. Succession is
  the other path to ownership.
- **Stamp**: The act of a principal with a seat in the room on one pin version,
  after being shown its exact text, author and key fingerprint. A stamped
  version of a type that restores restores word for word to that principal's own
  agents only. An edit, an unpin or a list removal leaves a stamped version
  restoring until its stamper unstamps it, a cut principal act (LANE-32).
- **Active pin**: A pin on its room's pin list (I10).
- **Room merge**: The one rule deriving all room state from the three act kinds,
  in causal order (LANE-31). Seats that are members with no add, and the seat
  ingest starts, stand beside it. It covers membership, roles, appointments,
  pins, pin versions and stamps. It covers mutes, presentations, picks, bars,
  handovers and their offers, OWN-21's ready and abandoned marks, the CI keys
  enrolled in the room and whether the git carrier is enabled for it. It covers
  the successor, title, labels, assignment, visibility, admission, the notice
  allowance and the **room settings** (whether drafts show live, whether typing
  shows, the roles a whole-room mute leaves posting to, and the **appointment
  rate**: how many kicks, bars and mutes an appointed moderator may set per
  period, PEER-09, SEC-32). When acts conflict, the more restrictive act wins,
  then the lower commitment. Concurrent picks resolve by **pick order** (the
  facilitator's seat, then any other moderator, then the owner, VIEW-22). Of two
  branch links naming one branch, the first in causal order stands, concurrent
  ones by the lower commitment (LANE-01).
- **Concurrent**: Of two acts or events: neither causally after the other;
  **causal order** puts each after every act or event it saw.
- **Room state**: Everything the room merge derives (LANE-31).
