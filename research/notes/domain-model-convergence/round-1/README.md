# Round 1

Three blind lens reviewers read the full model under the fixed brief
after round 0 closed themes DM-A to DM-O (`result.json`). They reported
13 needs-fix findings and 13 minor ones. The matcher found that no
finding repeats a closed theme: each needs-fix finding is a new theme.

## Themes

The verify workflow (`../verify-workflow.js`, output in `verify.json`)
checked each new theme against the model and the SRS, cited what
settles it, and classed it.

| Theme | New | Closed as                                                         |
| ----- | --- | ----------------------------------------------------------------- |
| DM-R  | 1   | fixed: a stamp restores only once PRV-10 ships (I2)               |
| DM-S  | 2   | fixed: newest qualifying version; a grant never displaces a stamp |
| DM-T  | 3   | fixed: events before a node identity change count as received     |
| DM-U  | 4   | fixed: a quota's first crossing is a structural event (I10)       |
| DM-V  | 7   | fixed: the model carries SEC-32's own-words rule                  |
| DM-W  | 8   | fixed: a trust grant covers only within device scope (PRV-10)     |
| DM-X  | 9   | fixed: a room id derives from its creating seat's first key       |
| DM-Y  | 12  | fixed: the model carries LANE-23's subagent join                  |
| DM-Z  | 13  | fixed: repository configuration may not lower the pin budget      |
| DM-AA | 5   | deferred by the stakeholder: OQ-41, M7                            |
| DM-AB | 6   | fixed by the stakeholder: refuse unless all three denied          |
| DM-AC | 10  | fixed by the stakeholder: restore follows the agent id            |
| DM-AD | 11  | fixed by the stakeholder: the owner's join assigns contributor    |

DM-AB supersedes round 30's decision Q4, which let the person confirm a
witness check the platform could not confine to its homes.

From round 0, DM-N and DM-P closed when the stakeholder removed the git
carrier, and DM-Q is accepted as residual risk R7 under the stakeholder's
OQ-29 decision: no software surface stops an unsandboxed agent of the
same OS user.

## Regression pass

A blind review of the entries round 1 edited (`regression.json`) found
ten needs-fix findings in nine new themes; six sat beside round 1's own
closing texts. The verify workflow (`regression-verify.json`) classed
them, and they close as follows.

| Theme | Closed as                                                           |
| ----- | ------------------------------------------------------------------- |
| DM-AE | fixed: Foreign room restores to the runs Pin names                  |
| DM-AF | fixed: a standing grant stops covering a relaying key (I10)         |
| DM-AG | fixed by the stakeholder: a leave, kick or bar drops no pin         |
| DM-AH | fixed: quarantines survive a node identity change (I5)              |
| DM-AI | fixed: a witness check denies every write outside its checkout      |
| DM-AJ | fixed: the facilitator's pins are in its own words too              |
| DM-AK | fixed: a resumed run carries the recall taint it resumes            |
| DM-AL | fixed by the stakeholder: the run's own rooms fill the budget first |
| DM-AM | fixed by the stakeholder: a subagent joins its parent's rooms       |

DM-AM reverses round 1's DM-Y rule that a subagent joins only on
acceptance; DM-Y's quotes follow the new sentence, as DM-AC's and
DM-E's follow the sentences later decisions extended.

## Count

Open themes fell from 2 (DM-P, DM-Q) at the end of round 0 to 0. With
its regression pass, the round opened 22 themes and closed 25. New
themes per pass have gone 15 (round 0), 13 (round 1) and 9 (its
regression pass), so discovery is falling slowly, and most regression
themes come from the round's own edits.
