---
title: "Principals and agents"
order: "01"
summary: >-
  Who acts in Cairn: principals, persons and service accounts, managed policy, agents, runs and subagents, the facilitator, authors, members, the owner and delegates.
---
# Principals and agents

- **Principal**: A person or a service account. It has a principal key, signs
  principal acts with a device key at a principal surface once PRV-10 ships,
  each recorded on a device seat, and is the principal of every agent whose runs
  its nodes record. Cairn counts principals by principal key: every device key,
  token key and seat key that chains to one principal key through device, token
  or seat certificates belongs to that principal; a chain never passes through
  another principal key. A certified service account is still its own principal;
  its **certifier** is whoever certified it. A principal key never certified
  counts as a person's, which nothing can prove; a service account whose
  certificate is revoked stays a service account. Only an agent's own principal
  widens what reaches that agent (I2); a room owner's notice allowance only lets
  through the opt-in notices an agent's principal opted it in to.
- **Person**: A human principal. No one certifies a person's principal key. Only
  a person records a verdict.
- **Service account**: A non-human principal with its own principal key,
  certified by a person, another service account or managed policy, and
  revocable only by its certifier: managed policy revokes one it lists by no
  longer listing it; none starts uncertified. It can own rooms, have seats and
  be the principal of its own agents, such as CI or runner agents.
- **Managed policy**: Settings belonging to root that an organisation sets on a
  machine. Cairn never overrides it (I7). Among its powers, it may disable
  boundaries B1 to B3 and the launcher, forbid risk acceptance and certify
  service accounts by listing their principal keys, each listing marking, as a
  service-account certificate does, whether that account relays text others
  wrote; a listing without the mark counts as relaying. The harness's own
  managed settings, which Cairn also never writes (ADM-03), are part of the
  **harness configuration**: the harness's settings, hooks and MCP registrations
  (I7). Managed policy alone overrides the principal's settings; repository
  configuration only tightens them.
- **Agent**: A worker a harness runs for exactly one principal: the principal of
  the node that records its runs (OWN-01). It receives restore blocks and
  recalls history; an agent is never a principal.
- **Run**: One agent's execution within one harness session, on one node, keyed
  by the harness session and the harness's agent id: a harness session's
  transcripts contain its main agent's run and one per subagent, a **harness
  resume** that keeps its harness session continues the run, while a harness
  resume that starts a new harness session, or a **harness clear**, starts a new
  run. It has run seats and carries its recall taint (SEC-13), sandbox state
  (OWN-22), seat keys (SEC-10), kernel namespace (CMP-02) and spans (LMK-01). As
  a noun, "run" has no other meaning.
- **Subagent**: An agent the harness started for another agent. It has its own
  run, tied to its parent's run by a parent link, with its own seats and
  writers. It takes a delegated task without a delegation grant.
- **Ingested run**: A run ingested from a transcript the hook handlers did not
  watch, with origin `ingested`, whose events are untrusted (REC-22).
  Transcripts the node's principal ingests from outside `transcript.roots` land
  in its personal room.
- **Facilitator**: The service account, at most one per room, whose device seat,
  on its own node, the owner appoints as the room's facilitator, making that
  device seat an appointed moderator (SEC-32); its program, neither a Cairn
  component nor an agent, acts through that node's CLI (`cairn room-summary
  write`). It posts and writes room summaries, answering summary requests
  (LANE-33).
- **Author**: The seat that wrote an event or a pin. The author's principal
  follows from the seat.
- **Member**: A seat whose add stands and that no bar covers, or a run's or a
  paired phone's personal-room seat, or a device seat a newly minted key started
  in the personal room, or the seat ingest starts while the run seat it names is
  a member (Seat, Seat key, Run seat); the seat's role says what it may do. A
  seat that left or was kicked, or that a bar covers, is not a member. A
  principal is never a member; **the room's principals** are those with a member
  seat, and text for people speaks of the room's principals. A personal-room
  seat can neither leave nor be kicked.
- **Owner**: The one principal who owns a room: its intent, roles, admission,
  appointments, successor and handover. Ownership changes only by handover or
  succession (LANE-11); it stays with the owner after all its seats leave. The
  owner stands beside the roles rather than having one: each of its device seats
  in the room, but a paired phone's, has every room capability, for room acts
  and the principal acts that need one, except writing a room summary; its room
  acts edit and unpin only pins its principal wrote from a device seat that do
  not restore unstamped, and make a list removal of any pin but the intent or a
  verdict, while its agents' run seats have only their role and any appointment.
  No one may kick, bar or mute the owner or any key that chains to its principal
  key (LANE-25). "Owner" means nothing else, except in the persona name
  "Returning owner" and where an outside domain qualifies it, as a code owner.
- **Pull-request author**: The outside party whose commits a foreign room's
  bundle describes, matched through their commit-signing identity and a
  **binding statement** that identity signs, naming the bundle's principal key
  (LANE-15). May have no seat at all, and is no principal unless its key is also
  one.
- **Delegate**: Any agent that receives a delegation, a subagent included
  (OWN-24). It keeps its own principal.
