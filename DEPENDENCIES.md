---
summary: >-
  Every direct dependency, Rust and Go, listed through the decision
  record that justifies it (ENG-18, ENG-26). The ENG-18 scenario fails
  the build when the manifests and the dependency ADRs disagree or a
  license is off the allow-list.
---
# Dependencies

SRS requirement ENG-18 asks for three things. Every direct dependency
is justified by an accepted decision record (ADR). Its license is
Apache-2.0, MIT, BSD or ISC. The direct set stays at ten or fewer.

Each dependency decision lives in its own file under
[docs/adr](docs/adr/), per ENG-26. The file's Decision table gives
every module's purpose, license and maintenance status, and its
Alternatives section weighs what was passed over. The list below is
generated from those files by `mdsmith fix`, so it is never edited by
hand.

The `ENG-18` scenario in
[features/engineering.feature](features/engineering.feature) reads
`go.mod`, `cargo metadata` and the ADRs. It fails the build in any
of these cases:

- a direct dependency is justified by no accepted ADR, or by more
  than one;
- an accepted ADR names a module no manifest requires any more;
- a row leaves a column empty, or its license is off the allow-list;
- this file does not list a dependency ADR.

Updates arrive as reviewed dependabot pull requests; see
[.github/dependabot.yml](.github/dependabot.yml).

<?catalog
glob: "docs/adr/*.md"
where: 'scope: "dependencies"'
sort: path
header: |

  | ADR | Status | Decision |
  | --- | ------ | -------- |
row: "| [{id}]({filename}) | {status} | {summary} |"
footer: |

?>

| ADR                                                          | Status     | Decision                                                                                                                                                                                                                                     |
| ------------------------------------------------------------ | ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [ADR-2609292234](docs/adr/ADR-2609292234-test-stack.md)      | superseded | The executable requirement matrix runs through godog and reads Gherkin through cucumber's messages types; testify asserts in every test. All three are test-only.                                                                            |
| [ADR-2610101442](docs/adr/ADR-2610101442-rust-test-stack.md) | accepted   | The repository tooling in Rust runs the scenarios through cucumber-rs and reads JSON through serde and serde_json; testify asserts in the Go tests that remain. All four serve tests and tooling only; none links into a shipped executable. |
<?/catalog?>

Every direct dependency serves tests and repository tooling today.
The SQLite driver's ADR stays proposed until M1 adds the module. None
links into the shipped `cairn` binary, and the import-closure test in
[cmd/cairn/imports_test.go](cmd/cairn/imports_test.go) keeps network
and process packages out of it.

[deny.toml](deny.toml) holds every crate in the tree, direct or not,
to the same allow-list, with each exception named, and to crates.io as
the only source; CI runs cargo-deny and cargo-audit over it (ENG-16).
Updates arrive weekly from dependabot for Cargo, Go modules and
actions.

Dev tools are no dependency. golangci-lint and govulncheck build from
[tools/go.mod](tools/go.mod), so they never enter this module's graph.
CI installs cargo-deny, cargo-audit and cargo-llvm-cov at exact
versions.
