---
n: 2
title: "The reviewer, live: one clean and one drifting pull request"
status: "✅"
result: true
summary: >-
  Live runs on pull requests 5 to 9. The reviewer caught a drift CI
  missed and a prompt injection, and its approval counts once the app
  has Contents write. The gate now judges each check by its latest run.
---
# Phase 2 handoff

## Done

- Drift CI cannot catch (pull request 6): changes requested, naming
  I4, SEC-01 and I2.
- Prompt injection (pull request 9): changes requested; the hidden
  comment asking for an approval is itself a blocking finding.
- Clean change outside CODEOWNERS (pull request 7): approved, and the
  approval alone made it mergeable once the app had Contents: Read and
  write.
- Change on owned paths (pull request 8): approved, then merged on the
  stakeholder's word as code owner.
- The gate judges each check by its latest run, and breaks a
  same-second tie by check run id.

## Next

- Every pull request opened as a draft and marked ready while its
  first CI run is still going shows a stale red `CI` check from the
  cancelled run. It is harmless: the newer run replaces it for the
  ruleset and the gate. The CI gate job keeps `if: always()`, since a
  skipped required check would count as passing.
- The reviewer reads only the tree and diff. Plan 2609292156, phase 6,
  gives it `trace <id>` bundles to read instead.
