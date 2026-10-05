# Cairn

A lossless, security-first context layer for long-running Claude
agents, written in Go.

Long agent runs compact, and compaction forgets. Cairn keeps an
append-only, provenance-tagged record of every agent run. It re-injects
the constraints you pinned, verbatim, after every compaction. Claude
recalls exact history on demand through MCP tools, always wrapped as
untrusted data. It does this without opening a new attack surface: no
automatic path from untrusted content to the model, a daemonless core
that never touches the network, every other component off until you
start it, strict per-person isolation.

## Status

Pre-implementation. This repository holds:

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
  compactions and agent runs, on every node that holds its writer's log.
  The only exceptions are secrets removed by redaction before storage or
  on import, and data an operator explicitly purges or expires by
  policy. Every exception is recorded.
- **I2 — No automatic path from untrusted content to the model.**
  Content that originates outside the trusted boundary (tool output,
  web, MCP servers, files, assistant text, and anything another writer,
  node or person produced) reaches the model only when Claude explicitly
  calls a recall tool, and always inside an untrusted-data envelope. The
  trusted boundary is this node's own `operator` and structural events
  and, once PRV-10 ships, owner acts signed by a device key the owner
  certified, within its scope. Cairn writes to an agent only through a
  closed set of paths. Without an owner act: restore blocks of pins
  (INJ-01, INJ-02), opt-in notices (INJ-10), and the fixed templates of
  OWN-04 and OWN-07, each built only from trusted structural fields and
  ids. On an owner act recorded at that time: through the harness's own
  input, only owner-typed text, a fixed template that references ids, or
  a post a principal endorsed exactly as shown inside the template of
  OWN-08. Under a delegation grant its principal recorded (OWN-23): a
  delegated task inside the fixed template of OWN-24. Once its
  requirements ship, a person may also trust a poster by key for their
  own agents, in one room or everywhere; that poster's posts then reach
  those agents as the person's own text would. Cairn applies such grants
  and never grants trust itself. No other write to an agent exists.
- **I3 — Constraints are never summarized.** Pinned constraints are
  stored verbatim and re-injected verbatim after every compaction.
- **I4 — Each component stays inside one declared network boundary, and
  only the core reaches the model.** B0, the core (hooks, the MCP
  server, the kernel worker, the CLI and everything that builds what
  reaches the model), opens no socket and makes no outbound connection.
  B1, machine: a component may listen on loopback only and connect
  nowhere else. B2, peer: encrypted, mutually authenticated connections
  to nodes the owner enrolled by key, off until the tenant turns it on.
  B3, public: read-only publishing and outbound exchange with hosts the
  owner names, off until turned on. Managed policy can disable B1, B2
  and B3. No component sends telemetry or depends on a central or
  third-party service. Data leaves the machine only when Claude receives
  recalled content through a tool call, or through a B2 or B3 component
  the tenant turned on; whatever such a component brings in is untrusted
  (I2).
- **I5 — Bad data can be removed from circulation without destroying
  evidence.** Any event, span, run, writer or derived artifact can
  be quarantined from recall immediately on the node that records the
  quarantine, while the record stays intact for forensics. A quarantine
  reaches another node only as a request that node's operator applies.
- **I6 — No silent failures.** Every dropped, rejected, redacted,
  timed-out, or failed operation is counted, logged, and visible to
  operators.
- **I7 — Configuration changes only on explicit instruction.** Cairn
  changes agent configuration only through explicit install and
  uninstall operations, shows the change first, and never overrides
  managed policy.
- **I8 — Isolation follows the tenant.** All local state is bound to one
  tenant's home with strict permissions, and Cairn refuses to operate on
  a home it does not own. Content another tenant or node wrote is held
  as theirs: attributed to its writer's key, untrusted, and never a way
  to widen what this tenant's agents trust or recall.
- **I9 — Cairn never degrades the agent.** A Cairn failure never blocks
  or slows the agent beyond defined budgets. Cairn fails open, except
  where continuing would violate I2, I4, or I8.
- **I10 — Everything derived is rebuildable.** All derived state
  (indexes, landmarks, active pins, quarantine set, statuses, queues,
  evidence, proof and gate verdicts, statistics) is a deterministic
  function of the set of writer logs a node holds and the node's own key
  set, independent of the order in which logs arrived. Rebuilding
  reproduces it exactly.
<?/include?>

## Development

```sh
go test ./...                                   # tests and scenarios
go test ./cmd/cairn -run TestFeatures -v        # the requirement matrix
go tool -modfile=tools/go.mod golangci-lint run # lint
mdsmith check .                                 # Markdown
```

[docs/development.md](docs/development.md) has the full reference.
[CLAUDE.md](CLAUDE.md) holds the working rules for agents and people
alike.

## Security

Report vulnerabilities privately; see [SECURITY.md](SECURITY.md).

## License

MIT; see [LICENSE](LICENSE). Open question OQ-09 in the SRS proposes
Apache-2.0 for its patent grant.
