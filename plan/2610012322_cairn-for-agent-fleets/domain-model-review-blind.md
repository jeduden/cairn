# Domain model: three blind reviews, merged

Three domain-model agents (A, B, C) each reviewed `docs/domain-model.md`
as it stands after commit bbc5f07. They checked it against itself and
against all of `docs/srs`, the invariants and `features/`, with
identical prompts and no shared context. The count after each finding
says how many found it on their own. "v2" points to a decision already
taken in `plan/2610012322_cairn-for-agent-fleets/domain-model-v2.md`
but not yet applied.

## 1. Contradictions inside the model

| #   | Finding                                                                                                                                                                                                                            | Found | Settled by    |
| --- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----- | ------------- |
| M1  | **Owner act vs room act.** The owner-act list includes kick, bar, read only, unbar, join, leave, present, pick and pin changes. LANE-24, LANE-31 and §9.5 make them room acts, signed with a seat key. Room act has no definition. | 3/3   | v2 D11        |
| M2  | **"Owner" has two meanings:** the room holder (Owner), and any person (owner key, owner act, presence check, away policy, risk acceptance).                                                                                        | 3/3   | v2 D1, D3     |
| M3  | **Witness run.** "A node that authored nothing" vs LANE-05 and Bound human: "whose bound human authored no change in the room".                                                                                                    | 3/3   | v2 D4 (check) |
| M4  | **Seat key vs writer key.** Never related to each other, and "seat key" is undefined.                                                                                                                                              | 3/3   | v2 D12        |
| M5  | **Bot vs "every seat belongs to one person".** A bot's key chains to no person.                                                                                                                                                    | 2/3   | v2 D2, D3     |
| M6  | **Events with no seat's writer:** owner acts, expire acts, node-level `operator` events (repository bind, purge tombstones).                                                                                                       | 2/3   | v2 D11, D19   |
| M7  | **Foreign room vs recall relation.** The relation widens recall to every room the agent holds a seat in; Foreign room and RCL-05 exclude foreign rooms even with a seat.                                                           | 2/3   | v2 D10        |
| M8  | **Pin author.** The model says the author is a seat, yet intent, verdict, CLI and configuration pins have none. "Its author's personal room" makes no sense for a seat.                                                            | 2/3   | v2 D17        |
| M9  | **Landmark bound to a `seq` range;** it should be an address range (LMK-02).                                                                                                                                                       | 2/3   | mechanical    |
| M10 | **Personal room is one per node,** so "its author's personal room" must say which node's.                                                                                                                                          | 2/3   | v2 (open 2)   |
| M11 | **Personal-room seat exists with no add,** but a seat "holds its place while its add stands".                                                                                                                                      | 1/3   | mechanical    |
| M12 | **Span is "one run's events",** and a run spans writers; landmarks and the `spans` table bind one writer's range.                                                                                                                  | 1/3   | **new**       |
| M13 | **Bound human overlaps Person.**                                                                                                                                                                                                   | 1/3   | **new**       |
| M14 | **Repository and branch "hold no Cairn state",** yet bundles travel as git refs, trailers go into commits, and PEER-08 puts segments in a git remote.                                                                              | 1/3   | **new**       |
| M15 | **Room merge is "the one rule",** yet LANE-09 adds an ADR-chosen rule for room metadata and LANE-31 has a pick-order exception.                                                                                                    | 1/3   | v2 D18        |
| M16 | **"Stop accepting events" is widening,** yet "reject a foreign room" is listed as cut.                                                                                                                                             | 1/3   | **new**       |
| M17 | **The model says the SRS defines no terms,** yet Provenance defers to SRS §5.2.                                                                                                                                                    | 1/3   | mechanical    |

## 2. Undefined terms

Found by all three: **member**, **actor**, **component** (run, room
view, peer, publish, bridge), **peer**, **seat key**, **operator** as a
person.

Found by two: **device**, **result/outcome**, **owner surface**,
**acceptance grant**, **child agent**, **room alias**, **away mode**
(overlaps away policy), **focus set**, **fork room**, the **Fleet,
Health, Setup and Peers** screens, **origin** (overlaps provenance),
**token**, **comment / room board**, **poster**.

Found by one: receipt, compaction, worktree, checkpoint, active and
candidate pin, trusted boundary, structural events, recall taint,
publisher, reviewer, closed room, deployment mode, trust policy.

v2 already defines member (D7), author for actor (D8), device and node
(D9), outcome (D13), seat key (D12) and the launcher for the run
component (D4). The rest are **new**: each needs a concept in the
model, or must be dropped from the text.

## 3. Drift in the SRS and scenarios

| #   | Finding                                                                                                                                                                                                 | Found | Settled by     |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----- | -------------- |
| S1  | **"Operator" for the person running Cairn:** I1, I5, I6, PIN-01, OPS-03, SEC-31, VIEW-09, §9.5 and 60–80 scenario steps ("the operator runs"). lane.feature:118 mixes it with the room role.            | 3/3   | v2 D16         |
| S2  | **"Run" for a command or tool run,** plus the run component, `cairn run` and witness run.                                                                                                               | 3/3   | v2 D4          |
| S3  | **VIEW-22: a verdict leaves "as a commit or a pin".** The model allows only a pin.                                                                                                                      | 3/3   | mechanical     |
| S4  | **"Session" beyond the allowance:** ASM-03, ASM-14, PRV-08 "session title", assumptions.feature, provenance.feature:164 (A found none; B and C did).                                                    | 2/3   | **new**        |
| S5  | **"Writer" for an author or a node's log:** RCL-11, recall.feature:141, OWN-08, LANE-10, NG9.                                                                                                           | 3/3   | v2 D12         |
| S6  | **LANE-12 "endorse queue"** vs Needs you as the one queue.                                                                                                                                              | 2/3   | v2 D14         |
| S7  | **§4.4 says a `harness` event;** the class is `harness_text`.                                                                                                                                           | 2/3   | mechanical     |
| S8  | **§9.2 parameters `seq`, `seq_from` and `seq_to` carry addresses.**                                                                                                                                     | 2/3   | **new** (name) |
| S9  | **Pin removal has four names:** remove, end, unpin, `/unpin`.                                                                                                                                           | 2/3   | **new**        |
| S10 | **"Rooms land"** (09b, 02, OWN-21/27) and VIEW-13 compares rooms.                                                                                                                                       | 1/3   | v2 D5          |
| S11 | **Seat kinds "person, agent or bot"** in LANE-23 vs device, run or bot.                                                                                                                                 | 1/3   | v2 D2, D9      |
| S12 | **`/intent` and `/pin` typed at the harness prompt are owner acts,** but OWN-02 says harness input never is.                                                                                            | 1/3   | v2 D19         |
| S13 | **"Notice" also names** the Notification hook payload, an envelope field, and RCL-06/08 and CMP-06 markers.                                                                                             | 1/3   | **new**        |
| S14 | **The change log and ADRs use participant, judge, tenant and session,** where the model says "Nowhere".                                                                                                 | 1/3   | mechanical     |
| S15 | **Small items:** §11.1 baselines B0/B1 clash with boundaries B0/B1; "presence" has three meanings; "user's home"; "claim of work" vs the `claim` evidence class; CON-05 "rebuilt from segments" vs I10. | 1/3   | mechanical     |

## 4. Invariants that need new wording

All three agree that each of these needs an ADR and a security review.

- **I1, I5, I6:** "operator" as a person (v2 D16).
- **I2:** "owner" as a person, and the owner act and room act split
  (v2 D1, D3, D11); "trusted boundary", "structural events" and
  "poster" are undefined.
- **I4:** "owner" as a person, and "owner" next to "tenant" for one
  person. Its core list leaves out the TUI that the model's Core
  includes.
- **I8:** "tenant" stays allowed for now.
- **I3, I10** (one reviewer): "pinned constraints" and "active pins"
  are broader than PIN-10's rule on which pins restore.

## 5. What this means

- 17 of the findings, including every 3/3 one but the verdict wording,
  are settled by v2 decisions you already took. They go away when v2
  is applied.
- 7 are mechanical fixes with no decision needed: M9, M11, M17, S3,
  S7, S14, S15, and the witness-run wording if v2 keeps LANE-05's rule.
- 8 are new and need you: M12, M13, M14, M16, S4, S8, S9, S13, and the
  undefined terms v2 does not cover.

## 6. Stakeholder decisions, 6 October 2026

The eight new questions, decided one by one:

1. **Span (M12):** a span never crosses writers. LMK-01 gains a
   boundary: the run's events move to another seat's writer. Every
   landmark keeps one address range, and a room's landmarks count only
   that room's events.
2. **Bound human (M13):** merged into Principal. Each principal is
   known by one root key (a person's owner key, a service account's own
   key); every device and seat key chains to it, and Cairn counts
   principals by root key. LANE-05 reads "a node whose principal
   authored no change".
3. **Repository (M14):** a repository may carry Cairn data: commit
   trailers (LANE-28), bundles as refs, and encrypted segments in a
   namespaced location the owner enabled (PEER-08). The model says no
   more; where state lives stays with I1, I8 and I10. The SRS is
   unchanged.
4. **Reject a foreign room (M16):** stays cut. The catch-all rule
   narrows to acts that stop this node recording its own runs' events,
   such as turning capture off, retiring a writer or pausing ingestion,
   which stay widening.
5. **Session (S4):** "session" may appear only as "harness session" or
   inside a harness name (hooks, fields, record types such as a session
   title, "allow for session") when describing the harness, never as a
   Cairn unit.
6. **Address parameters (S8):** `get(address)` and `expand(range)`,
   where range takes any shown range form or two addresses; the
   envelope field becomes `address`, and a tombstone carries `range`.
7. **Ending a pin (S9):** the act is "unpin": the pin stops restoring
   and its versions stay in the record. "End" and "remove" are
   reworded; "removes from a restore block" stays as the effect several
   acts share.
8. **Notice (S13):** "notice" names only the pushed, opt-in room
   notice. Recall returns a tombstone or a quarantine marker, kernel
   output carries a truncation marker, the envelope's field becomes
   `warning`, and §9.1 speaks of the harness's notification text.
