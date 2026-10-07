---
title: "Git and forge"
order: "08"
summary: >-
  Repositories, branches and commits, the forge, results with their evidence and proof classes, qualified links, comparisons, outcomes and verdicts.
---
# Git and forge

- **Repository**: A git repository, identified by its **repository identity**, a
  bound root commit, the same on every node holding a clone, shallow clones
  included (LANE-02), together with its remotes. It may carry room trailers,
  bundles as git refs and the git carrier's segments.
- **Branch**: A git branch, identified by repository identity, remote URL and
  branch name. A branch with no remote has a provisional node-local identity,
  rebound when it is pushed.
- **Commit**: A git commit.
- **Worktree**: A git working tree on a node, where a run edits a branch.
- **Forge**: The service that keeps remotes, pull requests, reviews and branch
  protection. It approves and lands; Cairn does neither, and its reports count
  as `asserted` (LANE-08).
- **Pull request**: The forge's review object for a branch.
- **Check**: A command and its exit status, bound to a tree; its **check state**
  is one of §9.7.2's.
- **Result**: What a room's runs established: a check passing or failing on a
  **tree** (git's snapshot of a commit's files), or a claim stated in text. It
  names the intent version and carries one evidence class.
- **Evidence**: The checks, attestations or text a result rests on.
- **Evidence class**: Of a result, ranked: `claim` (text only) < `own check` <
  `witness check` < `CI attested` (LANE-05).
- **Own check**: A check the hook handlers recorded on the node of the run that
  made the edits, run on the latest worktree checkpoint plus the recorded edits;
  otherwise it is marked `unbound` and counts as a `claim`. Its other marks,
  such as `outside intent` and `from checkpoint`, are §9.7's.
- **Witness check**: A check re-run through the launcher on a fresh checkout of
  the exact commit, by a node whose **git identity** (the author and
  commit-signing identities its git configuration sets) authored no commit in
  the range (a **commit author** is git's author of a commit, never a seat).
- **CI attested**: A check result for the exact commit, signed by a CI key.
- **Landing**: Git or the forge merging commits into a protected branch. Cairn
  never lands anything, and a landing is never a verdict.
- **Landing link**: The link from a landed commit to a room, carrying a proof
  class, derived by Cairn.
- **Proof class**: Of a landing link. Proven: `same commit`, `same patch`, `same
  tree`. Not proven: `likely`, `asserted`, and `not proven` with a reason
  (LANE-06).
- **Room trailer**: The `Cairn-Room:` line on a commit made on a room's branch.
  Each `Cairn-Link:` line is a trailer link. It counts as `asserted` until
  proven (LANE-28).
- **Qualified links**: Every link is named by what it connects: a branch link
  (room to branch), a pull-request link (branch to its pull request), a
  criterion link (result to criterion), a range link (post or pin to an address
  range), a parent link (subagent run to parent run), a delegation link
  (delegating run to the delegate's run, under a delegation grant), an invite
  link (carrying an access token), a landing link and a trailer link. "Link"
  never stands alone, except as the name of the room act that adds one qualified
  link.
- **Comparison**: A side-by-side view of two branches: each one's exposure,
  results and evidence. It picks no winner.
- **Outcome**: What a room's runs have produced so far: the heads of the
  branches it names, their results and evidence, as verdicts assess them.
- **Presentation**: What a seat put in the outcome window, such as a dev server,
  an artifact, a file or a diff, with the seat and branch.
- **Verdict**: A person's `met`, `not met` or `needs changes` on one criterion:
  a `verdict` pin, recorded as a principal act (OWN-27) by any person whose
  device seat in the room has the pin capability, bound to the intent version,
  the heads of every branch the room names, and the results and evidence shown.
  It goes stale when any of them changes. Cairn never derives one; service
  accounts contribute evidence instead.
