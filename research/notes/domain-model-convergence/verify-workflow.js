export const meta = {
  name: 'dm-round-verify',
  description: 'Verify each new round theme against the model and SRS, cite what settles it, propose exact tightening edits',
  phases: [{ title: 'Verify', detail: 'one checker per theme cluster' }],
}
// args: { repo, themesFile, clusters, decisions? }
//   themesFile: JSON keyed by new theme id (title, matcher note, finding).
//   clusters: lists of theme ids, one checker per list.
//   decisions: optional text naming stakeholder decisions taken since.
const { repo, themesFile, clusters, decisions } = args
const OUT = {
  type: 'object',
  properties: { themes: { type: 'array', items: { type: 'object', properties: {
    id: { type: 'string' },
    real: { type: 'boolean' },
    why: { type: 'string', description: 'why it is or is not a real defect in the current text, quoting the model' },
    settled_by: { type: 'string', description: 'requirement id, ADR or stakeholder decision that already settles it, with its exact quoted text; or "none"' },
    klass: { type: 'string', enum: ['settled', 'tighten', 'design', 'not-real'] },
    options: { type: 'array', items: { type: 'object', properties: {
      label: { type: 'string' }, effect: { type: 'string' }, invariant_change: { type: 'boolean' },
    }, required: ['label', 'effect', 'invariant_change'] } },
    recommendation: { type: 'string' },
    edits: { type: 'array', items: { type: 'object', properties: {
      file: { type: 'string' }, old: { type: 'string' }, new: { type: 'string' },
    }, required: ['file', 'old', 'new'] } },
    ledger_quote: { type: 'object', properties: { file: { type: 'string' }, quote: { type: 'string' } }, required: ['file', 'quote'] },
  }, required: ['id', 'real', 'why', 'settled_by', 'klass', 'options', 'recommendation', 'edits', 'ledger_quote'] } } },
  required: ['themes'],
}
const CONTEXT = `Cairn's domain model is docs/srs/invariants.md, docs/domain-model/index.md and the concept files in docs/domain-model/. The SRS is docs/srs/ (requirement tables; §9.6 is settings keys; §12 the delivery plan with milestones). A convergence loop reviews the model blind; its finding ledger is plan/2610012322_cairn-for-agent-fleets/finding-ledger.json (themes DM-A..DM-Q, already closed; read it to avoid contradicting a closing text). Stakeholder decisions already taken: B admission/invites/bars deferred to PRV-10 (M7, OQ-42); D lost run-seat key deferred to ASM-21 / S4 (OQ-43); E agent identity is a stable harness agent id, later runs of an agent inherit its rooms for recall and directed posts, never a seat; L service-account certificates countersigned, listings recorded in the key set; G access-token expiry is a refusal; I a pin candidate is confirmed only after its exact text is shown; the git carrier is removed entirely (PEER-08 retired). Rules for any edit you propose: edits only tighten, delete or clarify; never drop a feature; never weaken an invariant (an invariant wording change needs an ADR, so mark such an option invariant_change true); the model and the SRS change together, so give the edit in both where both state the rule; keep prose to 80 columns where you can, and keep each model file under 250 lines and 3000 tokens (say if an edit risks that). Use the SRS's and model's own terms. Every "old" must be an exact substring of the current file, unique in it, so it can be applied with a string replace; keep edits minimal.`
phase('Verify')
const results = await parallel(clusters.map((ids, k) => () => agent(
  `${CONTEXT}${decisions ? ' Later decisions: ' + decisions : ''}\n\nRepository: ${repo}. Edit no file.\n\nFor each theme below (a needs-fix finding from a blind review of the model, with the matcher's note), do this: (1) Check it against the current text: is the defect real? Read the model entries it names and everything they cite. (2) Search the SRS (grep the requirement ids and terms) and the ADRs under docs/adr for a rule that already settles it; quote it exactly or say none. Treat the matcher's settled_by as a lead to verify, not a fact. (3) Classify: settled (the SRS or a decision already says it; the model only needs to carry it), tighten (a clarifying or tightening rule that follows from the invariants and existing rules with no real behavioral choice), design (a real choice the stakeholder must make: give 2 or 3 options with their effect, flag any that changes invariant wording, and recommend one, preferring safety where usefulness and safety conflict), or not-real. (4) For settled and tighten, and for your recommended option of a design theme, give exact edits to the model and the SRS, and the one sentence of new model text the ledger will cite (ledger_quote, an exact substring of your new text, 10 to 200 characters).\n\nThemes: read ${themesFile}, a JSON object keyed by theme id (title, the matcher's settled_by and reason, and the finding or findings with file, quote, problem and fix); yours are ${ids.join(', ')}.`,
  { label: `verify:${ids.join(',')}`, phase: 'Verify', schema: OUT },
)))
return results.filter(Boolean).flatMap(r => r.themes)
