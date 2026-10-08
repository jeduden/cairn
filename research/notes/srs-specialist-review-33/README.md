# SRS review by the domain-model specialists, round 33

A blind review of the SRS (`docs/srs/`, invariants included) and the
scenarios under `features/` against the domain model. Twelve reviewers
took part: the domain-model agent over the whole model, and one
specialist per model file (`.claude/agents/domain-model-*.md`). Each
read the model and the SRS at the PR head after the walkthrough
change, and none saw the others' reports.

Status: bundled, not yet applied. The verify, conflict-panel and fix
steps wait until the domain-model subset search
([../domain-model-subset-search](../domain-model-subset-search/)) has
settled the core.

## Files

- `whole.md`, `hub.md` and one report per model file
  (`acts-and-roles.md` … `trust-and-flow.md`): each reviewer's report,
  kept verbatim but for lint formatting.
- `all-findings.json`: the 129 findings extracted from the reports
  (`extract.py`), each with reviewer, term, quote, location, what it
  breaks, the proposed fix, and whether it touches an invariant or is
  hard to revert. `model-findings.json` keeps the ones on the model
  itself.
- `bundle.json` and `bundle.md`: the findings merged into 112 clusters
  (`build_bundle*.py`, `ie_block.py`). Each cluster has a class: 2 for
  the stakeholder (an invariant's wording), 29 to decide between
  options, and 81 to fix. They also hold the cross-file conflicts and
  the proposed instruction-file edits.
- `p2.txt` and `p3.txt`: working lists of the needs-fix findings and
  the questions, for the verify pass.
- `ids.txt`: the agent id of each reviewer, which `extract.py` reads.
