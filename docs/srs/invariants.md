---
title: "1.3 Invariants"
summary: >-
  The single source of Cairn's ten invariants, I1 to I10, which every
  requirement serves; other files include it through mdsmith.
---
# 1.3 Invariants

The invariants are Cairn's contract. Every requirement serves at least one of
them. A change that would break an invariant is a design change requiring
security review and a new major version, not a bug fix.

- **I1 — Nothing is lost.** Every event an agent saw or produced remains
  recoverable by its stable address, (writer, seq), across any number of
  compactions and agent runs, on every node that holds its writer's log. The
  only exceptions are secrets removed by redaction before storage or on
  import, and data the node's principal explicitly purges or removes under a
  retention policy. Every exception is recorded.
- **I2 — No automatic path from untrusted content to the model.** Content that
  originates outside the trusted sources (tool output, web, MCP servers,
  files, assistant text, and anything another writer, node or principal
  produced) reaches the model only when Claude explicitly calls a recall tool,
  and always inside an untrusted-data envelope. The trusted sources are this
  node's own `operator`, `harness_meta` and structural events, its `user`
  turns while the deployment mode is `interactive`, and, once PRV-10 ships,
  principal acts signed by a device key the agent's principal certified,
  within its scope. Cairn writes to an agent only through a closed set of
  paths. Without a principal act: restore blocks of pins (INJ-01, INJ-02),
  opt-in notices (INJ-10), and the fixed templates of OWN-04 and OWN-07, each
  built only from trusted structural fields and ids. On a principal act
  recorded at that time: through the harness's own input, only principal-typed
  text, a fixed template that references ids, or a post a principal endorsed
  exactly as shown inside the template of OWN-08. Under a delegation grant its
  principal recorded (OWN-23): a delegated task inside the fixed template of
  OWN-24. Once its requirements ship, a principal may also trust another
  principal by key for its own agents, in one room or everywhere; the posts
  and pins that principal wrote from its device and service-account seats then
  reach those agents as the trusting principal's own text would. Cairn applies
  such grants and never grants trust itself. No other write to an agent
  exists.
- **I3 — Constraints are never summarized.** Pinned constraints are stored
  verbatim and restored verbatim after every compaction.
- **I4 — Each component stays inside one declared network boundary, and only
  the core reaches the model.** B0, the core (hooks, the MCP server, the
  kernel worker, the CLI, the TUI and everything that builds what reaches the
  model), opens no socket and makes no outbound connection. B1, machine: a
  component may listen on loopback only and connect nowhere else. B2, peer:
  encrypted, mutually authenticated connections to nodes the node's principal
  enrolled by key, off until that principal turns it on. B3, public: read-only
  publishing and outbound exchange with hosts the node's principal names, off
  until turned on. Managed policy can disable B1, B2 and B3. No component
  sends telemetry or depends on a central or third-party service. Data leaves
  the machine only when Claude receives recalled content through a tool call,
  or through a B2 or B3 component the node's principal turned on; whatever
  such a component brings in is untrusted (I2).
- **I5 — Bad data can be removed from circulation without destroying
  evidence.** Any event, span, run, writer or derived artifact can be
  quarantined from recall immediately on the node that records the quarantine,
  while the record stays intact for forensics. A quarantine reaches another
  node only as a quarantine request that node's principal applies.
- **I6 — No silent failures.** Every dropped, rejected, redacted, timed-out,
  or failed operation is counted, logged, and visible to the node's principal.
- **I7 — Configuration changes only on explicit instruction.** Cairn changes
  agent configuration only through explicit install and uninstall operations,
  shows the change first, and never overrides managed policy.
- **I8 — Isolation follows the principal.** All local state is bound to one
  principal's home with strict permissions, and Cairn refuses to operate on a
  home it does not own. Content another principal or node wrote is held as
  theirs: attributed to its seat key, untrusted, and never a way to widen what
  this principal's agents trust or recall.
- **I9 — Cairn never degrades the agent.** A Cairn failure never blocks or
  slows the agent beyond defined budgets. Cairn fails open, except where
  continuing would violate I2, I4, or I8.
- **I10 — Everything derived is rebuildable.** All derived state (indexes,
  landmarks, active pins, quarantine set, statuses, queues, evidence and proof
  classes, statistics) is a deterministic function of the set of writer logs a
  node holds and the node's own key set, independent of the order in which
  logs arrived. Rebuilding reproduces it exactly.
