---
summary: >-
  Every direct Go dependency, listed through the decision record that
  justifies it (ENG-18, ENG-26). The ENG-18 scenario fails the build
  when go.mod and the dependency ADRs disagree or a license is off the
  allow-list.
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
`go.mod` and the ADRs. It fails `go test ./...` in any of these cases:

- a direct dependency is justified by no accepted ADR, or by more
  than one;
- an accepted ADR names a module `go.mod` no longer requires;
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

| ADR                                                        | Status   | Decision                                                                                                                                                                                                                                                             |
| ---------------------------------------------------------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [ADR-2609292234](docs/adr/ADR-2609292234-test-stack.md)    | accepted | The executable requirement matrix runs through godog and reads Gherkin through cucumber's messages types; testify asserts in every test. All three are test-only.                                                                                                    |
| [ADR-2609302341](docs/adr/ADR-2609302341-sqlite-driver.md) | proposed | The store opens SQLite through github.com/ncruces/go-sqlite3, a cgo-free wasm2go translation of SQLite. It links no network or process package, and it ships encrypting VFSes. Spike S2 measured it against modernc.org/sqlite at 10M events (ADR-07, OQ-03, OQ-04). |
<?/catalog?>

Every dependency listed is test-only today. None links into the
shipped `cairn` binary, and the import-closure test in
[cmd/cairn/imports_test.go](cmd/cairn/imports_test.go) keeps network
and process packages out of it.

Dev tools — golangci-lint and govulncheck — build from
[tools/go.mod](tools/go.mod), so they never enter this module's graph.
