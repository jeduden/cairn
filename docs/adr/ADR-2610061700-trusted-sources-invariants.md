---
id: ADR-2610061700
title: "Security review of the trusted-sources invariant changes"
status: accepted
summary: >-
  The security reviewer's record for rewording I1, I2 and I3 after the
  second round of blind domain-model reviews: I2 lists interactive user
  turns and harness metadata as trusted sources and lets a trust grant
  cover pins as well as posts, I1 names the retention policy, and I3
  says restored. Accepted by the stakeholder on 6 October 2026.
---
# ADR-2610061700: Security review of the trusted-sources invariant changes

## Context

The second round of blind domain-model reviews
([merged note][blind2]) found the invariants out of step with the
requirements and the model in three places:

- **I2's trusted sources** listed only this node's `operator` and
  structural events and certified principal acts. PRV-02 also trusts
  `harness_meta`, and `user` turns in interactive deployment mode, and
  LMK-02 and PIN-05 depend on that trust.
- **I2's trust grant** carried only "posts", while OWN-29 and LANE-27
  let a trust grant cover the trusted principal's posts and pins.
- **I1** said data "expires by policy", which collides with the expire
  act, and **I3** said "re-injected" where the model says restore.

The stakeholder decided the first two on 6 October 2026, as recorded in
the note's section 5, and asked for every decision to be applied.
CLAUDE.md treats an invariant change as a design change that needs a
security review and a new major version.
[ENG-29](../srs/10-engineering-quality.md#104-process) makes the review
a gate. The reviewer is the stakeholder, @jeduden.

## Decision

The reviewer approves or declines each change below. The status moves to
accepted only when every change reads approved or withdrawn.

| #   | Change                                    | Review   |
| --- | ----------------------------------------- | -------- |
| 1   | I2, trusted sources                       | approved |
| 2   | I2, what a trust grant carries            | approved |
| 3   | I1, retention policy instead of "expires" | approved |
| 4   | I3, "restored" instead of "re-injected"   | approved |

### 1. I2, trusted sources

- **Before:** "this node's own `operator` and structural events and,
  once PRV-10 ships, principal acts …".
- **After:** "this node's own `operator`, `harness_meta` and structural
  events, its `user` turns while the deployment mode is `interactive`,
  and, once PRV-10 ships, principal acts …".
- **What changes:**
  - The invariant now states what PRV-02 already enforces; no
    requirement's behaviour changes.
  - Text a person types at the harness is trusted only in interactive
    mode. In automation mode a pipeline types it, and it stays
    untrusted.
  - `harness_meta` holds the harness's structural records, never free
    text: PRV-02 classes free text a record carries as `harness_text`.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 2. I2, what a trust grant carries

- **Before:** "that principal's posts then reach those agents".
- **After:** "the posts and pins that principal wrote from its device
  and service-account seats then reach those agents".
- **What changes:**
  - A trust grant may now make another principal's pin restore to the
    granting principal's agents, as OWN-29 and LANE-27 say.
  - It never covers run seats or relaying service accounts, so no
    agent's text becomes trusted through it.
  - A pin that restores can be added, edited or unpinned only by a
    widening principal act of its author's principal, and a moderator
    cannot drop it from another principal's agents.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 3. I1, retention policy instead of "expires"

- **Before:** "data the node's principal explicitly purges or expires
  by policy".
- **After:** "data the node's principal explicitly purges or removes
  under a retention policy".
- **What changes:** only the word; the expire act ends an expiry on a
  bar, a mute or a handover offer and destroys no data.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 4. I3, "restored" instead of "re-injected"

- **Before:** "re-injected verbatim after every compaction".
- **After:** "restored verbatim after every compaction".
- **What changes:** only the word, which matches the restore block.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

## Alternatives

- Drop interactive `user` trust from PRV-02 instead; declined, since
  landmarks and pin candidates rely on it.
- Keep trust grants to posts and require a stamp for every other
  principal's pin; declined by the stakeholder.

## Consequences

The reviewer approved all four changes, so the record is accepted
(6 October 2026) and the new wording binds. CLAUDE.md, AGENTS.md and
README carry it through their includes of
[invariants.md](../srs/invariants.md). No requirement's traces change.

[blind2]:
  ../../plan/2610012322_cairn-for-agent-fleets/domain-model-review-blind-2.md
