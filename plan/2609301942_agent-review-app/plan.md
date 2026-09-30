---
id: 2609301942
title: "Agent review through the reviewer app"
status: "🔳"
summary: >-
  An agent reviews each pull request after CI, from main's workflow,
  with read-only tools; a separate job holding the reviewer app's key
  posts the review a tested gate decides (ENG-21, ENG-28).
model: sonnet
depends-on: [2609292156]
---
# Agent review through the reviewer app

## Goal

Every pull request gets an agent's review once its CI passes. The
approval that lets it land comes from the reviewer app, never from
the agent itself, and only through a gate a test pins down.

## Context

ENG-21 lets agents approve each other's pull requests. Agent sessions
open pull requests under the stakeholder's account, so an approving
agent needs an identity that authors nothing. The stakeholder created
the GitHub App `jeduden-review-agent` for it and stored its key as the
repository secret `JEDUDEN_REVIEW_AGENT_KEY`.

What was reused:

- `anthropics/claude-code-action` runs the agent. It supports
  `workflow_run`, read-only tool lists and a JSON-schema verdict, so
  nothing custom runs the model.
- `actions/create-github-app-token` mints the app's token, scoped to
  this repository.
- The ENG-27 drift registry proves the new scenario's checks, and
  the engineering scenarios' checkout steps read the workflow.

Not reused: a YAML library. The ENG-28 steps read the workflow's
block layout line by line, as the other workflow scenarios do, so no
new dependency or ADR is needed (ENG-18).

The design and the alternatives weighed are in
[ADR-2609301941](../../docs/adr/ADR-2609301941-agent-review.md). Plan
2609292156's task 9, the reviewer protocol skill, lands here.

## Tasks

1. The gate, the workflow, the review skill and ENG-28 with its
   scenario and drift cases
2. A live dry run once the review workflow is on main: a seeded pull
   request with a known drift gets changes requested, and a clean one
   an approval that counts toward the ruleset

## Execution

| Phase | Model  | Gate                                                                      |
| ----- | ------ | ------------------------------------------------------------------------- |
| 1     | opus   | `@ENG-28` passes; the drift job catches its five review-workflow drifts   |
| 2     | sonnet | The seeded pull request gets changes requested; the clean one an approval |

## Phases

<?catalog
glob:
  - "phase-*.md"
  - "phase-*.result.md"
sort: numeric:n
header: |

  | # | Status | Phase |
  |---|--------|-------|
row-expr: |
  [if result {
    "|  | ↳ | \(summary) |"
  }, if !result {
    "| \(n) | \(status) | [\(title)](phase-\(n).md) |"
  }][0]
footer: |

?>

| #   | Status | Phase                                                                                                                                                                                         |
| --- | ------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | ✅     | [The review gate, the workflow and ENG-28](phase-1.md)                                                                                                                                        |
|     | ↳      | ENG-28 added and off @pending. The review workflow runs the agent read-only after CI; the post job alone holds the app key and posts what the tested gate decides. Five drift cases guard it. |
| 2   | 🔳     | [The reviewer, live: one clean and one drifting pull request](phase-2.md)                                                                                                                     |
<?/catalog?>

## Acceptance Criteria

- [x] Scenario @ENG-28 passes, off `@pending`
- [x] The drift suite catches every review-workflow drift
- [ ] A seeded drifting pull request gets changes requested
- [ ] A clean pull request gets an approval the ruleset counts
- [x] All tests pass: `go test -race ./...`
- [x] `mdsmith check .` is clean
- [x] `go tool -modfile=tools/go.mod golangci-lint run` is clean
