# Domain-model subset search

The search for the largest subset of Cairn's domain model that is
consistent, secure and useful. A candidate subset is assembled from the
model by dropping whole features, reviewed blind, scored, and repaired.
The search stops when a candidate has no findings and adding any
excluded feature brings findings back.

Status: in progress, selection only. The stakeholder asked for it during
the blind domain-model reviews of the cairn-for-agent-fleets plan, and then
ruled that the search selects feature combinations of the current model and
never changes its text. `annotated/` is the original model with feature
spans; `check_annotation.py` (run without `--no-orig`) proves it equals
`docs/domain-model/` and `docs/srs/invariants.md` byte for byte. Nothing
here is normative.

Versions v1 to v6 (`annotated-v1/` … `annotated-v6/`, the repair rulings and
`invariant-edits.md`) record an earlier phase that also repaired the text.
They are kept as history; the selection search does not use them.

## Method

1. **Genes.** `features.json` splits the model into a core and 47
   optional features. Each has its entries (concept items), the
   features it requires, soft links, a value and a risk. `items.json`
   and `inventory.json` list every list item of the model.
   `coverage.py` checks that each item belongs to exactly one feature.
2. **Annotated sources.** `annotated/` holds the ten concept files, the
   hub and the invariants. Spans `⟦f:ID⟧ … ⟦/f⟧` mark each feature's
   text and nest where a clause needs two features. The files end in
   `.md.ann` so that mdsmith leaves the markers alone.
   `annotate-brief.md` gives the marker rules, and `check_annotation.py`
   checks balance, ids and dangling terms.
3. **Assembly.** `assemble.py <genome> <outdir>` keeps the core and the
   genome's features, drops every other span and regenerates the hub's
   catalog. It is deterministic. Its closure check reports a term
   defined only in a dropped feature that is still used.
4. **Evaluation.** Blind reviewers read only the assembled
   `candidate.md` through one lens: consistency, security or
   usefulness (`eval-brief.md`, `eval-workflow.js`). Each finding
   quotes the candidate. `attribute.py` maps the quote back to the
   innermost feature span, so a finding blames one feature or the core.
5. **Fitness.** `ga.py` keeps the ledger (`ledger.json`) and scores a
   candidate as value − 4 × needs-fix findings − 0.5 × dangling terms.
   It also does closure, blame, mutation and crossover.
6. **Selection GA.** `ga_round.py` fits each feature's needs-fix rate per
   review from the ledger (`--only v0`: reviews of the original text), then
   breeds genomes by crossover and mutation, closed under `requires`. Each
   pick is a Thompson sample: every rate is drawn from its posterior, so
   rarely reviewed features get explored and well-measured ones settle.
   `ga-select.js` runs one generation as a workflow: breed, one blind probe
   review per candidate, record and attribute the findings, commit. The
   best candidates get a strong review to confirm.

## Results so far

Strong evaluations ran three blind reviewers, one per lens, on the
session's default model. Probes ran one reviewer covering all three
lenses on a faster model; they find fewer issues and confirm nothing.

| Version | Candidate    | Features | Needs fix | Minor | Review |
| ------- | ------------ | -------- | --------- | ----- | ------ |
| v0      | g0-core      | 0        | 23        | 18    | strong |
| v0      | g0-lowrisk   | 11       | 25        | 17    | strong |
| v0      | g0-all       | 45       | 22        | 16    | strong |
| v1      | g1-core      | 0        | 10        | 8     | strong |
| v1      | g1-all       | 45       | 9         | 18    | strong |
| v2      | g2-core      | 0        | 5         | 17    | strong |
| v2      | g2-local     | 27       | 3         | 3     | probe  |
| v2      | g2-localplus | 29       | 3         | 6     | probe  |
| v2      | g2-localpeer | 31       | 1         | 6     | probe  |
| v2      | g2-nobrv     | 44       | 1         | 7     | probe  |
| v3      | g3-core      | 0        | 10        | 10    | strong |
| v3      | g3-nobrv     | 44       | 13        | 15    | strong |
| v3      | g3-all       | 45       | 9         | 13    | strong |

What the runs show:

- Repairs cut the core's needs-fix findings from 23 to 5 by v2. The v3
  repairs added concepts (lineage acceptance, predecessor rules), and
  the core rose back to 10. Adding text to fix a finding tends to
  create new findings.
- The single-lens probes looked clean where the strong reviews were
  not: g2-nobrv had 1 needs-fix finding on probe, while g3-nobrv had
  13 under strong review. Only a strong review can confirm a clean
  candidate.
- Most findings sit in concepts a single node never needs: node clone,
  predecessors and seat-key revocation. Version 4 therefore makes the
  core minimal by moving these into two new optional features,
  `node-clone` and `key-revocation`. It also repairs nine recurring core
  defects (`repair-v4-rulings.md`). Without those features, the core
  invariants read exactly as in the repository.
- Blame per evaluation (`python3 ga.py blame`) is highest for
  browser-room-view, principal-keys, work-rooms and shared-rooms.

Next: confirm the v4 core with a strong review, repair it until it has
no needs-fix findings, then grow it batch by batch.

## Running it

From this folder, with Python 3 and nothing else:

```sh
python3 check_annotation.py                     # text unchanged, spans balance
python3 assemble.py genomes/g0-core.json out/g0-core
python3 ga.py rank                              # ledger by fitness
python3 ga_round.py v0-ga1 6 --only v0 --seed 1   # breed one generation
```

`out/` is derived and ignored by git. Rebuild any candidate from its
genome with the matching snapshot. Copy `annotated-vN/` over
`annotated/` to rebuild a candidate of an earlier generation exactly as
its reviewers read it. `eval-workflow.js` is a Claude Code workflow
script. It takes `{"ga": "<absolute path of this folder>", "jobs":
[{"cand", "lens", "model"?}]}`. `collect_wf.py` files its output into
`evals/` and the ledger.
