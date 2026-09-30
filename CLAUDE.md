# CLAUDE.md

## Project

Cairn — a lossless, security-first context layer for long-running
Claude agents, written in Go. It keeps an append-only record of every
session. It re-injects pinned constraints verbatim after compaction.
Claude recalls exact history on demand. Stored history never becomes
a prompt-injection channel.

Status: pre-implementation. The repository holds the specification,
the executable requirement matrix, and the CI and release pipeline.
The product itself arrives plan by plan.

## The Invariants Come First

The main principle. Cairn's contract is ten invariants, I1–I10, in
[docs/srs/01-introduction.md](docs/srs/01-introduction.md). Every
requirement serves at least one. The ones that shape everyday code:

- **I2** — no automatic path from untrusted content to the model.
  Only `TrustedText` reaches a restore block; recall is pull-only and
  always enveloped.
- **I4** — Cairn never talks to the network. No `net`, `net/http` or
  `os/exec` in shipped code; depguard and an import-closure test
  enforce it.
- **I6** — no silent failures. Every dropped, rejected, redacted or
  failed operation is audited and counted.
- **I9** — Cairn never degrades the agent. Hooks fail open, except
  where continuing would break I2, I4 or I8.
- **I10** — everything derived is rebuildable from the record, so
  projection code reads no clock and no randomness.

A change that would break an invariant is a design change that needs
security review and a new major version, not a bug fix. Where
usefulness and safety conflict, pick safety and make the convenience
opt-in.

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
- [docs/development.md](docs/development.md) — Build and test commands, the executable requirement matrix (every SRS id has one tagged Gherkin scenario, pending until written), the coverage floors, and how CI, the nightly fuzz run and the reproducible, signed release pipeline work.
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
