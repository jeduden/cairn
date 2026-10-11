---
id: ADR-2610112209
title: "I2 and I3 after the rulings on spikes S1 and S3"
status: proposed
summary: >-
  A proposed record, for the security reviewer's approval, of three
  changes to I2 and I3 that the
  stakeholder's rulings of 11 October 2026 on spikes S1 and S3 (OQ-51)
  need: trusted user events matched to a prompt, compaction guidance
  dropped from the closed paths, and pins omitted for run ambiguity.
  Supersedes ADR-2610091555 once accepted.
---
# ADR-2610112209: I2 and I3 after the rulings on spikes S1 and S3

## Context

Spikes S1 and S3 recorded Claude Code 2.1.296's hook contract and
transcripts. On 11 October 2026 the stakeholder ruled on their findings
(OQ-51):

- Cairn installs only the hooks through which it writes to an agent.
  The harness session's MCP server records each run from its transcripts
  (REC-13), so no hook handler records `user` or `harness_meta` events
  any more.
- Compaction guidance (PIN-07) is withdrawn with the `PreCompact` hook.
- After an ambiguous compaction, INJ-05 restores only the pins that
  qualify for every candidate run and names the rest. Spike S13 showed
  that no hook input or transcript names the compacting run while its
  hooks run.

The domain-model and security reviews of writing these in found that I2
and I3, as approved in [ADR-2610091555](ADR-2610091555-initial-invariants.md),
no longer match:

- I2 trusts "the `harness_meta` events its hook handlers recorded, and
  the `user` events they recorded". Read strictly, nothing is trusted
  now; read loosely, trust would rest on a transcript file any process
  of the same OS user can append to.
- I2 lists compaction guidance among the closed paths.
- I3 lets a restore omit a pin only "when the budget omits it".

ENG-29 lets a change to an invariant land only with an accepted record
of a named human security reviewer's approval.

## Decision

[docs/srs/invariants.md](../srs/invariants.md) changes in three places:

1. **I2's trusted sources.** "The `harness_meta` events its capture
   recorded, and the `user` events its capture matched to a prompt its
   hook handlers got (PRV-08), while the deployment mode is
   `interactive`". PRV-08 states the match: the line marks its prompt as
   the harness's own input, not the harness's own notice
   (`promptSource`), carries the hook input's `prompt_id`, and its text
   matches the prompt commitment a `UserPromptSubmit` hook handler of the
   same harness session kept. The commitment is a structural hook
   observation over the prompt's redacted text, keyed under a node-local
   key that never leaves the node and is erased when the matched event is
   purged. The match result is recorded, so trust derives from the
   record alone (I10). Only a prompt typed interactively counts; a
   print-mode prompt stays `user` but untrusted.
   Spike S13 found that the harness also submits its own task
   notifications through `UserPromptSubmit`, so the commitment alone is
   not enough.
2. **I2's closed paths.** Compaction guidance (PIN-07) is removed. The
   list only narrows.
3. **I3.** A pin may also be named by id and count instead of restored
   when INJ-05 cannot tell which run a restore is for.

The text in force is the one invariants.md holds in the commit that
adds this record. Its SHA-256 is:

```text
6a5868e9141bd559f84f3352703980cf44a22880319acf35635212478fe25a99
```

Until a security reviewer is named, the stakeholder holds that role.
This record becomes `accepted` when the stakeholder approves it by a
comment on the pull request that adds it. The comment names the
record's id and the hash above.

## Alternatives

- **Trust what capture recorded live, with no prompt match.** The
  smallest wording change, but trust would rest on a file the agent can
  write to through its tools. The stakeholder chose the match.
- **Keep a hook that records `user` events.** That contradicts the
  ruling that Cairn installs only the hooks through which it writes.
- **Leave I3 and treat the omission as a budget omission.** The reviews
  would keep flagging it, and the reason for an omission would not be
  stated.

## Consequences

- A prompt the harness rewrites before writing it to the transcript
  stays an untrusted `user` event; this fails safe.
- Other `harness_meta`, such as model tokens and metadata lines, is
  still read from the transcript and stays trusted: it carries only
  structural fields. Sandbox state and any field that gates a principal
  act come only from the launcher or a hook observation (PRV-02). The
  approval of this record accepts that residual risk (§6.1, R8).
- After a compaction while any subagent may run, background subagents
  included, the main agent may get fewer pins verbatim and must recall
  the rest.
- This record supersedes ADR-2610091555 once accepted (ENG-26), and the
  hash above replaces that record's as the approved text.
