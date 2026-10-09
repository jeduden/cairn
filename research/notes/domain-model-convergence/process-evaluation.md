# Process evaluation after round 7

The stakeholder stopped the convergence loop after round 7 to judge
whether it works. This note records the numbers and the proposal.

## Numbers

New themes per pass (needs-fix findings grouped into themes):

| Round     | Full review         | Regression pass |
| --------- | ------------------- | --------------- |
| 0         | 17                  | –               |
| 1         | 13                  | 9               |
| 2         | 6                   | 9               |
| 3         | 10                  | 7               |
| 4         | 4                   | 3               |
| 5         | 6                   | 3               |
| 6 (M1–M5) | 3, and 2 later-only | 5               |
| 7 (M1–M5) | 9, and 1 later-only | not run         |

- The ledger holds 125 themes: 100 fixed, 23 deferred, 2 accepted, none
  open. Across every pass only 3 findings repeated a closed theme.
- Since the ledger started, the model grew from 14,826 to about 18,200
  words and the SRS by 9%. Three model files and three SRS files sit at
  their token budgets, so each fix now needs an offset.
- About 40% of the themes since round 0 came from regression passes,
  mostly at the edges of the round's own fixes.
- A round with its checks and regression pass costs about 2.5M agent
  tokens and two hours.

## What works

- Closed themes stay closed: the ledger and its check stop churn.
- It finds serious defects: an altered backup planting trusted events,
  a nested harness session recorded as trusted `user` input, kernel
  variables serving quarantined content, and a cloned node carrying an
  `interactive` acceptance.
- Narrowing to M1–M5 moved the findings onto what ships first.

## What does not

1. New themes per round do not approach zero, so the done rule (two
   clean full reviews in a row) is out of reach. A strong blind
   reviewer finds five to ten defects in any 18,000-word model, and the
   "two careful implementers" lens has an endless supply at this depth.
2. Each fix adds text, and new text brings new findings.
3. Regression passes find about as many themes as full rounds.
4. The brief asks the model to carry every rule it needs, so fixes copy
   requirement detail into concept entries and the model grows into a
   second SRS.

## Proposal

1. Done becomes a severity bar: a full round finds no break of an
   invariant (I2, I4, I5, I8, I10) and no contradiction. Other gaps go to
   a known-issues list carried into M1.
2. The model keeps concepts and relations, and rules stay in the SRS and
   scenarios. The brief stops treating a missing mechanism as needs-fix
   where a requirement is cited, and one deduplication pass trims the
   model under ledger entries.
3. The separate regression pass is dropped, because the next full round
   re-reads the edited entries.
4. Implementation becomes the oracle: the M1 scenarios are made concrete
   so they find the remaining gaps.

The recommendation is all four, with one final review against the new
bar. If it is clean, the M1–M5 model is done.
