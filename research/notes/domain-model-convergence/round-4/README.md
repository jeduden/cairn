# Round 4

Three blind lens reviewers read the full model under the fixed brief
after round 3 and its regression pass closed every theme
(`result.json`). They reported 4 needs-fix findings in four new themes;
the consistency lens found none. Under the goal's defer-first rule
(`verify.json`) they close as follows.

| Theme | Closed as                                                           |
| ----- | ------------------------------------------------------------------- |
| DM-BT | fixed: flags, redaction and the deployment mode wait for acceptance |
| DM-BU | fixed: the facilitator's program writes no pin that restores        |
| DM-BV | deferred: OQ-41, M8 (acts past a retired writer's last seq)         |
| DM-BW | deferred: OQ-45, M8 (recall into a room the agent's seats left)     |

## Regression pass

A blind review of the entries round 4 edited (`regression.json`) found
five needs-fix findings in three new themes. The verify workflow
(`regression-verify.json`) classed them, and they close as follows.

| Theme | Closed as                                                        |
| ----- | ---------------------------------------------------------------- |
| DM-BX | deferred: OQ-06, M8 (may a token key's access token name a mode) |
| DM-BY | fixed: only a writer's own principal retires it by an act        |
| DM-BZ | fixed: the Agent entry names restore's and recall's rooms apart  |

DM-BX carries the rule PRV-04, ADM-04, PRV-10 and SEC-22 already imply:
a token-key-only node stays `automation` unless managed policy fixes
it. To keep `record.md` under its token budget, DM-BY folds the new
rule into the entry's existing sentence and leaves PEER-05's
before-PRV-10 clause to the SRS; the Origin entry's recorder list now
points at the `witnessed` list instead of repeating it, and the Purge
entry's pin-version sentence drops a repeated clause. No meaning
changes.

## Count

Open themes stay at 0: the round and its regression pass opened 7 and
closed 7. New themes per full round have gone 15, 13, 6, 10 and 4;
per regression pass 9, 9, 7 and 3.
