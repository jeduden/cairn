# Cairn

A lossless, security-first context layer for long-running Claude
agents, written in Rust.

Long agent runs compact, and compaction forgets. Cairn keeps an
append-only, provenance-tagged record of every agent run. It restores
the constraints you pinned, verbatim, after every compaction. The
agent recalls exact history on demand through MCP tools, always wrapped
as untrusted data. It does this without opening a new attack surface: no
automatic path from untrusted content to the model, a daemonless core
that never touches the network, every other component off until you
start it, strict per-principal isolation.

## Status

Pre-implementation. This repository contains:

- the [Software Requirements Specification](docs/srs/index.md), the
  normative source;
- one Gherkin scenario per requirement under `features/`, almost all
  `@pending`, kept in step with the SRS by a gate test;
- CI (build, race tests, coverage, lint, govulncheck, Markdown,
  workflow audit), a nightly fuzz job, and a reproducible, signed
  release pipeline;
- the delivery plans in [PLAN.md](PLAN.md).

`cairn version` is the only command so far.

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
  only exceptions are secrets removed by redaction before storage or on
  import, and data the node's principal explicitly purges, or removes under a
  retention policy that principal or managed policy sets. Every exception is
  recorded.
- **I2 — No automatic path from untrusted content to the model.** Content that
  originates outside the trusted sources (tool output, web, MCP servers,
  files, the model's replies, and anything another node or principal produced,
  except what the agent's principal signed through a device key it certified
  or a trust grant of that principal covers) reaches the model only inside an
  untrusted-data envelope when the agent explicitly calls a recall tool, or
  through one of the closed paths below that a principal act names. The
  trusted sources are this node's own `operator`, `harness_meta` and
  structural events, its `user` turns while the deployment mode is
  `interactive`, and, once PRV-10 ships, principal acts, posts and pins signed
  through a device key the agent's principal certified, within its scope, a
  pin version a principal stamped, for that principal's own agents, and the
  posts and pins a trust grant of the agent's principal covers. Cairn writes
  to an agent only through a closed set of paths. Without a principal act:
  restore blocks of pins (INJ-01, INJ-02), opt-in notices (INJ-10), and the
  fixed templates of OWN-04 and OWN-07, each built only from trusted
  structural fields and ids. On a principal act recorded at that time: through
  the harness's own input, only principal-typed text, a fixed template that
  references ids, or a post a principal endorsed exactly as shown inside the
  template of OWN-08. Under a delegation grant its principal recorded
  (OWN-23): a delegated task inside the fixed template of OWN-24. Once its
  requirements ship, a principal may also trust another principal by key for
  its own agents, in one room or everywhere; the pins that principal wrote
  from its device seats then restore to those agents, and its posts reach them
  inside the fixed template of OWN-29. Cairn applies such grants and never
  grants trust itself. No other write to an agent exists.
- **I3 — Constraints are never summarized.** Every pin that restores is stored
  verbatim and restored verbatim after every compaction, or named by id and
  count when the budget omits it.
- **I4 — Each component stays inside one declared network boundary, and only
  the core builds what reaches the model.** B0, the core (hook handlers, the
  MCP server, the kernel worker, the CLI, the TUI and everything that builds
  what reaches the model), opens no socket and makes no outbound connection.
  B1, machine: a component may listen only on loopback or on a local endpoint
  only the same OS user can reach, and connect nowhere else; the launcher
  carries into the harness's input only text the core built. B2, peer:
  encrypted, mutually authenticated connections to nodes and paired phones the
  node's principal enrolled by key, off until that principal turns it on. B3,
  public: read-only publishing and outbound exchange with hosts the node's
  principal names, off until turned on. Managed policy can disable B1, B2 and
  B3. No component sends telemetry or depends on a central or third-party
  service. Data leaves the machine only as recalled content an agent receives
  through a tool call, or as what Cairn writes to an agent through I2's closed
  paths, both of which the harness sends to its model, or through a B2 or B3
  component the node's principal turned on; whatever such a component brings
  in is untrusted (I2).
- **I5 — Bad data can be removed from circulation without destroying
  evidence.** Any event, span, run, writer or derived artifact can be
  quarantined from recall immediately on the node that records the quarantine,
  while the record stays intact for forensics. A quarantine reaches another
  node only as a quarantine request that node's principal applies.
- **I6 — No silent failures.** Every dropped, rejected, redacted, timed-out,
  or failed operation is counted, logged, and visible to the node's principal.
- **I7 — Configuration changes only on explicit instruction.** Cairn changes
  harness configuration only through explicit install and uninstall
  operations, shows the change first, and never overrides managed policy or
  the harness's managed settings.
- **I8 — Isolation follows the principal.** All local state is bound to one
  principal's home with strict permissions, and Cairn refuses to operate on a
  home that does not belong to the OS user running it. Content another
  principal wrote, or another node wrote outside what this principal signed
  through a device key it certified, is kept as theirs: attributed to its seat
  key, and untrusted unless this principal's own stamp or trust grant covers
  it. On its own it never makes this principal's agents trust it, nor extends
  their recall.
- **I9 — Cairn never degrades the agent.** A Cairn failure never blocks or
  slows the agent beyond defined budgets. Cairn fails open, except where
  continuing would violate I2, I4, or I8.
- **I10 — Everything derived is rebuildable.** All derived artifacts (indexes,
  landmarks, active pins, quarantine set, statuses, queues, evidence and proof
  classes, statistics) are a deterministic function of the set of writer logs
  a node holds and the node's own key set, independent of the order in which
  logs arrived. Rebuilding reproduces them exactly.
<?/include?>

## Development

```sh
go test ./...                                   # tests and scenarios
go test ./cmd/cairn -run TestFeatures -v        # the requirement matrix
go tool -modfile=tools/go.mod golangci-lint run # lint
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
