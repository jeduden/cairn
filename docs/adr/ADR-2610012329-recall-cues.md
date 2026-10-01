---
id: ADR-2610012329
title: "Moment-triggered pointer cues for retrieval"
status: proposed
summary: >-
  At hook moments where a need for evicted history is likely, Cairn
  pushes a short, gated TrustedText pointer to it, and Claude pulls
  the content through recall. No record text is pushed.
---
# ADR-2610012329: Moment-triggered pointer cues

## Context

Retrieval is what Cairn is for (SRS §1.2). Pull-only recall (§5.4)
keeps untrusted history away from the model until Claude asks (I2).
But after a compaction Claude sees only a structural index and a
hint. It cannot ask for history it does not know it lost. The recall
tool-use target in §11.1 rests on that unproven step.

Pushing history back solves the blind spot and reopens the poisoning
path: injection at session start carried poison forward 64.7% of the
time against 17.4% for pull-only retrieval (§3, item 3). Pushing too
much has a second cost. Context Claude does not need takes attention
and tokens, and a stream of unneeded hints teaches Claude to ignore
the next one.

## Decision

- Model retrieval need explicitly (SRS §5.11.1): the working view,
  needs, moments, anchors and failure signatures.
- At four moments a need is likely, Cairn offers a *cue*: a prompt
  that names an anchor found only out of view, a tool about to touch
  a file with out-of-view history, a failure seen before out of view,
  and a command that already ran out of view. Re-entry after
  compaction stays the restore block's job.
- A cue is a pointer, never a copy. It is `TrustedText`: `seq`
  ranges, counts, sanitized paths and anchors, provenance and trust,
  and the recall call that fetches the content. Claude pulls the
  content inside the envelope.
- Seven gates keep useless cues out: out of view, specific,
  unambiguous, fresh, clean, budget and calibrated. A candidate that
  fails one is dropped and counted.
- Every cue is recorded. A cue is *followed* when Claude recalls an
  event it named within a few tool calls; the follow rate is the
  online precision measure, and a moment kind that falls below its
  floor is muted for the session.
- Cues are on by default, because the pointer carries no record text.
  Tenant configuration can switch any moment off; project
  configuration can only switch off or tighten.

## Alternatives

- **Pure pull, as before.** No new surface, but Claude stays blind to
  what it forgot, and retrieval fails exactly when it matters.
- **Per-prompt memory hints with content** (lcm, claude-mem). Finds
  more, and is the poisoning path §3 measures. It would break I2.
- **Content excerpts for every target.** Saves Claude one call, but
  most targets are tool output, which is untrusted. Only trusted user
  turns in `interactive` mode may be quoted (CUE-10).
- **Cues on by heuristics alone, with no follow-through measure.**
  Simpler, but nothing would catch a moment kind that only adds
  noise. Calibration makes precision observable and self-correcting.
- **Embedding similarity for cue selection.** Would catch paraphrased
  references that anchors miss. It conflicts with NG3 and needs a
  pinned local model for I4 and I10; kept as an open question
  (OQ-13).
- **Cues off by default.** Follows the "convenience opt-in" rule, but
  the rule applies where usefulness and safety conflict. A pointer
  built only from `TrustedText` adds no injection path, and an
  opt-in cue would rarely be measured or tuned.

## Consequences

- `INJ-03` widens the builders that accept `TrustedText` to the cue
  builder, and `INJ-04` exempts cues from the off-by-default rule for
  `UserPromptSubmit`. `SEC-07`'s static check covers cue content too.
- Cairn needs `PreToolUse` and `PostToolUse` context injection
  (ASM-11), verified in spike S1. If it fails, the touch, recurrence
  and repeat moments are re-planned, as §2.3 requires.
- Hooks do an indexed lookup at the moments above, inside a 50 ms or
  100 ms budget. A lookup past its deadline delivers nothing and is
  counted (I9).
- A cue can act as a lure: content planted to match a likely anchor
  draws a cue, and Claude recalls it. Flagged and quarantined events
  are never named, every cue states its target's trust, and recall
  stays enveloped and taints the session (T13).
- Paraphrased references with no anchor get no cue. §11.1 measures
  how many needs that misses.
