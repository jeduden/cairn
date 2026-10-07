---
id: ADR-2610062800
title: "Security review of the witnessed-turns wording"
status: accepted
summary: >-
  The security reviewer's record for correcting I2 after the
  thirteenth round of blind domain-model reviews: the wording
  ADR-2610062600 accepted could be read to trust witnessed user turns in
  automation mode, or every turn in interactive mode; I2 now trusts the
  witnessed harness_meta events, and the witnessed user turns only while
  the deployment mode is interactive. Accepted by the stakeholder on 7
  October 2026.
---
# ADR-2610062800: Security review of the witnessed-turns wording

## Context

The thirteenth round of blind domain-model reviews ([merged
note][blind13]) found, in all three reviews, that the trusted-source
clause [ADR-2610062600](ADR-2610062600-fixed-text-and-witnessed-invariant.md)
accepted reads two ways. It said "the `harness_meta` events and `user`
turns its hook handlers witnessed, the turns while the deployment mode
is `interactive`". One reading trusts witnessed `user` turns in
`automation` mode. A turn runs from an input to the model's reply, so
another reading trusts every turn in `interactive` mode, though I2 names
the model's replies untrusted.

That record said its change narrowed trust; read literally, the words
widen it. PRV-02, PRV-04, VIEW-07, §9.7.6, the model and the PRV-02
scenario all trust witnessed `user` turns only in `interactive` mode, so
no requirement relied on the wider reading.

The stakeholder asked for rounds to repeat until none finds anything
to address, with the recommended option on every question (the note's
section 1). CLAUDE.md treats an invariant change as a design change
that needs a security review and a new major version.
[ENG-29](../srs/10-engineering-quality.md#104-process) makes the review
a gate. The reviewer is the stakeholder, @jeduden.

## Decision

The reviewer approves or declines each change below. The status moves to
accepted only when every change reads approved or withdrawn.

| #   | Change                        | Review   |
| --- | ----------------------------- | -------- |
| 1   | I2, witnessed turns corrected | approved |

### 1. I2, witnessed turns corrected

- **Before:** "the `harness_meta` events and `user` turns its hook
  handlers witnessed, the turns while the deployment mode is
  `interactive`".
- **After:** "the `harness_meta` events its hook handlers witnessed, and
  the `user` turns they witnessed while the deployment mode is
  `interactive`".
- **What changes:** the invariant says what the requirements already
  enforce: a `user` turn is trusted only when witnessed in `interactive`
  mode, and a model reply never is. This narrows the wording to the
  intended trust.
- **Review:** approved by the stakeholder, @jeduden (7 October 2026).

## Alternatives

- Leave the wording and rely on PRV-02; declined, since an invariant
  that reads wider than its requirements is the contract a reviewer
  checks against.

## Consequences

The reviewer approved the change, so the record is accepted (7 October
2026) and the new wording binds. CLAUDE.md, AGENTS.md and README carry
it through their includes of [invariants.md](../srs/invariants.md). No
requirement's traces change.

[blind13]:
  ../../plan/2610012322_cairn-for-agent-fleets/domain-model-review-blind-13.md
