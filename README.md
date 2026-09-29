# Cairn

A lossless, security-first context layer for long-running Claude
agents, written in Go.

Long sessions compact, and compaction forgets. Cairn keeps an
append-only, provenance-tagged record of every session. It re-injects
the constraints you pinned, verbatim, after every compaction. Claude
recalls exact history on demand through MCP tools, always wrapped as
untrusted data. It does this without opening a new attack surface: no
automatic path from untrusted content to the model, no network, no
daemon, strict per-tenant isolation.

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

## The ten invariants

| #   | Invariant                                                            |
| --- | -------------------------------------------------------------------- |
| I1  | Nothing is lost                                                      |
| I2  | No automatic path from untrusted content to the model                |
| I3  | Constraints are never summarized                                     |
| I4  | Cairn never talks to the network                                     |
| I5  | Bad data can be removed from circulation without destroying evidence |
| I6  | No silent failures                                                   |
| I7  | Configuration changes only on explicit instruction                   |
| I8  | Isolation follows the tenant                                         |
| I9  | Cairn never degrades the agent                                       |
| I10 | Everything derived is rebuildable                                    |

See [docs/srs/01-introduction.md](docs/srs/01-introduction.md) for
what each one means.

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
