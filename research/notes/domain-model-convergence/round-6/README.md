# Round 6

The first round under the brief narrowed to M1–M5 (`../m1-m5-scope.md`).
Three blind lens reviewers (`result.json`) reported 7 needs-fix findings,
13 minor ones and 2 of severity `later`. Three needs-fix findings repeat
closed themes (DM-CD twice, DM-BK); the other four form three new
themes, which close as follows (`verify.json`).

| Theme | Closed as                                                                      |
| ----- | ------------------------------------------------------------------------------ |
| DM-DD | fixed: a node clone's carried-over quarantines are its home's own (REC-24)     |
| DM-DE | fixed by the stakeholder: a backup restore's events count as received (ADM-06) |
| DM-DF | fixed: the CLI shows the record and the counters before the room view          |
| DM-DB | deferred: OQ-47, M7 (`later`: a join's consent and other principals' links)    |
| DM-DC | deferred: OQ-47, M7 (`later`: a room's visibility at creation)                 |

DM-DE could not wait: backup restore ships in M3 and seals in M7, so an
altered copy could plant trusted events in between. Whether a verified
seal may keep a restored event `witnessed` is deferred to OQ-48 (M7).

## Regression pass

A blind review of the entries round 6 edited (`regression.json`) found
eight needs-fix findings in five new themes, four of them next to the
backup-restore decision. The verify workflow (`regression-verify.json`)
classed them; one edit of DM-DJ was merged by hand where DM-DH and DM-DI
had changed the same sentences.

| Theme | Closed as                                                                 |
| ----- | ------------------------------------------------------------------------- |
| DM-DG | fixed: a backup restore rebuilds every derived artifact (I10, ADM-06)     |
| DM-DH | fixed: the restore applies the copy's quarantines as a request (I5)       |
| DM-DI | deferred: OQ-45, M8 (another node's personal room after a restore)        |
| DM-DJ | fixed: carried-over widening acts lapse until taken again (I8, REC-24)    |
| DM-DK | fixed: the CLI shows control and bidirectional characters as placeholders |

## Count

Open themes stay at 0: the round and its regression pass opened 10 and
closed 10. Under the narrowed brief the count of clean rounds starts here.
