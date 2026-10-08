# Domain-model convergence

The loop that drives the full domain model, every feature kept, to zero
open needs-fix findings. The stakeholder set it after the subset search
([../domain-model-subset-search](../domain-model-subset-search/README.md))
showed that leaving features out buys nothing, and after the history trace
([core-findings-history.md](../domain-model-subset-search/core-findings-history.md))
showed why 32 review rounds did not converge: decisions landed in the SRS
but not in the model, trims dropped rules silently, local fixes moved
problems, and nothing told noise from a real finding.

## How it converges

- **One ledger.** Every finding theme gets one entry in
  `plan/2610012322_cairn-for-agent-fleets/finding-ledger.json`, closed one
  of three ways: fixed, deferred (an open question and a milestone) or
  accepted (a §6.1 residual risk). Each closed theme cites the sentences
  that close it. `internal/ledger`'s `TestFindingLedgerIsCarried` fails
  when one of them leaves its file, and the drift suite proves it does.
  A later finding of a closed theme does not reopen it.
- **A fixed review.** Each round runs `round-workflow.js`: three blind
  reviewers, one per lens, read the model alone under `review-brief.md`,
  which stays the same between rounds. A matcher sorts each needs-fix
  finding into a closed theme, an open one or a new one.
- **Decisions.** A new theme that the SRS, an ADR or an earlier decision
  already settles is written in on that decision; invariant wording,
  ADR-level changes and new design choices go to the stakeholder in one
  batch with options.
- **Edits only tighten, delete or clarify**, in the model and the SRS
  together. No feature is dropped, no invariant weakened, no text trimmed
  without a ledger entry. A regression pass reviews the edited entries.
- **Monotonic.** A round that opens more themes than it closes stops the
  loop for a report.
- **Done** when two consecutive rounds on the unchanged model find no
  needs-fix finding outside the ledger, and the ledger check, mdsmith,
  `go test` and the drift suite pass.

## Rounds

Each round's findings and matches are kept in `round-<n>/`.
