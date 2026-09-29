---
id: ADR-2609292234
title: "The test stack: godog, cucumber messages and testify"
status: accepted
scope: dependencies
summary: >-
  The executable requirement matrix runs through godog and reads
  Gherkin through cucumber's messages types; testify asserts in every
  test. All three are test-only.
---
# ADR-2609292234: The test stack

## Context

The SRS is executable. Every requirement id has exactly one Gherkin
scenario, and a gate keeps the two in step
([docs/development.md](../development.md)). The gate reads each
scenario's tags and lines. Every test needs assertions with precise
failure output. Nothing chosen here may link into the shipped binary
(I4).

## Decision

| Module                                | Purpose                                                                                | License | Maintenance                                |
| ------------------------------------- | -------------------------------------------------------------------------------------- | ------- | ------------------------------------------ |
| `github.com/cucumber/godog`           | Runs the Gherkin scenarios under features/ that trace each SRS requirement (test only) | MIT     | Active; Cucumber project, regular releases |
| `github.com/cucumber/messages/go/v34` | Gherkin AST types the scenario gate reads tags and lines from (test only)              | MIT     | Active; released alongside godog           |
| `github.com/stretchr/testify`         | assert and require helpers in every test (test only)                                   | MIT     | Active; the de-facto Go assertion library  |

## Alternatives

- Plain table-driven Go tests instead of godog. They lose the
  requirement-readable scenario text the matrix is built on.
- Nothing instead of the messages types. godog exposes its parsed
  features only through them.
- The standard library alone instead of testify. Failure output gets
  verbose and less precise.

## Consequences

Three direct dependencies, all test-only, against ENG-18's target of
ten. A major version bump changes a module path, such as
`messages/go/v34`. The ENG-18 scenario then fails until the path in
the table above follows it. The decision itself stands.
