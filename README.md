# Cairn

A complete record of what your Claude agents did: exact recall for the
agents, one view for you, and no automatic path from untrusted content
to the model.

## First, for every run

- **"My agent forgot what I told it after compaction."** Pin it as a
  constraint. After every compaction it comes back word for word, or by
  id when over budget. The agent can recall exactly what it did hours
  ago, wrapped as untrusted data.
- **"We run fifty runners under managed policy."** Install shows its
  diff first, each principal's record stays in its own home, every
  failure is audited and counted, and you can purge content.

## Then, with the room view

The room view is optional: your browser, a terminal or the CLI.

- **"Which of my five agents needs me?"** It ranks what waits on you
  and spots runs in two rooms editing one file; with the launcher,
  answer, steer or stop them there.
- **"What happened while I was away?"** Catch up shows what changed,
  each line linked to its events and every gap shown; no model writes
  it. A head receipt kept off the machine shows any rewrite up to it.
  Search finds Saturday's decision.
- **"I review what my agents built."** The diff comes with the runs
  behind it and what backs each result: a bare claim or a check. You
  approve on your forge, and the landed commit links back to its room,
  marked proven or not.

## Later

- **"My agents run on three machines and in cloud sandboxes."** Your
  machines peer by key with no central service, keep working offline
  and sync after; a short-lived cloud node syncs as it runs, any gap
  shown.
- **"A teammate wants to help."** They join your room live by invite
  and post; you choose what reaches your agent, and they see what you
  chose.
- **"A stranger sent a pull request with its room."** It arrives as a
  redacted plain file, checked against their commit-signing identity
  and never trusted; your agents can only recall it.

## What reaches your agents

In every step, Cairn writes to an agent only your pins, an index of its
history, fixed text and what your own acts send or allow, such as an
endorsement or a trust grant. Other history reaches it only by recall.
The core opens no socket, and every component beyond it stays off
until you turn it on.

## Status

Specified, not built yet; [§12](docs/srs/12-delivery-plan.md) has the
milestones behind "First", "Then" and "Later". This repository holds:

- the [Software Requirements Specification](docs/srs/index.md), the
  normative source, and the [domain model](docs/domain-model/index.md)
  that defines its terms;
- one Gherkin scenario per requirement under `features/`, almost all
  `@pending`, kept in step with the SRS by a gate test;
- the `cairn` core executable, with `cairn version` and `cairn help`
  so far;
- CI, a nightly fuzz job, an agent review of every pull request and a
  signed release pipeline, every gate written in Rust;
- the delivery plans in [PLAN.md](PLAN.md).

<?include
file: docs/srs/invariants.md
heading-level: "2"
?>
## 1.3 Invariants

The invariants are Cairn's contract. Every requirement serves at least one of
them. A change that would break an invariant is a design change requiring
security review and a new major version, not a bug fix.

- **I1 — Nothing is lost.** Every event an agent saw or produced remains
  recoverable by its stable address, (writer, seq), across any number of
  compactions and agent runs, on every node that holds its writer's log. The
  only exceptions are secrets removed by redaction before storage or on import,
  and data the node's principal explicitly purges, or removes under a retention
  policy that principal or managed policy sets. Every exception is recorded.
- **I2 — No automatic path from untrusted content to the model.** Content that
  originates outside the trusted sources (tool output, web, MCP servers, files,
  the model's replies, and anything another node or principal produced, except
  the agent's principal's acts signed by a device key it certified, posts and
  pins written from a device seat such a key certified, a pin version that
  principal stamped, or what a trust grant of that principal covers) reaches the
  model only inside an untrusted-data envelope when the agent explicitly calls a
  recall tool, or through one of the closed paths below that a principal act
  names. The trusted sources are this node's own `operator` and structural
  events (hook observations only as PRV-08 allows), the `harness_meta` events
  its launcher recorded, and, while the deployment mode is `interactive` and no
  run on the node leaves R3 or R8 open, the `user` events matched to a prompt
  (PRV-08), and, once PRV-10 ships, principal acts signed by a device key the
  agent's principal certified and posts and pins written from a device seat such
  a key certified, within that key's scope, a pin version a principal stamped,
  for that principal's own agents, and the posts and pins a trust grant of the
  agent's principal covers. Cairn writes to an agent only through a closed set
  of paths. Without a principal act: restore blocks (INJ-01, INJ-02, INJ-04),
  built only from qualifying pins, trusted structural fields and fixed text
  Cairn ships; and opt-in notices (INJ-10) and the fixed templates of OWN-04 and
  OWN-07, each built only from fixed text Cairn ships, trusted structural fields
  and ids. On a principal act recorded at that time: through the harness's own
  input, only principal-typed text, a fixed template that references ids, or a
  post a principal endorsed exactly as shown inside the template of OWN-08.
  Under a delegation grant its principal recorded (OWN-23), and, for an agent of
  another principal, an acceptance grant that agent's principal recorded
  (OWN-26): a delegated task inside the fixed template of OWN-24. Once its
  requirements ship, a principal may also trust another principal by key for its
  own agents, in one room or everywhere; the pins that principal wrote from
  device seats one of its device keys certified then restore to those agents,
  and its posts reach them inside the fixed template of OWN-29. Cairn applies
  such grants and never grants trust itself. No other write to an agent exists.
- **I3 — Constraints are never summarized.** Every pin that restores is stored
  verbatim and restored verbatim after every compaction, or named by id and
  count when the budget or run ambiguity (INJ-05) omits it.
- **I4 — Each component stays inside one declared network boundary, and only the
  core builds what reaches the model.** B0, the core (hook handlers, the MCP
  server, the kernel worker, the CLI, the TUI and everything that builds what
  reaches the model), opens no socket and makes no outbound connection. B1,
  machine: a component may listen only on loopback or on a local endpoint only
  the same OS user can reach, and connect nowhere else; the launcher carries
  into the harness's input only text the core built. B2, peer: encrypted,
  mutually authenticated connections to nodes and paired phones the node's
  principal enrolled by key, off until that principal turns it on. B3, public:
  read-only publishing and outbound exchange with hosts the node's principal
  names, off until turned on. Managed policy can disable B1, B2 and B3. No
  component sends telemetry or depends on a central or third-party service.
  Cairn's components send data off the machine only as recalled content an agent
  receives through a tool call, or as what Cairn writes to an agent through I2's
  closed paths, both of which the harness sends to its model, or through a B2 or
  B3 component the node's principal turned on; whatever such a component brings
  in is trusted only as I2 allows.
- **I5 — Bad data can be removed from circulation without destroying the
  record.** Any event, span, run, writer or derived artifact can be
  quarantined from recall immediately on the node that records the quarantine,
  while the record stays intact for forensics. A quarantine reaches another
  node only as a quarantine request that node's principal applies.
- **I6 — No silent failures.** Every dropped, rejected, redacted, timed-out,
  or failed operation is counted, logged, and visible to the node's principal.
- **I7 — Harness configuration changes only on explicit instruction.** Cairn
  changes harness configuration only through explicit install and uninstall
  operations, shows the change first, and never overrides managed policy or the
  harness's managed settings.
- **I8 — Isolation follows the principal.** All local state is bound to one
  principal's home with strict permissions, and Cairn refuses to operate on a
  home that does not belong to the OS user running it. Content another principal
  wrote, or another node wrote other than this principal's acts signed by a
  device key it certified and its posts and pins written from a device seat such
  a key certified, is kept as theirs: attributed to its seat key, and untrusted
  unless this principal's own stamp or trust grant covers it. On its own it
  never makes this principal's agents trust it, nor extends their recall.
- **I9 — Cairn never degrades the agent.** A Cairn failure never blocks or
  slows the agent beyond defined budgets. Cairn fails open, except where
  continuing would violate I2, I4, or I8.
- **I10 — Everything derived is rebuildable.** All derived artifacts (the search
  index, landmarks, active pins, quarantine set, room state, origins, trust
  levels, run and integrity statuses, queues, evidence and proof classes, stats)
  are a deterministic function of the set of writer logs a node holds and the
  node's own key set, independent of the order in which logs arrived. Rebuilding
  reproduces their content exactly, whatever their storage layout.
<?/include?>

## Development

```sh
cargo test --workspace                          # tests and scenarios
cargo test -p scenario --test bdd               # the requirement matrix
cargo clippy --workspace --all-targets          # lint
cargo run -p coverage                           # coverage per test layer
mdsmith check .                                 # Markdown
```

[docs/development.md](docs/development.md) has the full reference.
[CLAUDE.md](CLAUDE.md) has the working rules for agents and people
alike.

## Security

Report vulnerabilities privately; see [SECURITY.md](SECURITY.md).

## License

MIT; see [LICENSE](LICENSE). Open question OQ-09 in the SRS proposes
Apache-2.0 for its patent grant.
