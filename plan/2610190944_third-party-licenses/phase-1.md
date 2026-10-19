---
n: 1
title: "Every crate has a license text, checked on each pull request"
status: "🔲"
result: false
---
# Phase 1: every crate has a license text, checked on each pull request

Requirements. Closes ENG-30. Takes the `@ENG-30` scenario off
`@pending`. ENG-31 stays pending.

BDD coverage: ENG-30 inspects the repository checkout. Its scenario is
bound in [the BDD steps](../../tooling/scenario/tests/bdd/),
beside ENG-20's. No cross-host or lease behavior is involved.

RED spec, in `tooling/engineering` (unit tests beside the module, over
fixture `cargo metadata` output):

- a crate whose source holds a license file passes;
- a crate with none and no kept text for its name and version fails,
  naming the crate and version;
- a kept text for that name and version makes it pass;
- a license file whose text does not match the declared license fails;
- the check reads only `Cargo.lock`, `cargo metadata` and fetched
  sources, and runs offline.

GREEN sites:

- a license-text check in `tooling/engineering`, sharing the
  linked-set reader ENG-18 uses;
- a command line entry;
- a CI step that runs it with `--locked` on every pull request
  (stakeholder path `.github/workflows/ci.yml`);
- the kept texts for any crate the tree needs today.

Drift case (ENG-27). Add one to `tooling/drift/src/cases.rs`: an edit
that adds a lockfile package with no license text, or removes a kept
text, must make the ENG-30 scenario fail. ENG-27 requires it because the
check inspects the repository's records.

Gate. Run the built tool against the real tree in an isolated
`CAIRN_HOME`, then against a scratch copy with one crate's license file
removed, and confirm the second run exits non-zero naming that crate.
