---
summary: >-
  Every direct Go dependency with its purpose, license, maintenance
  status and the alternatives considered (ENG-18). The ENG-18 scenario
  fails the build when go.mod gains a dependency with no row here or a
  license off the allow-list.
---
# Dependencies

SRS requirement ENG-18 asks for three things. Every direct dependency
is justified here. Its license is Apache-2.0, MIT, BSD or ISC. The
direct set stays at ten or fewer.

The `ENG-18` scenario in
[features/engineering.feature](features/engineering.feature) reads
`go.mod` and this table. It fails `go test ./...` when a direct
dependency has no row, leaves a column empty, or carries a license
off the allow-list. Updates arrive as reviewed dependabot pull
requests; see [.github/dependabot.yml](.github/dependabot.yml).

Every dependency below is test-only today. None links into the
shipped `cairn` binary, and the import-closure test in
[cmd/cairn/imports_test.go](cmd/cairn/imports_test.go) keeps network
and process packages out of it.

| Module                                | Purpose                                                                                | License | Maintenance                                | Alternatives                                                                    |
| ------------------------------------- | -------------------------------------------------------------------------------------- | ------- | ------------------------------------------ | ------------------------------------------------------------------------------- |
| `github.com/cucumber/godog`           | Runs the Gherkin scenarios under features/ that trace each SRS requirement (test only) | MIT     | Active; Cucumber project, regular releases | Plain table-driven Go tests lose the requirement-readable scenario text         |
| `github.com/cucumber/messages/go/v34` | Gherkin AST types the scenario gate reads tags and lines from (test only)              | MIT     | Active; released alongside godog           | None: godog exposes its parsed features only through these types                |
| `github.com/stretchr/testify`         | assert and require helpers in every test (test only)                                   | MIT     | Active; the de-facto Go assertion library  | The standard library alone, at the cost of verbose, less precise failure output |

Dev tools — golangci-lint and govulncheck — build from
[tools/go.mod](tools/go.mod), so they never enter this module's graph.
