---
n: 1
title: "Every crate has a license text, checked on each pull request"
status: "🔳"
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
- a crate with none fails, naming the crate and version, and the
  repository keeps no substitute text;
- a license file whose text does not match the declared license fails;
- the check reads only `Cargo.lock`, `cargo metadata` and fetched
  sources, and runs offline.

GREEN sites:

- a license-text check in `tooling/engineering`, sharing the
  linked-set reader ENG-18 uses;
- a command line entry;
- a CI step that runs it with `--locked` on every pull request
  (stakeholder path `.github/workflows/ci.yml`);
- a removal or replacement for any crate in the tree today that ships
  no license file.

Drift case (ENG-27). Add one to `tooling/drift/src/cases.rs`: an edit
that adds a lockfile package with no license text must make the ENG-30
scenario fail. ENG-27 requires it because the check inspects the
repository's records.

Gate. Run the built tool against the real tree in an isolated
`PARINN_HOME`, then against a scratch copy with one crate's license file
removed, and confirm the second run exits non-zero naming that crate.

## Progress

Done, red then green: the license-text check in
`tooling/engineering/src/license_texts.rs`, with the canonical texts of
every license `deny.toml` allows or grants by exception, and 17 unit
tests beside it: a fitting file passes; a crate with none fails naming
the crate and version; a text that does not fit the declared license
fails; the repository keeping a substitute text fails; the check reads
only `cargo metadata` output and fetched sources.

Held back, awaiting the stakeholder:

- The CLI entry, the CI step, the bound `@ENG-30` scenario and its drift
  case are not landed. Run over the real tree, the check fails on seven
  crates that ship no license text in their published source: in the
  root workspace `cucumber-codegen` 0.23.0, `peg-macros` 0.6.3,
  `peg-runtime` 0.6.3, `synthez-codegen` 0.4.0, `synthez-core` 0.4.0
  and `r-efi` 6.0.0; in the fuzz workspace `libfuzzer-sys` 0.4.13 (its
  NCSA part has no text). ENG-30 says such a crate is removed or
  replaced. The first five come with the cucumber test stack; `r-efi` is
  a Windows and UEFI crate of `getrandom`. Wiring the check now would
  turn CI red, and an exception list would contradict ENG-30. The
  stakeholder rules: replace the stack, amend ENG-30 to allow a
  recorded exception, or restrict the check to the crates linked on
  release targets.
- `@ENG-30` stays `@pending`.
