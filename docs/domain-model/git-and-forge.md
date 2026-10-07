---
title: "Git and forge"
order: "08"
summary: >-
  Repositories, branches and commits, the forge, results with their evidence and proof classes, qualified links, comparisons, outcomes and verdicts.
---
# Git and forge

- **Repository**: A git repository, identified by its **repository identity**, a
  bound root commit, the same on every node with a git clone, shallow clones
  included, else a provisional node-local identity bound later (LANE-02). The
  first bind is a structural event Cairn records at the first hook event;
  binding by hand or rebinding is a widening principal act. It has remotes and
  may carry room trailers, bundles as git refs and the git carrier's segments.
- **Branch**: A git branch, identified by repository identity, remote URL and
  branch name. A branch with no remote has a provisional node-local identity,
  rebound when it is pushed.
- **Commit**: A git commit.
- **Worktree**: A git working tree on a node, where a run edits a branch; its
  repository's **git directory** is git's own store of objects and refs.
- **Forge**: The service that keeps remotes, pull requests, reviews and branch
  protection. It approves and lands; Cairn does neither, and its **forge
  reports** count as `asserted` (LANE-08).
- **Pull request**: The forge's review object for a branch.
- **Check**: A command executed, or expected to execute, on a tree, with its
  exit status once it ends; its **check state** is one of §9.7.2's. A check is
  expected when one of the intent's criteria names its command (LANE-20); a
  check the forge reports, one of its **required checks** included, is a forge
  report, `asserted`, not a result.
- **Result**: What a room's seats established: a check passing or failing on a
  **tree** (git's snapshot of a commit's files), or a claim stated in text. It
  names the intent version and carries one evidence class; it is derived from
  the events it rests on and addressed by the event recording its exit status or
  its claim's text.
- **Evidence**: The checks, CI attestations or text a result rests on.
- **Evidence class**: Of a result, ranked: `claim` (text, or tool output alone)
  < `own check` < `witness check` < `CI attested` (LANE-05).
- **Own check**: A check the hook handlers recorded on the node of the run that
  made the edits, run on the latest worktree checkpoint plus the recorded edits;
  otherwise its result is marked `unbound` and counts as a `claim`.
- **Witness check**: A check re-run through the launcher on a fresh checkout of
  the exact commit, with network denied, by a node whose **git identity** (the
  author and commit-signing identities its git configuration sets) authored no
  commit on the branch since it left its base (a **commit author** is git's
  author of a commit, never a seat). Where the platform cannot deny network,
  Cairn refuses the witness check (OWN-18).
- **CI attestation**: A check's exit status for the exact commit, signed by a CI
  key the room's owner enrolled, recorded in the room so every principal sees
  the same class (LANE-22); a result resting on one is `CI attested`.
- **Landing**: Git or the forge merging commits into the repository's default
  branch or a branch the forge protects. Cairn never lands anything, and a
  landing is never a verdict. A branch lands when its head lands.
- **Landing link**: The qualified link from a landed commit to a room, carrying
  a proof class, derived by Cairn from the record alone: the clone facts it
  reads (landed commits, patch ids, trees) are recorded as structural events.
- **Proof class**: Of a landing link. Proven: `same commit`, `same patch`, `same
  tree`. Not proven: `likely`, `asserted`, and `not proven` with a reason
  (LANE-06). `asserted` is also the mark on forge reports and room trailers.
- **Room trailer**: The `Cairn-Room:` line in a commit message, naming a room;
  Cairn adds one to every commit on a room's branch (LANE-28). Each
  `Cairn-Link:` line is a trailer link. It counts as `asserted` until proven
  (LANE-28).
- **Qualified links**: Every link is named by what it connects: a branch link
  (room to branch), a pull-request link (branch to its pull request, derived by
  Cairn from forge bridge events), a criterion link (result to criterion), a
  range link (post or pin to an address range), a parent link (subagent run to
  parent run), a delegation link (delegating run to the delegate's run, under a
  delegation grant), an invite link (which connects nothing: it carries an
  access token), a landing link and a trailer link (commit to a pin, post or
  marked range). "Link" never stands alone, except as the name of the room act
  that adds one range link, branch link or criterion link.
- **Comparison**: A side-by-side view of two branches: each one's exposure,
  results and evidence. It picks no winner.
- **Outcome**: What a room's runs have produced so far: the heads of the
  branches it names, their results and evidence, as verdicts assess them.
- **Presentation**: What a seat put in the outcome window, such as a dev
  server's URL (inert text the person opens in their own browser), a build
  artifact, a file or a diff, with the seat and branch.
- **Verdict**: A person's `met`, `not met` or `needs changes` on one criterion:
  a `verdict` pin, recorded as a neutral principal act (OWN-27) by any person
  whose device seat in the room has the pin capability, bound to the intent
  version, the heads of every branch the room names, and the results and
  evidence shown. It goes stale when any of them changes. It is never edited: a
  newer verdict of the same person on the same criterion supersedes it, and
  unpinning one is neutral. Cairn never derives one; service accounts contribute
  evidence instead.
