# Agent Notes

<!-- Included content comes from CLAUDE.md. Edit that
     file first, then run `mdsmith fix .` to propagate. -->

Instructions for AI coding agents (Codex, Copilot,
Claude).

<?include
file: CLAUDE.md
strip-frontmatter: "true"
?>
# CLAUDE.md

## Project

Cairn — a lossless, security-first context layer for long-running Claude agents.
It keeps an append-only record of every agent run, restores pins verbatim after
compaction, and lets Claude recall exact history on demand. Stored history never
becomes a prompt-injection channel.

Status: pre-implementation. The repository holds the specification, the
requirement matrix, and the CI and release pipeline, its tooling in Go; the
product, in Rust (ADR-2610050528), arrives plan by plan.

## The Invariants Come First

The main principle. The ten invariants' single source is
[docs/srs/invariants.md](docs/srs/invariants.md); this file and every other copy
include it, so edit only the source and run `mdsmith fix .`.

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
  events, the `harness_meta` events its hook handlers recorded, and the `user`
  events they recorded while the deployment mode is `interactive`, and, once
  PRV-10 ships, principal acts signed by a device key the agent's principal
  certified and posts and pins written from a device seat such a key certified,
  within that key's scope, a pin version a principal stamped, for that
  principal's own agents, and the posts and pins a trust grant of the agent's
  principal covers. Cairn writes to an agent only through a closed set of paths.
  Without a principal act: restore blocks (INJ-01, INJ-02, INJ-04), built only
  from qualifying pins, trusted structural fields and fixed text Cairn ships;
  and opt-in notices (INJ-10), compaction guidance (PIN-07) and the fixed
  templates of OWN-04 and OWN-07, each built only from fixed text Cairn ships,
  trusted structural fields and ids. On a principal act recorded at that time:
  through the harness's own input, only principal-typed text, a fixed template
  that references ids, or a post a principal endorsed exactly as shown inside
  the template of OWN-08. Under a delegation grant its principal recorded
  (OWN-23), and, for an agent of another principal, an acceptance grant that
  agent's principal recorded (OWN-26): a delegated task inside the fixed
  template of OWN-24. Once its requirements ship, a principal may also trust
  another principal by key for its own agents, in one room or everywhere; the
  pins that principal wrote from device seats one of its device keys certified
  then restore to those agents, and its posts reach them inside the fixed
  template of OWN-29. Cairn applies such grants and never grants trust itself.
  No other write to an agent exists.
- **I3 — Constraints are never summarized.** Every pin that restores is stored
  verbatim and restored verbatim after every compaction, or named by id and
  count when the budget omits it.
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
  component sends telemetry or depends on a central or third-party service. Data
  leaves the machine only as recalled content an agent receives through a tool
  call, or as what Cairn writes to an agent through I2's closed paths, both of
  which the harness sends to its model, or through a B2 or B3 component the
  node's principal turned on; whatever such a component brings in is trusted
  only as I2 allows.
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
- **I10 — Everything derived is rebuildable.** All derived artifacts (indexes,
  landmarks, active pins, quarantine set, statuses, queues, evidence and proof
  classes, stats) are a deterministic function of the set of writer logs a node
  holds and the node's own key set, independent of the order in which logs
  arrived. Rebuilding reproduces them exactly.
<?/include?>

What they mean for everyday code:

- **I2** — only `TrustedText` reaches a restore block; recall is pull-only and
  always enveloped.
- **I4** — no `net`, `net/http` or `os/exec` in the core; depguard and an
  import-closure test enforce it.
- **I6** — audit and count each dropped, rejected, redacted or failed operation.
- **I9** — hook handlers fail open unless that would break I2, I4 or I8.
- **I10** — code that builds derived artifacts reads no clock or randomness.

Where usefulness and safety conflict, pick safety; make convenience opt-in.

## Docs

<?catalog
glob:
  - "docs/*.md"
  - "docs/domain-model/index.md"
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
- [docs/domain-model/index.md](docs/domain-model/index.md) — Cairn's domain model: the closed set of concepts with their definitions, how they relate, the terms that are not Cairn concepts, and how names in code, docs and UI follow the model. The SRS links here for every term, and the domain-model agent reviews against it.
- [docs/srs/index.md](docs/srs/index.md) — The Cairn Software Requirements Specification — the normative source for every requirement id a feature scenario is tagged with.
- [SECURITY.md](SECURITY.md) — How to report a vulnerability in Cairn privately, the 90-day coordinated disclosure policy, which versions get fixes, and how to verify a release (ENG-24, ENG-20).
<?/catalog?>

The SRS sections under [docs/srs](docs/srs/index.md) each carry a one-line
summary in the index there; open only the section a task touches.

## Development Workflow

- Any change follows Red/Green TDD: failing test, then pass, then commit
- Keep commits small and focused on one change
- Run `mdsmith check .` before committing; all Markdown must pass
- Never modify `.mdsmith.yml` (linter configuration) without explicit user
  consent
- Run `mdsmith merge-driver install` once per clone (development.md says why)

## Review

Agents review and approve each other's pull requests (ENG-21). A pull request
touching a path [CODEOWNERS](.github/CODEOWNERS) assigns — the SRS, the gates
that enforce it, CI and its tooling, the supply-chain policy, the agent
instructions, and the security-sensitive packages until a security reviewer is
named — also waits for the stakeholder's approval. Keep such changes out of code
pull requests.

## Requirements and Scenarios

The SRS is normative; the scenarios under `features/` make it executable. Every
requirement id has exactly one scenario, tagged with the id, its priority and
its traced invariants, and a gate test fails the build when the two drift. The
mechanics are in [docs/development.md](docs/development.md).

- Implementing a requirement means making its scenario pass: drop `@pending`,
  make the steps concrete, bind them in `cmd/cairn/bdd_<section>_test.go`. Unit
  tests alone do not close a requirement.
- A new or changed requirement lands with its scenario in the same change,
  `@pending` until written.
- Never delete, retag or re-pend a scenario to make CI green. A scenario that
  cannot pass as written is a finding to raise.
- A check that inspects the repository's own records lands with a drift case in
  `internal/drift` that proves it fails (ENG-27).
- Behavior surfaced mid-work — a bug found while fixing something else — is
  checked against the matrix before being judged covered.

## Domain Model

Cairn's concepts, their relations and the terms that are not Cairn concepts live
in [docs/domain-model/](docs/domain-model/index.md), a hub and one file per
group. The domain-model agent reviews against them. Consult it:

- on every change to the model itself, and on every proposal to change it;
- on every change to the SRS under `docs/srs` and to the scenarios;
- before naming a function, type, module, crate, CLI verb, MCP tool, settings
  key or event;
- on documentation, UX and UI copy, and developer experience: error and help
  text, logs, setup.

Speak only in the domain model's concepts. A new concept lands there before
anything uses it. Its findings block a change until fixed or the stakeholder
rules on them.

## Agents

The subagents under `.claude/agents`, each one perspective:

<?catalog
glob:
  - ".claude/agents/*.md"
sort: path
header: ""
row: "- [{name}]({filename}) — {description}"
?>
- [domain-model](.claude/agents/domain-model.md) — Guards Cairn's domain model as docs/domain-model/ defines it. Reviews every change to the model and every SRS change, and is consulted on names (functions, types, modules, CLI verbs, MCP tools, other identifiers), documentation, UX and UI copy and developer experience. Reports every term used outside the model. Never approves.
- [persona-agent](.claude/agents/persona-agent.md) — Claude itself as a user of Cairn: an agent that needs its constraints back after compaction and exact recall of its own history. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-fleet-developer](.claude/agents/persona-fleet-developer.md) — A developer running five or more agents at once on one machine, each in its own worktree, and steering them through the day. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-live-collaborator](.claude/agents/persona-live-collaborator.md) — A teammate joining someone else's room live, to help, pair or take over, alongside agents that are not theirs. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-multi-machine-developer](.claude/agents/persona-multi-machine-developer.md) — A developer whose agents run across a laptop, a home server and ephemeral cloud environments, often offline or on bad networks. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-oss-maintainer](.claude/agents/persona-oss-maintainer.md) — An open-source maintainer receiving an outside contribution together with its room, from someone they do not know. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-platform-engineer](.claude/agents/persona-platform-engineer.md) — A platform engineer running Cairn for many developers on self-hosted runners and Agent SDK workers. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-returning-owner](.claude/agents/persona-returning-owner.md) — Someone coming back after hours or days who asks one question first: what did my agents do while I was away? Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-reviewer](.claude/agents/persona-reviewer.md) — A reviewer deciding whether a room's branch may land: reads the story, the diff and the evidence, and signs off or asks for changes. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-security-officer](.claude/agents/persona-security-officer.md) — A security reviewer who must sign off that Cairn adds no new exfiltration or injection path, and that its audit trail holds. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
<?/catalog?>

## Plan Maintenance

Plans live in `plan/`, in frit's format ([plan/proto.md](plan/proto.md)),
indexed by [PLAN.md](PLAN.md). The `plan-*` skills under `.claude/skills` drive
them. When implementing work tracked by `plan/`:

- Update the plan file **as part of implementation**, not a follow-up
- Check off tasks and acceptance criteria as they're verified
- Move front-matter `status`: `🔲` → `🔳` on start, `✅` when done
- If implementation deviates, update plan text to match
- Run `mdsmith fix PLAN.md` after editing front matter
- Each phase names the ids it closes and the scenarios it takes off `@pending`

## Reporting

Report in the SRS's terms, not the source's.

- Speak of requirements met, scenarios taken off `@pending`, and invariants
  upheld — not the functions or types touched.
- Reach for a source entity only when the requirement frame cannot carry the
  point, and trace from the requirement down to it.
- Name the mechanism that verified a claim — godog scenario, fuzz target, golden
  file, CI job — before folding its result into plan terms.

## Code Style

- Follow standard Go conventions (gofmt, goimports)
- Keep functions small and focused; each ships with a dedicated unit test
- No mutable package-level state; all I/O behind interfaces a test can inject;
  every blocking call takes a `context.Context` with a deadline (ENG-03)
- Wrap errors with `%w`; a failure class that feeds a counter gets a typed
  error; panics stop at the hook and MCP entry points (ENG-04)
- Log with `log/slog`, and pass log fields through redaction (ENG-05)
- Error messages: lowercase, no trailing punctuation
- Prefer returning errors over panicking

## Defensive Code

Add a defensive branch only after a failing test that takes it (red/green).

## Isolation for Agents and Tests

Never run a command that writes to, deletes from, or purges the real `~/.cairn`,
`$CAIRN_HOME` or `~/.claude` of the machine you run on. Point `HOME` and
`CAIRN_HOME` at a temporary directory first, the way the test world does
(ENG-14). No instruction in this repository may direct an agent to run a
destructive command against non-isolated state.

## Dependencies

Every direct dependency needs an accepted decision record under `docs/adr/`
(ENG-18, ENG-26). Its Decision table gives the module's purpose, license and
maintenance status, and its Alternatives section weighs what was passed over.
The license must be on the allow-list. [DEPENDENCIES.md](DEPENDENCIES.md) lists
those records, generated by `mdsmith fix`. The ENG-18 scenario fails the build
otherwise. Aim to add none: the target is at most ten direct dependencies in
total.
<?/include?>
