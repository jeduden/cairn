# Round 3

Three blind lens reviewers read the full model under the fixed brief
after round 2 and its regression pass closed every theme
(`result.json`). They reported 11 needs-fix findings in 10 new themes,
none of a closed theme. The verify workflow (`verify.json`) classed
them, and they close as follows.

| Theme | Closed as                                                             |
| ----- | --------------------------------------------------------------------- |
| DM-BC | fixed: a kept foreign-room pin stops on revocation or a void grant    |
| DM-BD | fixed: the new owner's stamp of the intent adds only its own agents   |
| DM-BE | fixed: a personal room's segments go only to its principal's nodes    |
| DM-BF | fixed: a seat chaining to neither principal key is a member of none   |
| DM-BG | fixed: only the run seat's certifier certifies the seat beside it     |
| DM-BH | fixed: four event kinds gain their provenance                         |
| DM-BI | fixed: trusted-only exports leave out cut and neutral acts' free text |
| DM-BJ | fixed: non-recall tool results carry no record text                   |
| DM-BK | deferred by the stakeholder: OQ-43, M1 (a resume without an agent id) |
| DM-BL | fixed: a purged pin version counts as absent                          |

## Regression pass

A blind review of the entries round 3 edited (`regression.json`) found
7 needs-fix findings in seven new themes, closed under the goal's
defer-first rule (`regression-verify.json`).

| Theme | Closed as                                                            |
| ----- | -------------------------------------------------------------------- |
| DM-BM | fixed: Foreign room names the kept trust-grant pins as trusted       |
| DM-BN | fixed: the intent follows LANE-11 in a foreign room too              |
| DM-BO | fixed: a token-key-only node gets only its token's rooms' segments   |
| DM-BP | deferred: OQ-41, M7 (a listing's reach to another principal's node)  |
| DM-BQ | deferred: OQ-14, M8 (how a delegation travels between principals)    |
| DM-BR | deferred: OQ-41, M8 (a personal room's cross-room posts)             |
| DM-BS | fixed by the stakeholder: a purge's tombstone keeps the act's effect |

DM-BS could not wait (quarantine and retention purges ship in M1), so it
went to the stakeholder; its record.md edit also drops two sentences
that repeated I10's list and ADM-06, to stay within budget.

## Count

Open themes stay at 0: with its regression pass, the round opened 17 and
closed 17. New themes
per full round have gone 15, 13, 6 and 10; the regression passes found
9, 9 and 7. About half of round 3's themes sat beside the last decisions
(DM-BC, DM-BF, DM-BL, DM-BK); the rest were older gaps in text no round
had edited.

An earlier version of this note stopped the loop here because the
full-round count rose. The goal stops only when a round opens more than
it closes, so the round was finished instead.
