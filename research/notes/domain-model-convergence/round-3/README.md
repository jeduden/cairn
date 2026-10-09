# Round 3: the loop stops

Three blind lens reviewers read the full model under the fixed brief
after round 2 and its regression pass closed every theme
(`result.json`). They reported 11 needs-fix findings in 10 new themes,
none of a closed theme. The ledger keeps them open as DM-BC to DM-BL.

## Why the loop stops

The stakeholder bounded the loop: stop when themes stop falling. New
themes per full round went 15 (round 0), 13 (round 1), 6 (round 2) and
10 (round 3), so the full-round count rose. The regression passes found
9 and 9.

About half of round 3's themes sit beside the last decisions or
tightenings rather than in untouched text:

- DM-BC: the foreign-room freeze (DM-AG) meets revocations and relaying
  marks (DM-BA, DM-AF).
- DM-BF: the countersignature rule (DM-AU) lets a seat key chain to
  neither principal and slip a bar.
- DM-BL: a purged pin version, beside the quarantined one (DM-BB).
- DM-BK: a resume without a stable agent id, beside DM-AC and DM-E.

The rest are older gaps in text no round has edited yet: the
handover stamp (DM-BD), which ranges a peer may receive (DM-BE), the
seat ingest starts (DM-BG), provenance of four event kinds (DM-BH),
free text in trusted-only exports (DM-BI) and non-recall tool results
(DM-BJ).

## What the loop achieved

Across rounds 0 to 2, 54 themes closed: 46 fixed with model and SRS
text the ledger check holds in place, 6 deferred to an open question
and milestone (OQ-41, OQ-42, OQ-43) and 2 accepted as residual risk R7. Every closing text is
carried in `docs/` and checked by `TestFindingLedgerIsCarried` in the
drift suite.

## The pattern

Each round closes what it finds, and each closing sentence is a new
edge the next blind reader can probe. A model this dense in security
rules keeps yielding new needs-fix findings at roughly ten per strong
review, as the subset search measured before the loop began
(`../../domain-model-subset-search`). Driving the count to zero by more
rounds of the same shape is not converging.
