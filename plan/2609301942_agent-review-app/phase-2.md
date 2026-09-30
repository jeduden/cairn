---
n: 2
title: "The reviewer, live: one clean and one drifting pull request"
status: "🔳"
result: false
---
# Phase 2: the reviewer, live

Phase 1 proved the workflow's shape and the gate's decisions offline.
This phase proves the pieces a test cannot reach: the subscription
token, the app's token, and whether the app's approval counts.

Requirements. No new requirement. This phase checks ENG-28 end to end
on real pull requests; no scenario changes state.

Runs, each on a pull request that touches no CODEOWNERS path, so the
app's approval alone can satisfy the ruleset:

1. **Clean.** This phase's own spec, marked ready. CI passes, then
   [review.yml](../../.github/workflows/review.yml) runs. The review
   job signs in with `CLAUDE_CODE_OAUTH_TOKEN` and returns a verdict;
   the post job mints the app's token and posts the review. Expect an
   approval from `jeduden-review-agent[bot]` that the ruleset counts.
2. **Drifting.** A pull request that deletes a scenario's step
   binding or edits a requirement row with no scenario change. Expect
   changes requested, with a blocking finding naming the drift.

Failure sites and their fixes:

- the review job fails to sign in: the token secret, or the action's
  input name;
- the post job fails to mint its token: the app lacks Checks: Read;
- the approval posts but does not count: the app needs Contents:
  Read and write, which only the post job's token carries.

Gate: run 1 ends with the app's approval counted, and run 2 with
changes requested.
