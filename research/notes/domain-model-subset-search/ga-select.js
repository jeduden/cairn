export const meta = {
  name: 'ga-select',
  description: 'One selection-only GA generation over the unchanged domain model: breed, review, record, commit',
  phases: [
    { title: 'Breed', detail: 'Thompson-sampled GA picks k genomes; assemble them' },
    { title: 'Review', detail: 'one blind probe review per candidate' },
    { title: 'Record', detail: 'attribute findings to features; update the ledger' },
    { title: 'Commit', detail: 'copy the genomes, findings and ledger back; commit; push' },
  ],
}
// args: { ga, work, repo, branch, gen, k, trailer }
//   ga: absolute path of this folder; work: a scratch path for this run's copy.
// The text never changes: annotated/ is the original model (version 0) with
// feature spans, so only genomes are bred and reviewed.
const { ga, work, repo, branch, gen, k, trailer } = args
const tag = `v0-ga${gen}`
const env = `CAIRN_REPO=${repo}`

const NAMES = { type: 'object', properties: { cands: { type: 'array', items: { type: 'string' } }, surrogate: { type: 'string' } }, required: ['cands', 'surrogate'] }
const FINDINGS = {
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
const TEXT = { type: 'object', properties: { summary: { type: 'string' } }, required: ['summary'] }
const LENS = 'all three: consistency (contradictions, undefined terms, gaps), security (does the candidate uphold its own invariants) and usefulness (every included concept has a use; the core journeys work)'

phase('Breed')
const bred = await agent(
  `Run these shell commands exactly and report. Do not edit any file by hand.
rm -rf ${work} && cp -r ${ga} ${work} && cd ${work}
python3 ga_round.py ${tag} ${k} --only v0 --seed ${gen}
then for each genomes/${tag}-*.json written: python3 assemble.py genomes/<name>.json out/<name>
Return the candidate names (e.g. ${tag}-1) and the ga_round.py output as surrogate.`,
  { label: 'breed', phase: 'Breed', schema: NAMES, model: 'haiku' },
)
log(`bred ${bred.cands.join(', ')}`)

phase('Review')
const reviews = await parallel(bred.cands.map(c => () => agent(
  `Read ${ga}/eval-brief.md and follow it. CANDIDATE_DIR = ${work}/out/${c}. LENS = ${LENS}. Edit no file but one: as your last step, write your findings as JSON {"findings": [...]} (the same objects you return) to ${work}/evals/${c}.probe.json. Return the findings through the structured output too (quote copied exactly from candidate.md, one line, 10-200 chars).`,
  { label: `review:${c}`, phase: 'Review', schema: FINDINGS, model: 'sonnet' },
).then(r => ({ cand: c, lens: 'all', model: 'sonnet', findings: r ? r.findings : null }))))

phase('Record')
const rec = await agent(
  `Run: cd ${work} && python3 collect_probes.py ${tag} && python3 ga.py rank | head -15. Return the commands' output as summary. Expected candidates: ${bred.cands.join(', ')}; if one has no ${work}/evals/<cand>.probe.json, say so.`,
  { label: 'record', phase: 'Record', schema: TEXT, model: 'haiku' },
)
log(rec.summary)

phase('Commit')
const commit = await agent(
  `Run, stopping at the first failure and reporting it:
cp ${work}/genomes/${tag}-*.json ${ga}/genomes/ && cp ${work}/evals/${tag}* ${ga}/evals/ && cp ${work}/ledger.json ${ga}/
cd ${ga} && ${env} python3 check_annotation.py | tail -1   (must print OK: the text is unchanged)
cd ${repo} && git add ${ga}, then git commit -F <a message file> whose message is the block between the two ---- lines, then git push -u origin ${branch}, retrying up to 4 times with 2, 4, 8, 16 s waits on network errors.
----
research: selection GA generation ${gen}

${rec.summary.split('\n').slice(0, 12).join('\n')}

${trailer}
----
Return a summary with the commit sha.`,
  { label: 'commit', phase: 'Commit', schema: TEXT, model: 'haiku' },
)
return { tag, cands: bred.cands, surrogate: bred.surrogate, record: rec.summary, commit: commit.summary,
  needsFix: reviews.map(r => ({ cand: r.cand, nf: r.findings ? r.findings.filter(f => f.severity === 'needs-fix').length : null })) }
