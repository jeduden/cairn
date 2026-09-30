---
id: ADR-2609301941
title: "Agent review through a reviewer app and a deterministic gate"
status: accepted
summary: >-
  An agent reviews each pull request in a workflow main defines, with
  read-only tools and a read-only token. A separate job holding the
  reviewer app's key posts the review, and a tested gate decides it.
---
# ADR-2609301941: Agent review through a reviewer app

## Context

ENG-21 lets agents approve each other's pull requests. Agent sessions
open pull requests under the stakeholder's account, and GitHub never
counts an author's own approval. So an approving agent needs an
identity that authors nothing. An approval is also the one output
that lets a change land. The reviewing agent reads text the author
wrote, so that text can try to talk it into approving (I2). The rest
of the pipeline must hold even when the agent is fooled or wrong.

## Decision

- A GitHub App, `jeduden-review-agent`, is the reviewer. It is
  installed on this repository only, and its token may write pull
  request reviews and read checks, nothing more.
- `.github/workflows/review.yml` runs on `workflow_run` after CI
  completes. GitHub runs it as main defines it, so no pull request
  can rewrite its own reviewer.
- The review job checks out main, and the pull request's tree beside
  it as data. It runs `anthropics/claude-code-action`, pinned by SHA,
  with only the read, glob and grep tools and a read-only token. The
  agent follows main's `.claude/skills/review/SKILL.md` and answers
  with a verdict that must fit a JSON schema.
- The post job needs the review job and alone reads the app's key. It
  runs `cmd/review-gate`, which approves only an approving verdict
  with no blocking finding, on the head the agent read, with the CI
  check green there. Otherwise it requests changes. A stale head posts
  nothing, and anything the gate cannot decide fails the job.
- ENG-28 states these rules. Its scenario reads the workflow and runs
  the gate over a decision table, and drift cases prove it (ENG-27).

## Alternatives

- **`pull_request` trigger.** The simplest wiring, but GitHub runs
  the workflow as the pull request defines it. A branch could edit
  its reviewer to approve itself, or read the app's key.
- **`pull_request_target` trigger.** Runs main's definition too, but
  fires before CI finishes. The review would race CI, and the gate
  would have to wait on it.
- **Give the agent the app token.** One job, fewer moving parts. A
  fooled agent could then approve directly; the gate would be advice.
- **The Actions token (`github-actions[bot]`).** Needs no app, but
  its approvals count only if the repository lets Actions approve
  pull requests. That setting lets any workflow approve.
- **Only a human reviewer.** Holds I2 best, but no human reads most
  diffs at the rate many agents write them; ENG-21 chose agents.

## Consequences

- The review runs only once the definition is on main. The pull
  request that adds it is reviewed another way.
- A pull request touching a CODEOWNERS path still needs the
  stakeholder. The app's approval never replaces that review.
- The workflow needs `ANTHROPIC_API_KEY` and `JEDUDEN_REVIEW_AGENT_KEY`
  as repository secrets. Without either, the run fails and posts
  nothing.
- Fork pull requests are not reviewed here.
