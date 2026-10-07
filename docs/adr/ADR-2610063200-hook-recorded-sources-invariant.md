---
id: ADR-2610063200
title: "Security review of the hook-recorded trusted sources wording"
status: accepted
summary: >-
  The security reviewer's record for I2's trusted sources after the
  eighteenth round of blind domain-model reviews: the harness_meta events
  and user turns that the hook handlers recorded, not "witnessed", so the
  word stays the origin's alone. Accepted by the stakeholder on 7 October
  2026.
---
# ADR-2610063200: Security review of the hook-recorded trusted sources wording

## Context

The eighteenth round of blind domain-model reviews ([merged
note][blind18]) found "witnessed" in two senses. The origin `witnessed`
covers every event recorded live on this node, by its hook handlers,
CLI, MCP server or launcher. I2's trusted sources say "witnessed" by the
hook handlers, a narrower set. A `harness_meta` event the launcher
records live has origin `witnessed` but is not trusted, and a scenario
(PRV-02) rests on that difference. The model allows one meaning per
word.

The stakeholder asked for rounds to repeat until none finds anything
to address, with the recommended option on every question (the note's
section 1). CLAUDE.md treats an invariant change as a design change
that needs a security review and a new major version.
[ENG-29](../srs/10-engineering-quality.md#104-process) makes the review
a gate. The reviewer is the stakeholder, @jeduden.

## Decision

The reviewer approves or declines each change below. The status moves to
accepted only when every change reads approved or withdrawn.

| #   | Change                                 | Review   |
| --- | -------------------------------------- | -------- |
| 1   | I2, sources the hook handlers recorded | approved |

### 1. I2, sources the hook handlers recorded

- **Before:** "the `harness_meta` events its hook handlers witnessed,
  and the `user` turns they witnessed while the deployment mode is
  `interactive`".
- **After:** "the `harness_meta` events its hook handlers recorded, and
  the `user` turns they recorded while the deployment mode is
  `interactive`".
- **What changes:** only the word. The set is the same: events the hook
  handlers themselves recorded. Events the CLI, MCP server or launcher
  record live keep origin `witnessed` and stay untrusted as before.
- **Review:** approved by the stakeholder, @jeduden (7 October 2026).

## Alternatives

- Rename the origin `witnessed` instead; declined, since the origin
  names appear in recall output and §9.7.6's trust marks, while I2's
  word appears only in prose.
- Keep both senses, as the fourteenth round decided; declined, since
  every later round found the overlap again.

## Consequences

The reviewer approved the change, so the record is accepted (7 October
2026) and the new wording binds. CLAUDE.md, AGENTS.md and README carry
it through their includes of [invariants.md](../srs/invariants.md).
PRV-02, VIEW-07, §9.7.6 and the pending scenarios take the same word. No
requirement's traces change.

[blind18]:
  ../../plan/2610012322_cairn-for-agent-fleets/domain-model-review-blind-18.md
