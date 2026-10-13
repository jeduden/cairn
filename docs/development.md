---
summary: >-
  Build commands, static analysis and supply-chain checks, and how CI,
  the nightly fuzz job and the signed release pipeline work. Testing
  has its own page, docs/testing.md.
---
# Development

Build and release reference, plus the mechanics behind the rules in
[CLAUDE.md](../CLAUDE.md). How Cairn is tested, and how the tests are
measured, is in [testing.md](testing.md).

## Build & test commands

The repository is one Cargo workspace: the product's crates under
`crates/`, today the `cairn` core executable, and the repository
tooling under `tooling/`. Its Rust release is fixed in
[rust-toolchain.toml](../rust-toolchain.toml) (ENG-01), and rustup
installs it on the first `cargo` command.

- `cargo test --workspace` — every test, scenarios included
- `cargo run -p cairn -- version` — the core executable, built for
  this machine
- `cargo test -p scenario --test bdd` — every scenario, pending ones
  counted
- `cargo test -p scenario --test bdd -- --tags @REC-03` — one scenario
  by its requirement id
- `cargo test -p srs --test gates` and `cargo test -p scenario --test
  gate` — the gates alone
- `cargo run -p coverage` — line coverage per test layer and per crate,
  held to the floors (see [testing.md](testing.md))
- `cargo fmt --all --check` and `cargo clippy --workspace --all-targets
  -- -D warnings` — layout and lint (ENG-16)
- `cargo deny check licenses bans sources` and `cargo audit` — the
  supply chain (ENG-16), with [deny.toml](../deny.toml)'s policy
- `mdsmith check .` — lint the Markdown; `mdsmith fix .` rewrites
  what it can, including the generated catalogs

## Static analysis

The workspace's lints in [Cargo.toml](../Cargo.toml) forbid `unsafe`,
and refuse `unwrap`, `expect` and `panic!` outside tests
([clippy.toml](../clippy.toml) lets a test use them). CI runs clippy
with every warning denied, and rustfmt.
[deny.toml](../deny.toml) holds every crate in the tree to ENG-18's
license allow-list, each exception named, and to crates.io as the
only source. cargo-audit refuses a crate with a known vulnerability.

Crates may only depend on the crates they declare (ENG-02). Each
crate's `Cargo.toml` holds a `cairn` metadata table with three keys:

- `responsibility`: the one thing the library crate does.
- `may-depend-on`: the workspace crates it may depend on, an empty
  list included.
- `may-dev-depend-on`: the workspace crates its tests may depend on.

The architecture check (`engineering::architecture`) reads every
workspace manifest through `cargo metadata`. It fails naming both
crates when a dependency is not declared, and when a declaration names
a crate that is no workspace member. It also fails when a library
crate declares no responsibility or several, or when two crates
declare the same one. The tooling crates fall under the same table.

One rule needs no declaration. A product crate under `crates/` never
depends on a tooling crate under `tooling/` as a normal or build
dependency; only its tests may, as declared. The manifests are the
stakeholder's paths in CODEOWNERS, so changing a direction waits for
the stakeholder. The browser room view client, a TypeScript package,
does not exist yet (M7); its dependency boundary joins this check
when it lands.

The core executable's reach is held two ways (SEC-01, ENG-16). Its
own [clippy.toml](../crates/cairn/clippy.toml) refuses the
`std::net` socket types and `std::process::Command` in its code, and
[its reach test](../crates/cairn/tests/reach.rs) refuses any crate
linked into it, so neither can arrive through a dependency. Both are
the build-time half of SEC-01; the network-deny sandbox half is
ENG-12, still pending.

## CI, nightly and release

Every workflow lives in `.github/workflows`. Every action is fixed
to a commit SHA with its version in a trailing comment, and dependabot
proposes the bumps after a seven-day cooldown.

[ci.yml](../.github/workflows/ci.yml) runs on every push and pull
request to `main`. It runs every test and scenario, clippy and
rustfmt, cargo-deny and cargo-audit, and the drift suite, and the
release build of every target (NFR-10, CON-02). It also runs
`mdsmith check .` and zizmor over the workflows themselves. A final
job named `CI` passes only when every other job passed. It is the one
check the branch and release-tag rulesets require, because GitHub
names a required check after a job, never after the workflow.

[nightly.yml](../.github/workflows/nightly.yml) is where
cargo-fuzz runs every target under `fuzz/` (ENG-07). There are none
yet; the job fails the night a `fuzz/` crate appears until the plan
that adds it pins the nightly and the run. The crash-consistency test
(ENG-06), the concurrency soak (ENG-09) and the live Claude Code
contract test (ENG-17) join it as their plans land.

[build.yml](../.github/workflows/build.yml) is the release build.
CI runs it on every change, and the release workflow runs it too. One
x86_64 Linux runner builds `linux/amd64` and `linux/arm64`, linked
statically against musl with the rust-lld Rust ships (CON-02), and
`windows/amd64` with mingw-w64, pinned to Ubuntu 24.04's exact
packages. One arm64 macOS runner builds `darwin/arm64`.
`--remap-path-prefix` keeps the builder's paths out. `CAIRN_VERSION`
and `CAIRN_COMMIT` stamp the version and the commit (ENG-01). Each
binary then runs once on its own platform and must report what it
was stamped with. Each target is built once: the release build is not
checked for reproducibility.

[review.yml](../.github/workflows/review.yml) reviews each pull
request from a branch of this repository once its CI passes (ENG-21,
ENG-28). It skips drafts, so a pull request is reviewed when it is
marked ready, which reruns CI. The agent runs on the stakeholder's
Claude subscription, not an API key billed per model token. It triggers on
`workflow_run`, so GitHub runs it as `main` defines it, never as the
pull request does. It also triggers on `issue_comment`, which GitHub
runs as `main` defines it too: a code owner's comment on a pull
request whose CI passed reviews it again. Consent to a stakeholder
path is such a comment, so the review that reads it follows at once.
The review job first runs main's `review-bundle`
over main's tree and the pull request's: it writes the review bundle
the agent reads, sets the review's class and makes the deterministic
findings
(see Review below). The agent then runs with read-only tools on the
pull request's tree, following
[the review skill](../.claude/skills/review/SKILL.md). The agent only
writes a review outcome. The post job alone has the reviewer app's
key. It runs the `review-gate` crate's executable, which approves only an
approving review outcome with no blocking finding, its own or a
deterministic one, on the reviewed head, with `CI` green there;
otherwise it requests changes.
[ADR-2609301941](adr/ADR-2609301941-agent-review.md) records why.

[release.yml](../.github/workflows/release.yml) runs from the Actions
"Run workflow" button with a version like `v0.1.0`. A pushed tag is
deliberately not the trigger: a failed build would leave a public tag
pointing at nothing. The workflow takes these steps:

1. Validate the version and run the whole suite.
2. Run the release build above, with the version stamped, and stop
   when any binary fails to report it.
3. Generate an SPDX SBOM, attest build provenance and the SBOM, and
   sign `checksums.txt` keylessly with cosign (ENG-20).
4. Create the tag and publish the release, only now that everything
   above succeeded.

GitHub's build-provenance attestation meets SLSA Build Level 2. Level
3 needs the build isolated in a reusable workflow; that step keeps
ENG-20 pending. [SECURITY.md](../SECURITY.md) has the verification
commands.

## Review

The SRS is enforced at the cheapest layer that can decide each rule.
About nine in ten requirements name a test as their verification
method, so their scenario, once off `@pending`, enforces them on every
pull request without an agent.

What CI decides, on every pull request:

- the requirement–scenario matrix, Appendix B and C and the persona
  agents (the gate tests), and every drift case (ENG-27);
- dependencies and their ADRs (ENG-18) and the decision records
  (ENG-26);
- rustfmt and clippy, with each product crate's bans on sockets and
  program starts (I4) and on clocks and hash order (I10), and the
  reach test;
- the crates' declared dependency directions and responsibilities
  (ENG-02);
- no mutable global state in a product crate (ENG-03);
- coverage per test layer (ENG-11), and every live scenario.

What `review-bundle` decides, comparing the pull request with main:

- a scenario live on main deleted or put back on `@pending`: blocking;
- an invariant, or a row ENG-29 marks for I2 review, changed without
  both an accepted ADR in the same change that names ENG-29 and each
  guarded id, and a comment of the stakeholder on the pull request:
  blocking;
- a clippy ban lifted at a call site, or a diff header it cannot read:
  a nit, and an unread header makes the review deep.

The review class sets the agent's model, turn budget and tools. A change
holding only Markdown outside the stakeholder's paths, the domain model,
plans and scenarios is light. Any path CODEOWNERS assigns makes it
deep, and only a deep review may consult the read-only reviewer
subagents. Anything else, an empty diff included, is standard.
The review bundle quotes only the touched rows, their scenarios and
the invariants, a few thousand model tokens where the SRS holds
hundreds of thousands. A re-review starts from the diff since the reviewer app's
last review, found by the app's own login. The pull request's agent
instructions move out of the agent's directory, so none loads; the
agent reads them as data. The posted review names the gate as its
author, states the agent's verdict, and keeps the agent's findings,
the deterministic ones and failed checks under their own headings.

What is left to the agent is judgement: whether the code and the step
bindings do what each touched row says, new paths to the model (I2),
new reach (I4), content of another principal (I8), errors and seams
no lint sees, and the domain model's terms. Persona reviews belong to
plans and SRS changes (the `persona-review` skill), not to each pull
request. The nightly job takes what is too slow for a pull request:
fuzzing, crash consistency, soak and benchmarks, as their plans land.

## The plan catalog and its merge driver

Work is tracked as plans under `plan/`, indexed by
[PLAN.md](../PLAN.md), in frit's format: see
[plan/proto.md](../plan/proto.md) and the `plan-*` skills under
`.claude/skills`. Run `mdsmith merge-driver install` once per clone.
`PLAN.md` is a generated catalog, so two branches adding a plan
conflict on it by construction; the driver regenerates it during a
merge instead of leaving conflict markers.
