# Patterns in the finding ledger

The stakeholder asked for the ledger's patterns before deciding on the
[process proposal](process-evaluation.md). This note reads the 107
finding themes (the 18 settlement themes DM-CJ to DM-DA aside), the 264
raw findings of rounds 1 to 7, the 84 verdicts and their 367 drafted
edits.

Method: the topic, chain and lifecycle labels are a hand classification
of each theme's title and verdict. A finding counts as landing on the
loop's own text when its quote shares a six-word run with a sentence an
earlier verdict or the M1–M5 settlement added. That is a lower bound,
since quotes are often paraphrased.

## Nine chains hold most themes

67 of the 107 themes (63%) sit in nine chains. A chain is one rule fixed
one case at a time: each fix covers the reported case, and a later
review finds the next case beside it. Six chains began in round 0, and
five still yielded a theme in round 7.

| Chain                            | Themes | Rounds           | Deferred |
| -------------------------------- | ------ | ---------------- | -------- |
| Backup restore, clone, identity  | 9      | 1, 6, 7          | 2        |
| Which pin version restores       | 9      | 0, 1, 2, 3, 5, 7 | 0        |
| Agent identity and recall taint  | 8      | 0, 1, 2, 3, 5, 7 | 1        |
| Configuration and its acceptance | 8      | 0, 1, 4, 5, 7    | 3        |
| Key chains to principal keys     | 8      | 0, 1, 2, 3       | 1        |
| Foreign rooms and later runs     | 8      | 1, 3, 4          | 1        |
| Run-seat key loss (OQ-43)        | 6      | 0, 2, 3, 5       | 4        |
| Branch links                     | 6      | 0, 1, 2, 5       | 1        |
| Quarantine reach                 | 5      | 2, 3, 7          | 0        |

- Restore: DM-T, AH, DD, DE, DG, DH, DI, DJ, DM.
- Pin version: DM-K, R, S, AL, BB, BD, BL, CE, DS.
- Recall taint: DM-E, AK, AR, AV, BJ, BK, CC, DN.
- Configuration: DM-F, Z, BT, BX, CF, CH, CI, DR.
- Key chains: DM-L, V, AF, AP, AU, BA, BF, BP.
- Foreign rooms: DM-AC, AE, AG, BC, BM, BN, BW, BZ.
- Run-seat key: DM-D, AN, AZ, BG, CD, CG.
- Branch links: DM-M, X, AO, AT, AX, CB.
- Quarantine: DM-AS, AW, BI, BS, DQ.

## Lifecycle events crossed with derived state

44 themes (41%) ask what one lifecycle event does to another concept. By
event: leave 6, restore 6, key loss 5, join 5, revoke 4, resume 4,
retire 3, and two each for expiry, clone, quarantine, handover and
purge. A node identity change adds one more. The events form a closed
list, and I10 lists the derived artifacts, so the space is finite. At
13 events by the dozen artifacts I10 lists it is large, though, and a blind review
samples it at random.

## Fixes feed the next round

- 38 of the 105 needs-fix findings (36%) quote text the loop added: 24
  of 47 in regression passes, and 14 of 58 in full rounds.
- In full rounds the share rises: 4 of 32 findings in rounds 1 to 3,
  and 10 of 26 in rounds 4 to 7.
- A fix drafts about 110 words over 4.4 edits: 1.6 in the model, 1.8 in
  the SRS and 1.0 in scenarios. In rounds 6 and 7 a fix drafts 134
  words, so fixes grew rather than shrank.
- The decision on DM-DE, that restored events count as received, raised
  four themes in its own regression pass (DM-DG to DM-DJ). A fifth,
  DM-DM, came in round 7. Each was a consequence that was not carried
  through.

## The model lags the SRS

20 of the 84 verdicts are `settled`: the SRS already answered the
question, and the model did not say so. That was 12 of 67 verdicts in
rounds 1 to 5 (18%), and 8 of 17 in rounds 6 and 7 (47%). In total, 69
verdicts settle by citing a requirement. Once the scope narrowed, half
the defects were the model not repeating the SRS.

## Findings follow where the review looks

Before the narrowing, 54 of 84 themes arose only from M6 to M9
features. These include 10 of the 12 room themes, all 9 peer themes
and 8 of the 11 seat and key themes of those rounds. In rounds 6 and 7 the
themes moved onto M1 machinery that had barely been read before:

- backup restore and clone, with seven themes (two before round 6);
- what shows the record in M1–M5, with three themes;
- harness input and its trust, with three themes.

## Hot spots by file and invariant

- places.md, principals-and-agents.md, trust-and-flow.md and record.md
  draw 66% of the needs-fix findings. 05-functional is the most edited
  file, with 40 edits.
- By invariant: I2 33 themes, I8 22, I3 14, I10 12, I5 8, I7 6, I6 5,
  I1 and I4 3 each, I9 1. I2 and I8 together carry half.

## Gates that do not filter

- The verifier judged all 84 themes real. It drafted fixes, but it
  never refuted a theme.
- The reviewers' type labels do not sort severity. DM-DQ, kernel
  variables serving quarantined content, breaks I5, yet it was
  labelled a gap. DM-DO was labelled security, yet it needed only a
  missing sentence.
- Only 3 of the 105 needs-fix findings matched a closed theme, so the
  ledger holds.
- Minor findings run from 6 to 17 a pass (155 in all), with no
  downward trend.

## Open debt inside M1–M5

4 of the 23 deferred themes lie in M1–M5 scope. Three are under OQ-43
(M1), which heads the run-seat key chain, open since round 0. The fourth is OQ-49
(M5).

## What the patterns mean for the proposal

1. **Severity bar.** It is needed, but it does not end the loop alone.
   By the reviewers' labels, rounds 4 to 7 would still each have had
   two or three blocking themes. By my reading, round 7 had three that
   break an invariant or contradict: DM-DN, DM-DQ and DM-DS. A stop rule that waits for zero will not end, so
   the last round should be fixed in advance. Its blocking themes are
   fixed and become scenarios, judged by a verifier that defaults to
   "not blocking".
2. **Concepts in the model, rules in the SRS.** The 47% settled share
   and the 1.6 model edits per fix support it.
3. **No regression pass.** Most of its findings sit on the round's own
   text, and the next full round re-reads that text.
4. **Implementation as oracle.** The lifecycle grid tells the scenarios
   what to enumerate: one outline per lifecycle event, with one example
   row per derived artifact.

The patterns add three more:

5. **Fix a chain at its rule.** One pass per open chain (restore, pin
   version, recall taint, configuration, quarantine) states the general
   rule once in the SRS. For example, after any restore or clone, every
   derived artifact is rebuilt from the events.
6. **Sweep each decision.** Before the next review, a new design
   decision is walked against the lifecycle grid. The DM-DE follow-ups
   show what this would have caught at once.
7. **Settle OQ-43 before M1.** Spike S4 answers it, and its chain
   holds the most deferred themes.
