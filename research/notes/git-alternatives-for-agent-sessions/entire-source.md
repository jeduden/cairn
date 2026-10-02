# Entire Checkpoints CLI: source verification

Source: github.com/entireio/cli, branch `main` at commit
`89c26160877a27a5017dc0bd614768c463e793bd` (2026-10-01), nightly tag
`v0.11.4-nightly.202610020627.89c261608`. Latest stable tag: v0.11.3
(2026-09-25). History was fetched back to 2026-08-15 for commit archaeology.
Nothing was executed. Code was only read. All paths below are relative to the
repository root. `cli/` stands for `cmd/entire/cli/`.

## 1. Storage layout

### Two backends, chosen per repo

- The backends are `git-branch` (legacy) and `git-refs`. They are selected by
  `checkpoints.primary.type` in `.entire/settings.json`, or overridden by
  `ENTIRE_CHECKPOINTS_PRIMARY`.
- An absent block means `git-branch`. A first-time `entire enable` writes
  `git-refs` without asking (`cli/setup.go:1665-1680`;
  `docs/architecture/ref-checkpoint-backend.md:185-186`).
- Built-in registry: `cli/checkpoint/registry.go`. Topology resolution:
  `cli/checkpoint/open.go:60` (`PrimaryIsRefs`), `:96` (`Open`).
- Reads route by ID kind across both backends
  (`cli/checkpoint/routing_store.go`):
  - a ULID is read from refs only;
  - a hex ID is read from refs first, then from the branch.

### git-branch backend

- Branch: `entire/checkpoints/v1`, i.e. `refs/heads/entire/checkpoints/v1`
  (`cli/paths/paths.go:50`). It is pushed as a single ref
  (`cli/checkpoint/persistent_refs.go:29-36`).
- Sharding inside the tree is `<id[:2]>/<id[2:]>/`: the FIRST two characters
  (`cli/checkpoint/persistent.go:58-63`, using `CheckpointID.Path()`).
- Every condensation adds one commit on the branch tip, which splices the
  checkpoint subtree into the root tree (`persistent.go:63-110`).
- There is a second orphan branch, `entire/trails/v1`, for "trails"
  (`paths.go:54`).

### git-refs backend

- One ref per checkpoint: `refs/entire/checkpoints/<shard>/<id>`
  (`cli/checkpoint/refs_naming.go:16`, `:27-32`).
- Shard = **last two characters** of the ID, for both ID formats
  (`cli/checkpoint/id/id.go:134-140`).
- `ParseRef` rejects:
  - a shard that does not match the ID;
  - extra path segments.

  The shard comparison is case-insensitive because of a macOS/NTFS
  shard-folding bug (`refs_naming.go:53-72`).
- The ref points at a commit whose tree root IS the checkpoint
  (`metadata.json`, `0/`, `1/`, `tasks/...`). The first write is an orphan
  commit. Later writes (backfills, more sessions) add parented commits, so each
  checkpoint carries its own small history (`cli/checkpoint/refs_store.go:145-160`,
  `:249-277`).

### ID formats

`id.go:26-46`:

- Legacy: 12 lowercase hex characters from 6 bytes of `crypto/rand`
  (`id.go:166-172`).
- ULID: 26 Crockford base32 characters via `oklog/ulid` with `crypto/rand`
  entropy (`id.go:182-188`).
- Deterministic ULIDs for import: SHA-256(seed) entropy (`id.go:195-206`).
- Which format gets minted depends on the backend: a ULID iff the primary is
  git-refs (`cli/checkpoint/generate.go:21-28`).

Real commit trailers in Entire's own repo carry ULIDs, for example
`Entire-Checkpoint: 01M2R9THXKFTAP54AE30PE9YM0` on commit 87df65461.

### Files inside one checkpoint

Constants: `cli/paths/paths.go:30-46`. Layout:
`docs/architecture/sessions-and-checkpoints.md:525-543`.

```
metadata.json            CheckpointSummary (root)
<n>/metadata.json        per-session Metadata (n = 0,1,2... stable per session_id)
<n>/full.jsonl           agent transcript, sanitized + redacted (chunked .001,.002 over the blob cap)
<n>/transcript.jsonl     compacted full session; this checkpoint's slice starts at compact_transcript_start
<n>/prompt.txt           checkpoint-scoped user prompts (redacted)
<n>/content_hash.txt     sha256 of full.jsonl (dedup short-circuit only)
<n>/assets/manifest.json + blobs   externalized images (opt-in), stored UNREDACTED
tasks/<tool-use-id>/task.json, agent-<id>.jsonl   subagent records
```

Schemas live in `api/checkpoint/metadata.go`:

- `Metadata` at `:395-493`. Fields include:
  - `cli_version`, `checkpoint_id`, `session_id`, `strategy`, `created_at`, `branch`, `commit_sha`;
  - `checkpoints_count`, `save_step_count`, `files_touched`, `agent`, `model`, `turn_id`;
  - `checkpoint_transcript_start`, `compact_transcript_start`, `token_usage`, `skill_events`;
  - `session_metrics`, `summary`, `initial_attribution`, `prompt_attributions`, `kind`;
  - `review_skills`, `review_prompt`, `investigate_run_id`, `investigate_topic`.
- `SessionFilePaths` at `:520-536`.
- `CheckpointSummary` at `:557-589`. Fields:
  - `checkpoint_id`, `strategy`, `branch`, `commit_sha`, `checkpoints_count`, `files_touched`;
  - `sessions[]` (the path pointers), `token_usage`, `combined_attribution`;
  - `has_review`, `has_investigation`, `imported`.

### How objects and refs are written

The design is a hybrid. go-git writes objects, and the git binary moves refs.

- Blobs, trees and commits are built in go-git. `CreateCommit` encodes an
  `object.Commit` and calls `repo.Storer.SetEncodedObject`
  (`cli/checkpoint/persistent.go:2661-2693`).
- Commit message: subject `Checkpoint: <id>`, then the trailers
  `Entire-Session`, `Entire-Strategy`, `Entire-Agent` and `Ephemeral-branch`
  (`persistent.go:1287-1306`).
- Ref updates run `git update-ref --stdin` with a start/prepare/commit
  transaction and an expected old value (CAS) (`cli/gitrepo/ref_cas.go:34-58`,
  `:86`, `:118-125`).
- Push and fetch run `git push --no-verify --porcelain` and `git fetch`
  (`cli/checkpoint/remote/git.go:430-460`, `:241`).

## 2. Session capture

### Claude Code hooks

These are written into `.claude/settings.json` of the worktree root
(`cli/agent/claudecode/hooks.go:58`, `:65`, `:692`):

| Claude Code event | Matcher                  | Command                                       |
| ----------------- | ------------------------ | --------------------------------------------- |
| SessionStart      | (none)                   | `entire hooks claude-code session-start`      |
| SessionEnd        | (none)                   | `entire hooks claude-code session-end`        |
| Stop              | (none)                   | `entire hooks claude-code stop`               |
| SubagentStop      | (none)                   | `entire hooks claude-code subagent-stop`      |
| UserPromptSubmit  | (none)                   | `entire hooks claude-code user-prompt-submit` |
| PreToolUse        | `Agent`                  | `entire hooks claude-code pre-task`           |
| PostToolUse       | `Agent`                  | `entire hooks claude-code post-task`          |
| PostToolUse       | `TaskCreate\|TaskUpdate` | `entire hooks claude-code post-todo`          |

Sources: `hooks.go:52-53` and `:180-245`. Install also removes a retired
metadata deny rule from `permissions` (`hooks.go:76-83`).

### Git hooks

- `prepare-commit-msg`, `commit-msg`, `post-commit`, `post-rewrite` and
  `pre-push` (`cli/strategy/hooks.go:42`, `:558-622`). They respect
  `core.hooksPath`.
- All hooks except pre-push swallow exit codes. Pre-push does not, so that an
  OPF failure can abort the user's push (`hooks.go:563-581`).

### Local state

- Session state: `.git/entire-sessions/<session-id>.json` in the git common
  dir, shared across worktrees (`cli/session/state.go:28`, `:109`).
- Shadow branches, one per base commit per worktree:
  `entire/<commit[:7]>-<sha256(worktreeID)[:6]>`
  (`cli/checkpoint/ephemeral.go:38-55`, `:723-730`).
- These are real `refs/heads` branches. They hold a full worktree snapshot
  (code is stored **unredacted**) plus `.entire/metadata/<session>/...`.
  They are never pushed by Entire. They are deleted after condensation and push
  (`cli/strategy/cleanup.go:228`, `:290-316`).
- Local unredacted copy: on Stop, the agent transcript is sanitized and written
  to `.entire/metadata/<session>/full.jsonl` (0600) in the worktree, with no
  redaction (`cli/lifecycle.go:805-821`). Redaction happens when that file is
  blobbed into the shadow tree.

### When writes happen

- **Stop (turn end):** a temporary checkpoint commit on the shadow branch
  (SaveStep), redacted with the regex layers.
- **prepare-commit-msg:** mints the checkpoint ID (or reuses a pending one) and
  appends `Entire-Checkpoint: <id>` (`cli/strategy/manual_commit_hooks.go:2782-2799`).
  - For editor commits it adds a comment that says the trailer can be removed.
  - For `-m`/`-F` it prompts via /dev/tty (`:325-339`).
- **commit-msg:** strips the trailer if the message has no other content
  (`:145-169`).
- **post-commit:** condenses the shadow data into the persistent store under
  that ID (`PostCommit`, `:1086`), and on git-refs enqueues the ref for push.
- **pre-push:** pushes (see section 3).

### Trailer under history rewrites

- **Amend** (source `commit`): an existing trailer is kept. If none is present,
  it is restored from the session's `LastCheckpointID` when
  `state.BaseCommit == HEAD` (`manual_commit_hooks.go:367-370`;
  `manual_commit_redo.go:119`; `manual_commit_hooks.go:704-780`).
- **Rebase / cherry-pick / revert:** prepare-commit-msg is skipped
  (`isGitSequenceOperation`, `:301`, `:343-352`). The trailers survive only
  because git copies the messages.
  - post-rewrite (amend|rebase) remaps the session's BaseCommit and related
    linkage (`:174-233`).
  - Checkpoint data is not rewritten.
- **Squash:**
  - `git merge --squash`: trailers found in `SQUASH_MSG` are inherited when a
    staged path matches (`:556-700`; CHANGELOG 0.11.3, PR #2574).
  - Reset-and-redo: trailers are inherited from HEAD's reflog
    (`inheritReplacedCommitsTrailers`, `manual_commit_redo.go`).
  - Interactive-rebase squash/fixup gets no special handling. Git
    concatenates or drops messages.
- **Merge commits** are deliberately left unlinked (`:358-365`).

## 3. Push and fetch

### Refspecs

- git-refs batch push, one round trip:
  `refs/entire/checkpoints/<s>/<id>:refs/entire/checkpoints/<s>/<id>` per ref.
  It never forces (`cli/strategy/push_common.go:58-66`).
- v1 branch: bare `entire/checkpoints/v1`. Non-branch refs use an explicit
  `ref:ref` (`push_common.go:455-458`). Again no force.
- Non-fast-forward recovery fetches
  `+<ref>:refs/entire-fetch-tmp/<ref>`, cherry-picks the local commits on top
  and re-pushes (`push_common.go:561-600`).
- Per-checkpoint read fetch:
  1. probe with `ls-remote`;
  2. then fetch `+<ref>:<ref>`.

  Source: `cli/checkpoint/remote/checkpoint_ref.go:340-375`. Discovery uses
  `git ls-remote refs/entire/checkpoints/*` (names only).

### When pushes happen

- Only in the user's `git push` (pre-push hook), plus explicit commands such as
  the migration push.
- git-refs uses a flock-protected JSONL queue in the git common dir:
  `entire-checkpoint-push-queue.jsonl` with its `.lock`
  (`cli/checkpoint/pushqueue.go:26-27`, `:45-80`).
  - The queue is drained at pre-push.
  - Entries are removed only after a confirmed push.
  - Failures stay queued and never fail the user's push
    (`cli/strategy/manual_commit_push.go:407-446`, `:486-560`).

### Resolution order

Inside `prePush` (`manual_commit_push.go:56-227`):

1. **`push_sessions: false`** returns early (`:79-81`).

  - `--skip-push-sessions` on `enable`/`configure` writes
     `strategy_options.push_sessions=false` (`cli/setup.go:51`, `:107-114`,
     `:898`).
  - It is read by `IsPushSessionsDisabled` (`cli/settings/settings.go:2105-2115`).

2. **`checkpoint_remote`** is
   `strategy_options.checkpoint_remote = {"provider":"github"|"gitlab","repo":"owner/repo"}`.

  - `--checkpoint-remote github:owner/repo` writes it (`setup.go:115-128`).
  - An invalid flag value prints only a warning and skips the write
     (`setup.go:117-118`).
  - It is resolved in `resolvePushSettings` (`cli/strategy/checkpoint_remote.go:82-134`)
     through `remote.PushURL` (`cli/checkpoint/remote/util.go:387-512`).
  - If PushURL returns `enabled=false`, `checkpointURL` stays empty and the
     push target falls back to the user's push remote name
     (`checkpoint_remote.go:59-64`, `:99-110`).

3. **Single-remote gate** (when there is no dedicated URL): checkpoints go
   only when the pushed remote is the elected "checkpoint sync remote"
   (`manual_commit_push.go:99-107`; `cli/strategy/checkpoint_sync_remote.go:72-145`).
   Election order:
  1. `checkpoint_push_remote`, which **fails closed**: if the named remote has
      no fetch URL, sync is disabled until fixed (`:89-93`);
  2. the captured election (fail-soft);
  3. `origin`;
  4. the sole remote;
  5. the first remote.

  - A push to any other remote or a raw URL carries nothing, and only a debug
     log is written (`:136-142`).

### PushURL silently falls back to the origin/push-remote URL

Each of these cases logs only a `logging.Warn` to `.entire/logs` and returns
`(fallbackURL, false, nil)`:

- settings fail to load (`util.go:393-402`);
- the push remote's URLs are unreadable (`:418-427`);
- **the ownership vote refuses the store** (`:437-448`);
- the push URL is unparseable (`:450-458`);
- the checkpoint URL cannot be derived (`:493-509`).

The ownership vote (`checkpointRemoteIsInherited`, `util.go:633-710`) refuses
when origin or any push URL names an owner different from the checkpoint repo's
owner (`OwnershipDisproved`), or when no owner can be read
(`OwnershipUnprovable`). `settings.local.json` is the escape hatch
(`CheckpointRemoteIsLocalOnly`).

### The 0.11.0 incident

Previously recorded from trost-systems/emotely#199: a stale committed
`checkpoint_remote` with another owner; the log said "ignoring checkpoint_remote
that appears to belong to another owner; pushing checkpoints to the push remote
instead"; refs went to the public origin.

- The log string is exactly `util.go:442` at HEAD, and it was already
  `util.go:439` at tag v0.11.0.
- Path: `prePush` calls `resolvePushSettings`, which calls `PushURL`.
  `PushURL` finds `verdict.Refused()` (Disproved), returns the origin URL with
  `enabled=false`, and `checkpointURL` stays "".
  - The gate then passes, because origin is the elected sync remote by default.
  - `flushCheckpointRefsQueue` pushes to `origin` (`manual_commit_push.go:426`,
    `:512-528`).
- This is the same class as issue #1139 (0.6.0, open): checkpoint_remote under
  another owner, but v1 was pushed to origin.

### Was it fixed?

No, not in the fail-closed sense.

- PR #2521 "Say when a checkpoint_remote is ignored, and offer to claim it" was
  merged 2026-09-30 as 3fc8c2a7b (key commits 87df65461 and 36596bbde). It is
  not in any stable release; the first build containing it is nightly
  v0.11.4-nightly.202610010628. It adds `warnIgnoredCheckpointRemote`
  (`cli/strategy/checkpoint_remote_hint.go:14-90`), which prints to stderr
  **after** checkpoints were delivered.
- The commit message says outright: "It blocks nothing. Checkpoints keep
  flowing to the elected remote, because refusing to push them would turn a
  misconfiguration into lost work."
- Commit 36596bbde then limited the pre-push warning to `OwnershipUnprovable`
  only (`checkpoint_remote_hint.go:55-62`). In the incident's exact case
  (a *different owner* = `OwnershipDisproved`), pre-push still writes only the
  Warn log and pushes to origin at HEAD. That case is reported only by
  `entire status` / `entire enable`.
- Fallbacks for reasons other than ownership never print a terminal warning
  (`checkpoint_remote_hint.go:56-60`).
- The CHANGELOG has no entry for this incident (0.11.0 to 0.11.3). Related
  fail-closed work concerns only `checkpoint_push_remote` (PR #2345, 0.11.0)
  and the read side (`resolveCheckpointFetchURL`, `checkpoint_remote.go:284-312`;
  `probeAndFetchCheckpointRef` refusing to treat fallback emptiness as absence).

### Missing or stale remote

- A missing `checkpoint_push_remote` fails closed: no push, one Warn
  (`checkpoint_sync_remote.go:89-93`, `:131-137`).
- A stale captured election falls through to origin.
- If `checkpoint_remote` cannot be resolved, it falls back to origin (above).
- On git-branch with a dedicated URL, the metadata branch is fetched once if
  missing. A fetch failure is a Warn (`checkpoint_remote.go:125-131`,
  `:234-271`).
- If push fails, refs stay queued.
- A remote that refuses keeps failing every push. The fallback is bounded by
  `checkpointFlushBudget` and `maxConsecutiveRefPushFailures`, then the queue
  is rotated (`docs/architecture/ref-checkpoint-backend.md:122-124`).

## 4. Redaction

### Engine

The engine is `redact/` (`redact/redact.go:165-276`). Layers, in order:

1. Shannon entropy > 4.5 on `[A-Za-z0-9+_=-]{10,}`;
2. betterleaks (`github.com/betterleaks/betterleaks/detect`, on by default)
   and/or goredact (`github.com/lastpersonlabs/goredact`, off by default)
   (`redact/scanners.go:23-55`);
3. provider token prefixes;
4. credentialed URIs;
5. DB connection strings;
6. user custom rules / rule packs;
7. `*_PASSWORD=` style key/value pairs;
8. opt-in PII.

An optional 9th layer is OPF. It is **not network**: it shells out to a local
`opf` binary (`redact/opf.go:21-70`, `os/exec`), only at pre-push. It has a
circuit breaker and fails closed for the trailer-stamping path. Replacement
text is `REDACTED` (`redact.go:70`).

### Fields covered

JSONL-aware: each string leaf of `full.jsonl`, `transcript.jsonl` and
subagent transcripts. Exceptions (`redact.go:1336-1372`):

- keys ending in `id`/`ids` or `signature`, and path keys (`file_path`, `cwd`,
  `path`, ...) are skipped;
- objects with `type: image*|base64` are skipped (images stored unredacted).

Also redacted:

- prompts (`RedactedJoinedPrompts`, `persistent.go:440`, `:699`);
- Summary text (`RedactSummary`, `:1221-1270`);
- task description (`:515`);
- `review_prompt` and `investigate_topic` (`:754-756`; added 2026-09-25,
  51a80a47a);
- metadata `.json` files as JSON;
- other text blobs as plain bytes (`RedactBlobBytes`, `:2594-2620`).

Not redacted:

- shadow-branch code snapshots;
- binary blobs;
- externalized images;
- the local `.entire/metadata` copy.

### Ordering

Redaction runs before objects are created:

- `createRedactedBlobFromFile` redacts first, then calls `CreateBlobFromContent`
  (`persistent.go:2519-2567`);
- condensation calls `redactSessionTranscript` before the write
  (`cli/strategy/manual_commit_condensation.go:840-860`);
- the store API takes a `redact.RedactedBytes` type (`redact.go:92-118`).

### Failure behaviour

Fail-closed for JSON(L):

- `ErrScannerDegraded`, when goredact is the sole scanner and fails
  (`scanners.go:16-21`);
- `ErrRedactionIncomplete` (`redact.go:1303`).

Both fail the write (`persistent.go:2608-2613`), and condensation errors
(`manual_commit_condensation.go:855-857`). A JSON parse failure falls back to
plain-text redaction, which is not a failure.

## 5. Integrity

- No hash chain or signature over checkpoint *content* beyond git's own object
  graph. With git-refs, each checkpoint is an isolated orphan chain, not linked
  to other checkpoints.
- `content_hash.txt` is a sha256 of `full.jsonl`, used for dedup, not for
  verification.
- Commit signing (`persistent.go:2698-2730`; `docs/architecture/checkpoint-signing.md`):
  - it is best-effort, through go-git objectsigner plugins (GPG/SSH);
  - it is on by default (`sign_checkpoint_commits`, `settings.go:2129-2143`),
    but only takes effect if `commit.gpgsign=true` at global or system scope;
  - signer failure produces an unsigned commit plus a Warn log;
  - nothing verifies signatures on read.
- Rewriting is designed in:
  - OPF pre-push **rewrites unpushed checkpoint commits** and moves the refs
    (`cli/strategy/manual_commit_opf_refs.go`, `manual_commit_opf_rewrite.go`);
  - fetch+replay cherry-picks local commits onto the remote ref;
  - `DeleteOrphanedCheckpoints` removes checkpoint subtrees from v1
    (`cli/strategy/cleanup.go:398`, called at `:654` from cleanup);
  - shadow branches are deleted after push.
- Remote push is never forced, and per-checkpoint history is append-only by
  convention. The docs say there is "no server-side ref protection".
- Retention:
  - no checkpoint retention or GC;
  - session state files are purged after 7 days stale (`cli/session/state.go:32`);
  - the zombie sweep finalizes sessions;
  - Antigravity status snapshots last 14 days.

## 6. Network and dependencies

`go.mod` has **40 direct** and 92 indirect requirements. Notable direct ones:

- go-git v6 (pseudo-version alpha) and go-git objectsigner plugins;
- betterleaks, goredact;
- oklog/ulid, gofrs/flock;
- posthog-go (telemetry), zalando/go-keyring, entireio/auth-go;
- ogen-go/ogen and go-faster/jx (API client);
- playwright-go, creack/pty;
- the charm.land bubbletea, huh and glamour family, cobra;
- golang.org/x/net, golang.org/x/crypto, yaml.v3, hujson, machineid.

`net/http` is imported in 44 non-test files, 13 of them in the main
`cmd/entire/cli` package, so it is in the binary. Uses include:

- auth, api, trail, recap, search, plugin fetch;
- the `git-remote-entire` helper (`internal/remotehelper`);
- the Entire cloud cluster client (`internal/entireclient`);
- a version check against `api.github.com/repos/entireio/cli/releases/latest`
  (`cli/versioncheck/types.go:19`);
- PostHog telemetry to `https://eu.i.posthog.com`, **on by default**, with
  opt-out (`cli/telemetry/detached.go:26`, `:446`;
  `docs/security-and-privacy.md:547-557`).

`os/exec` is imported in 96 non-test files: git, opf, agents and summarizers.

These figures come from grep. `go list -deps` was not run because the local
toolchain is 1.26.8 and the module requires 1.27.1.

## 7. Concurrency

- A per-ref writer flock in the common dir,
  `entire-persistent-ref-locks/<ref>.lock`, serializes Entire's writers
  (`cli/checkpoint/persistent_ref_update.go:36-64`).
- Inside the lock, build is followed by native `git update-ref` CAS, with up to
  16 retries and jittered backoff. Each retry rebuilds from the fresh tip and
  deletes the loser's loose object (`:130-160`; `cli/checkpoint/shadow_ref.go:17-30`).
- The same machinery protects:
  - v1 (`cli/checkpoint/store.go:115-124`);
  - each per-checkpoint ref;
  - shadow branches, which have their own per-shadow flock.
- On v1, all agents and worktrees contend on one tip. On git-refs, contention
  is per checkpoint only.
- Concurrent sessions in one worktree share a shadow branch and interleave.
  Each session gets its own numbered folder (`0/`, `1/`) inside a checkpoint.
- Session-state mutation is serialized (`MutateSessionState`,
  `cli/strategy/session_state.go:538`).
- The push queue is flock plus atomic rename.
- Cross-machine races: a non-fast-forward push is recovered by fetch plus
  cherry-pick replay. A true conflict (both sides edited root `metadata.json`)
  leaves the ref queued, never forced.
- Commit linking across worktrees uses process-ancestry identity plus a
  15-minute liveness window. It declines when ambiguous (docs section
  "Commit-to-session linking").

## Comparison with the earlier summary

| Claim                                              | Code verdict                                                                                                                                                                                                                                                                                                                                                                      |
| -------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| refs `refs/entire/checkpoints/<shard>/<id>`        | Confirmed, but only for the `git-refs` backend. It was added in 0.8.0 (#1566) and became the default for new setups in 0.9.0 (#1789), written without a prompt since 0.10.0 (#1900). Repos without a `checkpoints` block still use the `entire/checkpoints/v1` branch.                                                                                                            |
| shard = last two ID characters                     | Confirmed for refs. On the v1 branch the shard is the **first** two characters (`<id[:2]>/<id[2:]>/`).                                                                                                                                                                                                                                                                            |
| ID format                                          | Two formats: a 12-hex ID (branch backend) or a 26-char ULID (refs backend). The summary did not state which.                                                                                                                                                                                                                                                                      |
| each checkpoint holds metadata.json and full.jsonl | Incomplete. Root `metadata.json` is a summary. `full.jsonl` sits under per-session folders `<n>/`, next to `transcript.jsonl` (compact), `prompt.txt`, `content_hash.txt`, optional `assets/`, and `tasks/<tool-use-id>/`.                                                                                                                                                        |
| `Entire-Checkpoint: <id>` trailer                  | Confirmed (`cli/trailers/trailers.go:42`). It is added in prepare-commit-msg, the user can remove it, it is stripped from empty messages, preserved or restored on amend, and inherited on `merge --squash` / reset-redo. Checkpoint commits themselves carry `Entire-Session`, `Entire-Strategy`, `Entire-Agent`, and optionally `Entire-OPF-Applied`.                           |
| secrets redacted before writing                    | Mostly confirmed for transcripts, prompts and metadata text. JSONL fails closed on scanner degradation. Exceptions: shadow-branch code snapshots (raw, in local git objects); images (base64 inline or `assets/`) are never scanned; keys ending in `id`, `signature` or path names are skipped; the worktree copy `.entire/metadata/<session>/full.jsonl` is unredacted on disk. |
