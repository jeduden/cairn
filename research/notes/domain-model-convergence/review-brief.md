# Review brief: one convergence round on the domain model, M1 to M5

You review Cairn's domain model as it stands, every feature still defined.
This brief stays the same from round to round, so rounds can be compared.
Cairn
is a lossless, security-first context layer for long-running Claude agents:
an append-only record of every agent run, pins restored verbatim after
compaction, exact recall on demand, and stored history never a
prompt-injection channel.

Scope, set by the stakeholder on 9 October 2026: the review covers what
ships in milestones M1 to M5. An entry or clause about any of the following
ships later and is dormant: another principal and PRV-10's principal keys,
device certificates and device scopes; service accounts; admission, leave,
kick, bar and mute; roles and appointments beyond a run seat's contributor
role and the owner's device seat's every capability; stamps; trust grants;
delegation; the facilitator and room summaries; the launcher and terminal
takeover; the room view and its surfaces; seals and receipts of chain
heads; peers, sync and blind peers; paired phones; ephemeral and
token-key-only nodes and access tokens; expire acts; endorsement and
directed posts; held requests; verdicts, landings and room trailers;
bundles, import, publishing and bridges. A defect that arises only from
dormant text has severity `later` and names the feature it waits for;
it is never needs-fix. A defect in what ships in M1 to M5 is needs-fix,
even where its fix would also reach dormant text.

Read the model fully: `docs/srs/invariants.md`, `docs/domain-model/index.md`
(the hub) and every concept file in `docs/domain-model/`. Review only this
text. Do not read the rest of `docs/srs/`, the scenarios, the plan notes or
any review history: a rule the model needs must be in the model, or cited
there by requirement id. Requirement ids (REC-01, OWN-11, …) and section
numbers the model cites are references you may not resolve; never report
them.

The model is a domain model: it defines concepts, their relations and the
rules that make them coherent; mechanisms (how something is authenticated,
stored, enforced or computed) live in the requirements it cites. Where a
sentence cites a requirement id for a rule, assume the rule holds as
stated. Report a missing mechanism only when the model states no rule and
cites no requirement for something its own invariants depend on.

Settled by the stakeholder, never a finding: the principal's own tunnel or
`git push` carries data off the machine as the principal's tool, outside
Cairn's components and outside I4; room trailers are always on.

Your lens: LENS.

Report only real defects, each one of:

- contradiction: two statements in the model cannot both hold;
- undefined: a term the model uses as a Cairn concept but never defines, or
  a definition that relies on something the model lacks;
- gap: a rule needed for two implementers to build the same thing is
  missing (say exactly what), or a concept that cannot work as defined;
- security: a path from untrusted content to the model outside the closed
  paths, a trust or key rule that can be bypassed, data leaving outside the
  declared boundaries, a silent failure, or an invariant the model's own
  concepts break;
- useless: a concept nothing can create, reach or act on, or the model
  fails a core journey: an agent gets its constraints back verbatim after
  compaction; it recalls exact history; a person sees what agents did;
  untrusted history never instructs an agent.

Do not report wording style, missing examples or nice-to-haves. Ask: would
this make two careful implementers build different things, or open a hole?
If not, skip it.

Output: one fenced `json` block, an array of objects with `type` (one of
the five above), `file` (the file you quote), `quote` (exact text copied
from it, 10 to 200 characters, from the line at fault, no line breaks
inside), `problem` (one or two sentences), `fix` (the smallest wording
change that fixes it) and `severity` (`needs-fix`, `minor` or `later`).
Then one line: `FINDINGS: <n needs-fix>`. An empty array is a valid and welcome
answer when the model holds.
