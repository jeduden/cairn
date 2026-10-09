export const meta = {
  name: 'dm-convergence-round',
  description: 'One convergence round: strong blind review of the domain model at M1 to M5, findings matched against the finding ledger',
  phases: [
    { title: 'Review', detail: 'three blind reviewers, one per lens, on the full model' },
    { title: 'Match', detail: 'each needs-fix finding matched to a ledger theme or named as a new theme' },
  ],
}
// args: { repo, round, scope? }
//   repo: absolute path of the checkout to review (the committed model).
//   scope: optional list of model entries to review instead of the whole
//   model (the regression pass over edited entries).
const { repo, round, scope } = args
const BRIEF = `${repo}/research/notes/domain-model-convergence/review-brief.md`
const LEDGER = `${repo}/plan/2610012322_cairn-for-agent-fleets/finding-ledger.json`
const LENS = {
  consistency: "consistency: contradictions, undefined terms and gaps between the model's own statements (its invariants, hub relations and concept entries)",
  security: "security: does the model's own concept set uphold its own invariants? Look for paths from untrusted content to the model outside the closed paths, trust or key rules that can be bypassed, data leaving outside declared boundaries, silent failures",
  usefulness: 'usefulness: every concept has a use, and the core journeys (constraints restored verbatim after compaction, exact recall, a person sees what agents did, untrusted history never instructs) work with what the model defines',
}
const FINDINGS = {
  type: 'object',
  properties: { findings: { type: 'array', items: { type: 'object', properties: {
    type: { type: 'string', enum: ['contradiction', 'undefined', 'gap', 'security', 'useless'] },
    file: { type: 'string' }, quote: { type: 'string' }, problem: { type: 'string' }, fix: { type: 'string' },
    severity: { type: 'string', enum: ['needs-fix', 'minor', 'later'] },
  }, required: ['type', 'file', 'quote', 'problem', 'fix', 'severity'] } } },
  required: ['findings'],
}
const MATCHES = {
  type: 'object',
  properties: { matches: { type: 'array', items: { type: 'object', properties: {
    lens: { type: 'string' }, index: { type: 'integer' },
    theme: { type: 'string', description: 'the ledger theme id it belongs to, or NEW-<n> for a new theme' },
    kind: { type: 'string', enum: ['closed', 'open', 'new'] },
    reason: { type: 'string' },
  }, required: ['lens', 'index', 'theme', 'kind', 'reason'] } },
  new_themes: { type: 'array', items: { type: 'object', properties: {
    id: { type: 'string' }, title: { type: 'string' }, settled_by: { type: 'string', description: 'an invariant, SRS requirement or earlier stakeholder decision that already settles it, or "none"' },
  }, required: ['id', 'title', 'settled_by'] } } },
  required: ['matches', 'new_themes'],
}
const focus = scope && scope.length
  ? ` This is a regression pass: review only these entries and what they touch, reading the rest only as context: ${scope.join('; ')}.`
  : ''

phase('Review')
const reviews = await parallel(Object.keys(LENS).map(lens => () => agent(
  `Read ${BRIEF} and follow it. The repository is ${repo}. LENS = ${LENS[lens]}.${focus} Edit no file. Return the findings through the structured output.`,
  { label: `r${round}:${lens}`, phase: 'Review', schema: FINDINGS },
).then(r => ({ lens, findings: r ? r.findings : null }))))

phase('Match')
const needs = reviews.flatMap(r => (r.findings || []).map((f, i) => ({ lens: r.lens, index: i, ...f })).filter(f => f.severity === 'needs-fix'))
const match = needs.length === 0 ? { matches: [], new_themes: [] } : await agent(
  `Read the finding ledger ${LEDGER}: themes with an id, a title, a status (open, fixed, deferred, accepted) and the texts that close them. Also read ${repo}/research/notes/domain-model-subset-search/core-findings-history.md for what each theme means. Below are this round's needs-fix findings on the domain model. For each one decide whether it is the same defect as a ledger theme (kind "closed" if that theme is fixed, deferred or accepted; "open" if that theme is open) or a new defect (kind "new", theme "NEW-<n>"; group findings that are one defect under one NEW id). A finding is the same defect only if fixing the theme as closed would also answer it; a finding that shows the closing text itself is wrong or incomplete is "new". For each new theme say whether an invariant, an SRS requirement or an earlier stakeholder decision in the plan notes under ${repo}/plan/2610012322_cairn-for-agent-fleets already settles it (cite it) or "none". Edit no file.\n\n${JSON.stringify(needs)}`,
  { label: `r${round}:match`, phase: 'Match', schema: MATCHES, model: 'sonnet' },
)
return { round, reviews, match }
