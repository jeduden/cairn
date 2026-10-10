# Repair brief: make a new version of the annotated model

Folder GA = research/notes/domain-model-subset-search (the caller gives its absolute path).
GA/annotated/ holds the domain model and invariants with feature spans
⟦f:ID⟧ … ⟦/f⟧ (see GA/annotate-brief.md for the marker rules and
GA/features.json for the features). Candidates are assembled from it by
removing the spans of excluded features (GA/assemble.py).

You get a list of verified findings (each with the candidate it came from, a
quote, the problem, a suggested fix and the feature it is attributed to). For
each finding, edit GA/annotated/ so the defect is gone in EVERY candidate that
includes that feature, and nothing changes for candidates without it:

- Edit text inside the attributed feature's span (or core text, for core
  findings). A fix that needs another optional feature's concept must sit in
  a span of that feature too (nest it), so candidates without it do not
  mention it.
- Keep markers balanced; never remove a span or move text between features
  unless the finding is exactly that the text sits in the wrong feature.
- Prefer the smallest wording change. You may also delete a clause that
  causes a contradiction when nothing else needs it.
- Never weaken an invariant's protection to remove a finding: tighten the
  concept instead, or leave the finding unrepaired and say why.
- If a finding is wrong (the candidate already holds), skip it and say why.

Edit a copy of this folder, never the repository's own (see the latest
rulings for how), so a half-edited version never lands in a commit. The
annotated sources end in `.md.ann`.

After editing, run `python3 GA/check_annotation.py --no-orig` (balance and
ids only) until it prints OK, then assemble each named candidate
(`python3 GA/assemble.py GA/genomes/<name>.json GA/out/<name>`) and read the
repaired places there.

Report: per finding, repaired / skipped (why), and the exact before → after
text.
