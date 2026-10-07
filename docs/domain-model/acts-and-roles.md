---
title: "Acts and roles"
order: "07"
summary: >-
  The three act kinds and what they may do: room acts, principal acts and their classes, expire acts, roles, appointments, joins, moderation, handover, stamps and the room merge.
---
# Acts and roles

Room state derives from three kinds of act only, and each act belongs to exactly
one kind (LANE-31).

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
  room act by OWN-02's surface mark. An act a seat key signs is a room act. Its
  classes:
  - **Cut:** deny, interrupt, pause, stop, cancel a delegation, end a permission
    grant, reject a foreign room, decline a join request, revoke a CI key, stop
    a publication, record a `needs changes` verdict, quarantine content without
    removing anything from a restore block, withdraw a risk acceptance, revoke a
    trust grant, end a delegation grant or an acceptance grant, turn an away
    policy off, withdraw a named successor, revoke an appointment, unstamp a pin
    version, turn notices off, decline a handover, withdraw as successor,
    dismiss a directed post, revoke an access token, a seat key or a service
    account's certificate, turn capture on, and turn off the peer component, a
    bridge or the git carrier.
  - **Neutral:** mark a room ready or abandoned, acknowledge an overlap, record
    a `met` or `not met` verdict, accept or ask for a join, open the forensic
    view, make a purge request, choose a branch to compare, acknowledge
    counters, dismiss a Q3 or Q4 item other than a directed post, unpin a pin
    its own agent's run seat wrote, and add a room to or remove it from the
    focus set.
  - **Widening:** allow, answer a hand-off (a hand-back), reply, steer, send a
    correction, retry from a worktree checkpoint, set or revise an intent,
    resume, record a delegation grant or an acceptance grant, add, edit or unpin
    a pin of a type that restores written from a device seat a device key
    certified (before PRV-10 ships, this node's own device seat), confirm a pin
    candidate, change a room's visibility, invite a key, issue an invite link,
    choose a fork, confirm a command taken from an untrusted event (OWN-18),
    turn on the peer component, accept a handover or succession, endorse, loosen
    a rule level, turn on or change an away policy other than turning it off,
    quarantine that removes a pin or a landmark from a restore block, release a
    quarantine, purge or answer an erasure request, export, bind a repository
    identity, accept open residual risks (OWN-22), certify a service account's
    principal key, assign a role, set a room's admission, appoint a moderator or
    the facilitator, stamp a pin version, name a successor, hand over a room,
    record a trust grant, allow notices for a room or opt in to them, enable a
    bridge or the git carrier, set a room setting, publish, answer a purge
    request, accept configuration (recording its digest), enroll a CI key,
    rotate a device key, retire a writer, turn capture off or pause it, enroll
    or revoke a device, peer or authenticator, mint or rotate an access token,
    start a witness check, and a backup restore.

  Any principal act that removes a pin from a restore block, or stops this node
  recording its own runs' events, is widening whatever verb carries it. An
  unstamp and a trust-grant revocation are the exceptions: each withdraws only
  the acting principal's own trust (LANE-32, OWN-29). Applying a quarantine
  request takes the class of the quarantine it applies. A principal act no
  requirement names is widening. OWN-11 and OWN-12 follow this list, and a gate
  fails when a requirement names a principal act this entry does not classify.
- **Expire act**: Once PRV-10 ships, an act a node with a device key records,
  signed with that key, ending only an expiry its original act set: on a bar, a
  mute or a handover offer (LANE-25). Before PRV-10 ships no expiry can be set.
- **Role**: A named set of room capabilities: viewer, contributor or moderator.
  The owner gives a seat its role by a **role assignment**, which an invite or
  invite link also records; a seat with none, other than the owner's device
  seats, is a viewer, and a run's personal-room seat, or its seat in a room it
  created, a contributor. A handover records a moderator role assignment for the
  former owner's seats (LANE-11). Only the facilitator's device seat writes a
  room summary, beside its appointment.
  - **Viewer:** read, a role request and a summary request.
  - **Contributor:** read and a summary request; post, link and present; pin,
    edit and unpin its own pins, its **pin capability**; and work.
  - **Moderator:** a contributor's capabilities, plus a list removal of any pin
    but the intent, kick, bar, unbar, mute, unmute, pick, and set title, labels
    and assignments.
- **Work**: A capability, not an act: to edit, execute commands, write worktree
  checkpoints and commit (LANE-16); a run's events go to its run seat while the
  run works on any branch the room names and its role has work; an assignment
  only asks. As a capability, "work" means nothing else; what one agent hands
  another is a delegated task.
- **Appointment**: A principal act that makes a run seat or another principal's
  device seat an **appointed moderator**, a moderator within SEC-32's limits.
  The owner may appoint, and so may a principal whose device seat has the
  moderator role by role assignment, except the facilitator, whom only the owner
  appoints. The appointer or the owner may revoke it.
- **Join**: The act that adds a seat to a room under its admission, or without
  it for a device seat whose principal has a member seat there. A run joins only
  when its principal asks for the join or accepts it (LANE-23); a device seat
  joins by a join room act its principal takes at a principal surface, or its
  node takes before recording a principal or expire act there, without admission
  when its principal has a member seat there. A paired phone never joins. An
  owner whose seats have all left rejoins under admission, which its own invite
  satisfies. **Leave** is a seat's room act ending its own add.
- **Retire a writer**: Seal a writer for the last time, by a principal act or
  when the access token behind its seat key expires (PEER-05).
- **Kick**: Revokes a seat's current add. Only that seat's principal may add it
  again (LANE-25).
- **Bar**: Names a principal key and keeps every key that chains to it out,
  until an unbar or an expire act (LANE-25).
- **Mute**: Withdraws every capability but read from one seat or from the whole
  room; a whole-room mute leaves posting to the roles the owner names (LANE-16).
- **Present**: Puts a presentation in the outcome window.
- **Pick**: Chooses which presentation the outcome window shows.
- **Admission**: Whether a room is invite only or admits a list of principal
  keys. An invite (a principal key and a role) and an invite link are widening
  principal acts of the owner.
- **Successor**: A principal the owner names in advance, who accepts ownership
  once every seat of the owner has left the room; that acceptance is a
  **succession**. Until a handover, a succession or the owner's rejoin, a room
  whose owner left keeps its pins as they were (LANE-11).
- **Handover**: Transfers ownership by an offer and an acceptance. Succession is
  the other path to ownership.
- **Stamp**: The act of a principal with a seat in the room on one pin version,
  after being shown its exact text, author and key fingerprint. A stamped
  version of a type that restores restores word for word to that principal's own
  agents only. An edit or an unpin leaves a stamped version restoring until its
  stamper unstamps it, a cut principal act (LANE-32).
- **Focus set**: The rooms a principal marks to come first in Needs you, changed
  by a recorded neutral act, so every device shows one order.
- **Active pin**: A pin on its room's pin list (I10).
- **Room merge**: The one rule deriving all room state from the three act kinds,
  in causal order (LANE-31). It covers membership, roles, appointments, pins,
  pin versions and stamps. It covers mutes, presentations, picks, bars,
  handovers and their offers. It covers the successor, title, labels,
  assignment, visibility, admission, the notice allowance and the **room
  settings** (whether drafts show live, whether typing shows, the roles a
  whole-room mute leaves posting to, and the **appointment rate**: how many
  kicks, bars and mutes an appointed moderator may set per period, PEER-09,
  SEC-32). When acts conflict, the more restrictive act wins, then the lower
  commitment. Concurrent picks resolve by **pick order** (the facilitator's
  seat, then any other moderator, then the owner, VIEW-22). Of two branch links
  naming one branch, the first in causal order stands, concurrent ones by the
  lower commitment (LANE-01).
- **Concurrent**: Of two acts or events: neither causally after the other;
  **causal order** puts each after every act or event it saw.
- **Room state**: Everything the room merge derives (LANE-31).
- **Room status**: A room's one status from §9.7.2's closed set, such as
  Running, Quiet or Ready for review. It is computed where shown from room
  state, its runs' statuses and their freshness marks. It is never recorded and
  never set directly; OWN-21's ready and abandoned marks feed it.
