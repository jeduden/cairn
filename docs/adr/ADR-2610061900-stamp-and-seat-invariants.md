---
id: ADR-2610061900
title: "Security review of the stamp and seat invariant changes"
status: accepted
summary: >-
  The security reviewer's record for rewording I2, I3, I4 and I8 after
  the third round of blind domain-model reviews: a stamped pin version
  is a trusted source for its stamper's own agents, trust grants cover
  device seats only, I3 speaks of every pin that restores, I4 of hook
  handlers, and I8 names the stamp and trust-grant exceptions.
  Accepted by the stakeholder on 6 October 2026.
---
# ADR-2610061900: Security review of the stamp and seat invariant changes

## Context

The third round of blind domain-model reviews ([merged
note][blind3]) found the invariants out of step with the requirements
and the model in five places:

- **I2's trusted sources** did not list a stamped pin version, though
  LANE-32 and PIN-10 restore it word for word to its stamper's agents.
- **I2's trust grant** named service-account seats, a seat kind the
  stakeholder removed: every principal now acts without a run through
  one device seat per device.
- **I3** said "pinned constraints", which is wider than PIN-10's
  qualifying pins, and silent on a pin the restore budget omits.
- **I4** listed "hooks" in the core, where the hook is the harness's
  callback and Cairn's code is its hook handlers.
- **I8** said Cairn refuses a home "it does not own", while the model
  says "owns" only of rooms, and called all content from another
  principal untrusted, which a stamp or a trust grant overrides.

The stakeholder decided the first three on 6 October 2026, as recorded
in the note's section 4, and approved the I4 and I8 wording with the
instruction to apply the round. CLAUDE.md treats an invariant change as
a design change that needs a security review and a new major version.
[ENG-29](../srs/10-engineering-quality.md#104-process) makes the review
a gate. The reviewer is the stakeholder, @jeduden.

## Decision

The reviewer approves or declines each change below. The status moves to
accepted only when every change reads approved or withdrawn.

| #   | Change                                    | Review   |
| --- | ----------------------------------------- | -------- |
| 1   | I2, a stamped pin version is trusted      | approved |
| 2   | I2, trust grants cover device seats only  | approved |
| 3   | I3, every pin that restores               | approved |
| 4   | I4, hook handlers                         | approved |
| 5   | I8, the home's OS user and two exceptions | approved |

### 1. I2, a stamped pin version is trusted

- **Before:** "… principal acts signed by a device key the agent's
  principal certified, within its scope."
- **After:** "… within its scope, and a pin version a principal
  stamped, for that principal's own agents."
- **What changes:**
  - The invariant now states what LANE-32 and PIN-10 already do; no
    requirement's behaviour changes.
  - A stamp is a widening principal act, taken after the stamper is
    shown the version's exact text, author and key fingerprint. It
    trusts that one version for the stamper's own agents only, never
    for another principal's.
  - A room act never changes any restore block: an edit adds an
    unstamped version and an unpin takes the pin off the room's pin
    list, while the stamped version keeps restoring until its stamper
    unstamps it, a cut principal act.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 2. I2, trust grants cover device seats only

- **Before:** "the posts and pins that principal wrote from its device
  and service-account seats".
- **After:** "the posts and pins that principal wrote from its device
  seats".
- **What changes:** only the seat's name. A service account now acts
  without a run through its device seat, so the grant covers the same
  posts and pins as before, and still never a run seat.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 3. I3, every pin that restores

- **Before:** "Pinned constraints are stored verbatim and restored
  verbatim after every compaction."
- **After:** "Every pin that restores is stored verbatim and restored
  verbatim after every compaction, or named by id and count when the
  budget omits it."
- **What changes:**
  - The invariant covers PIN-10's qualifying pins, of the types that
    restore, rather than every pin.
  - A pin the restore budget leaves out is never cut short or
    summarized. PIN-08 already counts what it left out; it now also
    names each omitted pin by id.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 4. I4, hook handlers

- **Before:** "B0, the core (hooks, the MCP server, …".
- **After:** "B0, the core (hook handlers, the MCP server, …".
- **What changes:** only the word. The hook is the harness's callback;
  Cairn's hook handlers answer it inside the core. No component moves
  across a boundary.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

### 5. I8, the home's OS user and two exceptions

- **Before:** "Cairn refuses to operate on a home it does not own.
  Content another principal or node wrote is held as theirs:
  attributed to its seat key, untrusted, and never a way to widen what
  this principal's agents trust or recall."
- **After:** "Cairn refuses to operate on a home that does not belong
  to the OS user running it. Content another principal or node wrote
  is held as theirs: attributed to its seat key, and untrusted unless
  this principal's own stamp or trust grant covers it. It never widens
  what this principal's agents trust or recall on its own."
- **What changes:**
  - The home check is the one SEC-02 makes, against the running UID;
    only the verb changes.
  - The two exceptions are I2's own: a stamp and a trust grant are
    widening principal acts of this principal. Content never widens
    trust by itself.
- **Review:** approved by the stakeholder, @jeduden (6 October 2026).

## Alternatives

- End every stamp when the pin is edited or unpinned, as LANE-32 said
  before; declined by the stakeholder, since a room act would then
  change another principal's restore block.
- Keep "pinned constraints" in I3; declined, since `preference` and
  `intent` pins restore too and `fact` pins never do.

## Consequences

The reviewer approved all five changes, so the record is accepted
(6 October 2026) and the new wording binds. CLAUDE.md, AGENTS.md and
README carry it through their includes of
[invariants.md](../srs/invariants.md). No requirement's traces change.

[blind3]:
  ../../plan/2610012322_cairn-for-agent-fleets/domain-model-review-blind-3.md
