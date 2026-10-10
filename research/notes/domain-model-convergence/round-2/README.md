# Round 2

Three blind lens reviewers read the full model under the fixed brief
after round 1 closed every theme (`result.json`). They reported 8
needs-fix findings and 15 minor ones; the matcher found 6 new themes and
no finding of a closed one. The verify workflow (`verify.json`) classed
them, and they close as follows.

| Theme | Closed as                                                          |
| ----- | ------------------------------------------------------------------ |
| DM-AN | deferred by the stakeholder: OQ-42, M7 (a lost-key seat's leave)   |
| DM-AO | fixed: a foreign room's branch link claims no branch               |
| DM-AP | fixed: a managed-policy listing needs the listed key's countersign |
| DM-AQ | fixed: turning the peer component off stays cut, with its reason   |
| DM-AR | fixed: a subagent's recall taint reaches the run it reports to     |
| DM-AS | fixed: quarantine also stops trust-grant posts and delegated tasks |

## Regression pass

A blind review of the entries round 2 edited (`regression.json`) found
13 needs-fix findings in nine new themes, several of them older gaps
near the edited entries. The verify workflow (`regression-verify.json`)
classed them, and they close as follows.

| Theme | Closed as                                                            |
| ----- | -------------------------------------------------------------------- |
| DM-AT | fixed: a branch moves only when a room enters or leaves the set      |
| DM-AU | fixed: every certificate needs the certified key's countersignature  |
| DM-AV | fixed: a delegate on another node carries taint it cannot derive     |
| DM-AW | fixed: quarantine also keeps content out of trusted-only exports     |
| DM-AX | fixed: a verdict binds the branch heads named on the recording node  |
| DM-AY | deferred by the stakeholder: OQ-42, M7 (who is another principal)    |
| DM-AZ | deferred by the stakeholder: OQ-42, M7 (any principal's lost key)    |
| DM-BA | fixed by the stakeholder: a revocation voids grants naming the key   |
| DM-BB | fixed by the stakeholder: a quarantined pin version counts as absent |

## Count

Open themes stay at 0: with its regression pass, the round opened 15
and closed 15. New themes per pass have gone 15 (round 0), 13 (round 1),
9 (its regression pass), 6 (round 2) and 9 (its regression pass): full
rounds fall, while each regression pass finds about nine, partly gaps
beside the round's own edits and partly older ones it surfaces.
