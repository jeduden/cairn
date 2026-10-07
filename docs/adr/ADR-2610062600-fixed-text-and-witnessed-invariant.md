---
id: ADR-2610062600
title: "Security review of the fixed-text and witnessed wording"
status: accepted
summary: >-
  The security reviewer's record for rewording I2 after the eleventh
  round of blind domain-model reviews: restore blocks, opt-in notices
  and fixed templates may carry fixed text Cairn ships, and only the
  harness_meta events and user turns this node's hook handlers
  witnessed are trusted. Accepted by the stakeholder on 7 October 2026.
---
# ADR-2610062600: Security review of the fixed-text and witnessed wording

## Context

The eleventh round of blind domain-model reviews ([merged
note][blind11]) found I2 out of step with the model and the
requirements:

- **Fixed text.** I2 built restore blocks only from qualifying pins and
  trusted structural fields, and notices and templates only from
  structural fields and ids. A restore block carries the recall hint
  and fixed headings (INJ-01, §9.4), and OWN-07's template carries its
  own wording, none of which is a structural field.
- **Ingested events.** I2 trusted this node's `harness_meta` events and
  `user` turns, while REC-22 and PRV-02 keep an ingested run untrusted,
  though it is this node's own transcript.

The stakeholder asked for rounds to repeat until none finds anything
to address, with the recommended option on every question (the note's
section 1). CLAUDE.md treats an invariant change as a design change
that needs a security review and a new major version.
[ENG-29](../srs/10-engineering-quality.md#104-process) makes the review
a gate. The reviewer is the stakeholder, @jeduden.

## Decision

The reviewer approves or declines each change below. The status moves to
accepted only when every change reads approved or withdrawn.

| #   | Change                     | Review   |
| --- | -------------------------- | -------- |
| 1   | I2, fixed text Cairn ships | approved |
| 2   | I2, witnessed events only  | approved |

### 1. I2, fixed text Cairn ships

- **Before:** "restore blocks …, built only from qualifying pins and
  trusted structural fields; and opt-in notices … and the fixed
  templates of OWN-04 and OWN-07, each built only from trusted
  structural fields and ids."
- **After:** "restore blocks …, built only from qualifying pins, trusted
  structural fields and fixed text Cairn ships; and opt-in notices …
  and the fixed templates of OWN-04 and OWN-07, each built only from
  fixed text Cairn ships, trusted structural fields and ids."
- **What changes:** the invariant names the wording Cairn ships, which
  INJ-01, INJ-10 and OWN-07 already use. No content from outside Cairn
  gains a path.
- **Review:** approved by the stakeholder, @jeduden (7 October 2026).

### 2. I2, witnessed events only

- **Before:** "this node's own `operator`, `harness_meta` and
  structural events, its `user` turns while the deployment mode is
  `interactive`".
- **After:** "this node's own `operator` and structural events, the
  `harness_meta` events and `user` turns its hook handlers witnessed,
  the turns while the deployment mode is `interactive`".
- **What changes:** ingested transcript lines no longer read as
  trusted, as REC-22 requires. This narrows trust.
- **Review:** approved by the stakeholder, @jeduden (7 October 2026).

## Alternatives

- Keep I2 and drop the recall hint from restore blocks; declined, since
  the hint is how an agent learns the recall tools exist.

## Consequences

The reviewer approved both changes, so the record is accepted
(7 October 2026) and the new wording binds. CLAUDE.md, AGENTS.md and
README carry it through their includes of
[invariants.md](../srs/invariants.md). No requirement's traces change.

[blind11]:
  ../../plan/2610012322_cairn-for-agent-fleets/domain-model-review-blind-11.md
