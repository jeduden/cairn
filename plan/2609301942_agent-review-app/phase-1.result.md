---
n: 1
title: "The review gate, the workflow and ENG-28"
status: "✅"
result: true
summary: >-
  ENG-28 added and off @pending. The review workflow runs the agent
  read-only after CI; the post job alone holds the app key and posts
  what the tested gate decides. Five drift cases guard it.
---
# Phase 1 handoff

## Done

- ENG-28 (P0) is in the SRS, and its scenario passes (godog
  `TestFeatures/^ENG-28:`).
- The gate approves only an approving verdict with no blocking
  finding, on the reviewed head, with the `CI` check green there. It
  posts nothing for a head that moved, and fails on a malformed
  verdict or a head whose CI did not pass.
- The drift suite catches all five review-workflow drifts: another
  trigger, a write permission or the app key in the agent's job, the
  post job running after a failed review, and the gate bypassed.

## Next

- The workflow runs only from main, so this change cannot review
  itself. Phase 2 runs it live once it merges.
- The agent needs `ANTHROPIC_API_KEY` as a repository secret. Without
  it the review job fails and nothing is posted.
- Check that the app's approval counts toward the ruleset. If it does
  not, give the app Contents read and write; only the post job holds
  its token.
