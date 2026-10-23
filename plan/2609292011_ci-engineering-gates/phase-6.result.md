---
n: 6
title: "Main's protection and CODEOWNERS (ENG-21)"
status: "🔳"
result: true
summary: >-
  Main's ruleset is committed and checked against the live one nightly, CODEOWNERS is checked against the paths that enforce the SRS, a required Security approvals check counts the two approvals security-sensitive crates need, and the ENG-21 scenario is off pending.
---
## Handoff

What landed:

- `.github/rulesets/main.json`: main's ruleset in GitHub's export
  format. No direct push, no bypass actor, no deletion or history
  rewrite, signed commits, `CI` required, one approval that is not the
  last pusher's, stale approvals dismissed, a code owner's approval
  where CODEOWNERS names one.
- The `protection-check` executable and a `protection` job in the
  nightly workflow: it fetches the live ruleset with the workflow's token
  and fails when the live one is weaker. Reading a ruleset works with
  that token. Its bypass actors do not show, since GitHub gives them
  only to a caller with write access to the ruleset.
- CODEOWNERS checks derived from the SRS: the stakeholder and the
  security reviewer from the index, the security-sensitive crates from
  ENG-11, the enforcing classes from the ENG-21 row. The current
  CODEOWNERS conforms; nothing in it was changed.
- Ten drift cases, six integration gate tests for the ruleset and
  CODEOWNERS, and an end-to-end test of the executable.
- After the merge, the two live tag rulesets were committed beside
  main's: `releases-semver` keeps each `v*.*.*` tag fixed on a signed
  commit that passed `CI`, and `no-other-tags-allowed` refuses every
  other tag. The nightly job compares every file under
  `.github/rulesets` with the live ruleset of its name, and
  `protection-check` also refuses a live ruleset on another target or
  with a `do_not_` flag switched on. The live `no-other-tags-allowed`
  included no ref, so it applied to nothing; the committed one includes
  `~ALL` but the release tags, and a gate test refuses a ruleset with
  no include. The stakeholder applies it on GitHub again.

Scenario: `@ENG-21` is off `@pending`. Two approvals on the
security-sensitive crates, with the security reviewer as one, is not a
number a ruleset or CODEOWNERS holds, so a required check enforces it:

- `review-bundle approvals` reads the pull request's changed files and
  reviews and main's CODEOWNERS. A change reaching a crate directory
  CODEOWNERS gives whole needs two distinct approvals of the current
  head, not the author's, one from that crate's owner; a later review,
  a stale head or a team owner does not count. Eleven unit tests, five
  command tests.
- `security-approvals.yml` runs as main defines it
  (`pull_request_target`, and `workflow_run` after the no-op
  `security-approvals-trigger.yml` that a review starts: a
  `pull_request_review` run would use the pull request's own copy), reads
  the pull request, its files and reviews through the API without
  checking anything out, and reports a check run named `Security
  approvals` on the head. Its token can write checks, not approve or
  write code. A review on a fork's pull request recounts on its next
  push.
- Residual risk: the ruleset matches the check by name, so any workflow
  on a pull request with `checks: write` could post it; the CODEOWNERS
  review of `.github/workflows/` is what holds that, not the check.
- `.github/rulesets/main.json` requires the check; a gate test holds
  the workflow, and six drift cases (the ruleset dropping the check,
  the workflow running as the pull request defines it, counting on a
  review, checking it out or no longer counting, and the trigger
  acting with a token) were each shown to fail with their message.
  The full drift suite needs mdsmith and ran only in CI.

For the stakeholder:

- Approve this change (it touches CODEOWNERS-gated paths) and apply
  `.github/rulesets/main.json` on GitHub again, which adds the
  `Security approvals` required check; until then the nightly
  `protection` job is red. Apply it after the workflow is on main, or
  every pull request waits on a check nothing reports.
- Name the security reviewer; until then the stakeholder owns the
  security-sensitive crates, and since GitHub never counts an author's
  own approval, a pull request the stakeholder authors there cannot
  reach two approvals.
- Decide how a pull request the stakeholder authors on an owned path
  merges: GitHub never counts an author's own approval, and the ruleset
  has no bypass actor.
- AI-generated labelling is in the ENG-21 row but in no scenario step
  and no mechanism; it is still open.
