export const meta = {
  name: 'ga-eval-team',
  description: 'Evaluate assembled domain-model candidates with blind reviewer lenses',
  phases: [{ title: 'Review', detail: 'one blind reviewer per candidate and lens' }],
}
// GA: absolute path of this folder, passed as args.ga
const GA = args.ga
const LENS = {
  consistency: 'consistency: contradictions, undefined terms and gaps between the candidate\'s own statements (its invariants, hub relations and concept entries)',
  security: 'security: does the candidate\'s own concept set uphold its own invariants? Look for paths from untrusted content to the model outside the closed paths, trust or key rules that can be bypassed, data leaving outside declared boundaries, silent failures',
  usefulness: 'usefulness: every included concept must have a use within this candidate, and the core journeys (constraints restored verbatim after compaction, exact recall, a person sees what agents did, untrusted history never instructs) must work with what the candidate defines',
  all: 'all three: consistency (contradictions, undefined terms, gaps), security (does the candidate uphold its own invariants) and usefulness (every included concept has a use; the core journeys work)',
}
const SCHEMA = {
  type: 'object',
  properties: {
    findings: { type: 'array', items: { type: 'object', properties: {
      type: { type: 'string', enum: ['contradiction', 'undefined', 'gap', 'security', 'useless'] },
      quote: { type: 'string' }, problem: { type: 'string' }, fix: { type: 'string' },
      severity: { type: 'string', enum: ['needs-fix', 'minor'] },
    }, required: ['type', 'quote', 'problem', 'fix', 'severity'] } },
  },
  required: ['findings'],
}
const jobs = args.jobs // [{cand, lens, model}]
const res = await parallel(jobs.map(j => () => agent(
  `Read ${GA}/eval-brief.md and follow it. CANDIDATE_DIR = ${GA}/out/${j.cand}. LENS = ${LENS[j.lens]}. Edit no file. Return the findings through the structured output (quote copied exactly from candidate.md, one line, 10-200 chars).`,
  { label: `${j.cand}:${j.lens}`, phase: 'Review', schema: SCHEMA, ...(j.model ? { model: j.model } : {}) },
).then(r => ({ ...j, findings: r ? r.findings : null }))))
return res
