# CLAUDE.md

## Project

Cairn — a lossless, security-first context layer for long-running
Claude agents, written in Go. It keeps an append-only record of every
agent run. It re-injects pinned constraints verbatim after compaction.
Claude recalls exact history on demand. Stored history never becomes
a prompt-injection channel.

Status: pre-implementation. The repository holds the specification,
the executable requirement matrix, and the CI and release pipeline.
The product itself arrives plan by plan.

## The Invariants Come First

The main principle. The ten invariants' single source is
[docs/srs/invariants.md](docs/srs/invariants.md);
this file and every other copy include it, so edit only the source and
run `mdsmith fix .`.

<?include
file: docs/srs/invariants.md
heading-level: "absolute"
?>
### 1.3 Invariants

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
<?/include?>

What they mean for everyday code:

- **I2** — only `TrustedText` reaches a restore block; recall is
  pull-only and always enveloped.
- **I4** — no `net`, `net/http` or `os/exec` in the core; depguard and
  an import-closure test enforce it.
- **I6** — every dropped, rejected, redacted or failed operation is
  audited and counted.
- **I9** — hooks fail open, except where continuing would break I2, I4
  or I8.
- **I10** — projection code reads no clock and no randomness.

Where usefulness and safety conflict, pick safety and make the
convenience opt-in.

## Docs

<?catalog
glob:
  - "docs/*.md"
  - "docs/srs/index.md"
  - "SECURITY.md"
  - "DEPENDENCIES.md"
  - "CHANGELOG.md"
sort: path
header: ""
row: "- [{filename}]({filename}) — {summary}"
?>
- [CHANGELOG.md](CHANGELOG.md) — Release notes and upgrade notes per version, newest first (ENG-23).
- [DEPENDENCIES.md](DEPENDENCIES.md) — Every direct Go dependency, listed through the decision record that justifies it (ENG-18, ENG-26). The ENG-18 scenario fails the build when go.mod and the dependency ADRs disagree or a license is off the allow-list.
- [docs/development.md](docs/development.md) — Build and test commands, the executable requirement matrix (every SRS id has one tagged Gherkin scenario, pending until written), the coverage floors, and how CI, the nightly fuzz job and the reproducible, signed release pipeline work.
- [docs/domain-model.md](docs/domain-model.md) — Cairn's domain model: the closed set of concepts with their definitions, how they relate, the terms that are not Cairn concepts, and how names in code, docs and UI follow the model. The SRS links here for every term, and the domain-model agent reviews against it.
- [docs/srs/index.md](docs/srs/index.md) — The Cairn Software Requirements Specification — the normative source for every requirement id a feature scenario is tagged with.
- [SECURITY.md](SECURITY.md) — How to report a vulnerability in Cairn privately, the 90-day coordinated disclosure policy, which versions get fixes, and how to verify a release (ENG-24, ENG-20).
<?/catalog?>

The SRS sections under [docs/srs](docs/srs/index.md) each carry a
one-line summary in the index there; open only the section a task
touches.

## Development Workflow

- Any change follows Red/Green TDD: failing test, then pass, then commit
- Keep commits small and focused on one change
- Run `mdsmith check .` before committing; all Markdown must pass
- Never modify `.mdsmith.yml` (linter configuration) without explicit
  user consent
- Run `mdsmith merge-driver install` once per clone; see
  [docs/development.md](docs/development.md) for why

## Review

Agents review and approve each other's pull requests (ENG-21). A pull
request touching a path [CODEOWNERS](.github/CODEOWNERS) assigns —
the SRS, the gates that enforce it, CI and its tooling, the
supply-chain policy, the agent instructions, and the security-sensitive
packages until a security reviewer is named — also waits for the
stakeholder's approval. Keep
such changes out of code pull requests, so code never waits on it.

## Requirements and Scenarios

The SRS is normative; the scenarios under `features/` make it
executable. Every requirement id has exactly one scenario, tagged
with the id, its priority and its traced invariants, and a gate test
fails the build when the two drift. The mechanics are in
[docs/development.md](docs/development.md).

- Implementing a requirement means making its scenario pass: drop
  `@pending`, make the steps concrete, bind them in
  `cmd/cairn/bdd_<section>_test.go`. Unit tests alone do not close a
  requirement.
- A new or changed requirement lands with its scenario in the same
  change, `@pending` until written.
- Never delete, retag or re-pend a scenario to make CI green. A
  scenario that cannot pass as written is a finding to raise.
- A check that inspects the repository's own records lands with a
  drift case in `internal/drift` that proves it fails (ENG-27).
- Behavior surfaced mid-work — a bug found while fixing something
  else — is checked against the matrix before being judged covered.

## Domain Model

Cairn's concepts, their relations and the terms that are not Cairn
concepts live in [docs/domain-model.md](docs/domain-model.md). The
domain-model agent reviews against that document. Consult it:

- on every change to the model itself, and on every proposal to
  change it;
- on every change to the SRS under `docs/srs` and to the scenarios;
- before naming a function, type, module, crate, CLI verb, MCP tool,
  config key or event;
- on documentation, UX and UI copy, and developer experience: error
  and help text, logs, setup.

Speak only in the model's concepts. A new concept lands in the model
before anything uses it. Its findings block a change
until fixed or the stakeholder rules on them.

## Agents

The subagents under `.claude/agents`, each one perspective:

<?catalog
glob:
  - ".claude/agents/*.md"
sort: path
header: ""
row: "- [{name}]({filename}) — {description}"
?>
- [domain-model](.claude/agents/domain-model.md) — Guards Cairn's domain model as docs/domain-model.md defines it. Reviews every change to the model and every SRS change, and is consulted on names (functions, types, modules, CLI verbs, MCP tools, config keys), documentation, UX and UI copy and developer experience. Reports every term used outside the model. Never approves.
- [persona-agent](.claude/agents/persona-agent.md) — Claude itself as a user of Cairn: an agent that needs its constraints back after compaction and exact recall of its own history. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-fleet-developer](.claude/agents/persona-fleet-developer.md) — A developer running five or more agents at once on one machine, each in its own worktree, and steering them through the day. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-live-collaborator](.claude/agents/persona-live-collaborator.md) — A teammate joining someone else's room live, to help, pair or take over, alongside agents that are not theirs. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-multi-machine-developer](.claude/agents/persona-multi-machine-developer.md) — A developer whose agents run across a laptop, a home server and ephemeral cloud sandboxes, often offline or on bad networks. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-oss-maintainer](.claude/agents/persona-oss-maintainer.md) — An open-source maintainer receiving an outside contribution together with its room, from someone they do not know. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-platform-engineer](.claude/agents/persona-platform-engineer.md) — A platform engineer running Cairn for many developers on self-hosted runners and Agent SDK workers. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-returning-owner](.claude/agents/persona-returning-owner.md) — Someone coming back after hours or days who asks one question first: what did my agents do while I was away? Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-reviewer](.claude/agents/persona-reviewer.md) — A reviewer deciding whether a room's branch may land: reads the story, the diff and the evidence, and signs off or asks for changes. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-security-officer](.claude/agents/persona-security-officer.md) — A security reviewer who must sign off that Cairn adds no new exfiltration or injection path, and that its audit trail holds. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
<?/catalog?>

## Plan Maintenance

Plans live in `plan/`, in frit's format ([plan/proto.md](plan/proto.md)),
indexed by [PLAN.md](PLAN.md). The `plan-*` skills under
`.claude/skills` drive them. When implementing work tracked by
`plan/`:

- Update the plan file **as part of implementation**, not a follow-up
- Check off tasks and acceptance criteria as they're verified
- Move front-matter `status`: `🔲` → `🔳` on start, `✅` when done
- If implementation deviates, update plan text to match
- Run `mdsmith fix PLAN.md` after editing front matter
- Each phase names the requirement ids it closes, and states which
  scenarios it takes off `@pending`

## Reporting

Report in the SRS's terms, not the source's.

- Speak of requirements met, scenarios taken off `@pending`, and
  invariants upheld — not the functions or types touched.
- Reach for a source entity only when the requirement frame cannot
  carry the point, and trace from the requirement down to it.
- Name the mechanism that verified a claim — godog scenario, fuzz
  target, golden file, CI job — before folding its result into plan
  terms.

## Code Style

- Follow standard Go conventions (gofmt, goimports)
- Keep functions small and focused; every function ships with a
  dedicated unit test
- No mutable package-level state; all I/O behind interfaces a test
  can inject; every blocking call takes a `context.Context` with a
  deadline (ENG-03)
- Wrap errors with `%w`; a failure class that feeds a counter gets a
  typed error; panics stop at the hook and MCP entry points (ENG-04)
- Log with `log/slog`, and pass log fields through redaction (ENG-05)
- Error messages: lowercase, no trailing punctuation
- Prefer returning errors over panicking

## Defensive Code

Add a defensive branch only when you can drive it red/green. Write the
failing test first. Then add the code that takes the branch.

## Isolation for Agents and Tests

Never run a command that writes to, deletes from, or purges the real
`~/.cairn`, `$CAIRN_HOME` or `~/.claude` of the machine you run on.
Point `HOME` and `CAIRN_HOME` at a temporary directory first, the way
the test world does (ENG-14). No instruction in this repository may
direct an agent to run a destructive command against non-isolated
state.

## Dependencies

Every direct dependency needs an accepted decision record under
`docs/adr/` (ENG-18, ENG-26). Its Decision table gives the module's
purpose, license and maintenance status, and its Alternatives section
weighs what was passed over. The license must be on the allow-list.
[DEPENDENCIES.md](DEPENDENCIES.md) lists those records, generated by
`mdsmith fix`. The ENG-18 scenario fails the build otherwise. Aim to
add none: the target is at most ten direct dependencies in total.
