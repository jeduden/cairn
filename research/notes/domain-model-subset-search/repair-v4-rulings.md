# Version 4: a minimal core, plus repairs

Folder GA = research/notes/domain-model-subset-search (the caller gives its absolute path).
GA/annotated/ is version 3 (snapshot GA/annotated-v3/). Follow
GA/repair-brief.md and GA/annotate-brief.md. Findings: GA/evals/g3-*.findings.json.
Settled by the stakeholder, keep as is: the principal's own tunnel and `git
push` are the principal's tools outside I4; room trailers are always on.

The strong reviews show findings concentrating in core concepts that a single
node never needs. Make the core minimal by turning them into optional genes.

## 1. New genes (edit GA/features.json and the spans)

Add two features to GA/features.json (same fields as the others; entries may
be empty lists when the concept lives in clauses inside other items):

- `node-clone`: node clone, node identity change, a seat's predecessors,
  lineage acceptance, the carried-over personal room, every "however far
  back" and predecessor clause, the predecessor exceptions in I2 and I8
  (E2, E3, E5 in GA/invariant-edits.md: move those clauses into node-clone
  spans so the core invariants read as before them), and the backup
  restore's predecessor rule (nest in store-protection). requires: [].
  value 2, risk 4.
- `key-revocation`: revoking a run seat's or device seat's key, the new seat
  a revocation mints, revoked seats' pins and versions not qualifying,
  configuration pins re-recorded after a revocation, the revocation clauses
  in the Cut and Widening lists. requires: []. value 2, risk 3. Keep key
  rotation and the run seat replaced when its key is lost with its MCP
  server in core.

Wrap every clause, sentence and item of these concepts in its new span, in
every annotated file, nesting where another feature also applies. With both
genes absent, core must read as one coherent model of one node that is never
cloned, whose seat keys are never revoked. Update `requires` of existing
features that depend on them (for example access-tokens or principal-keys if
they rely on revocation) only where a dependency is real.

## 2. Core repairs from generation 3

R1 Principal surface naming: the room view's surfaces are where things are
   shown; the client where principal acts are taken is a principal surface.
   Use one meaning: rename Room view's sentence so "surface" stays the room
   view's, and a principal act is marked with the principal surface (client)
   it was taken at.
R2 Principal surface test: a process the harness starts (a hook command, an
   MCP server) is never a principal surface either; only a CLI or TUI at a
   terminal that neither a run's tool process nor the harness started.
R3 Pins by room act: a qualifying pin's creating event and restoring version
   are principal acts; a device seat's room act that pins or edits a pin of a
   type that restores is refused and counted.
R4 Configuration pins change only through configuration acceptance; any
   other act on one is refused and counted.
R5 Cut-only configuration changes: a change to the principal's configuration
   takes effect only once accepted, cut-only changes included, since the file
   can change outside any principal act; repository configuration stays
   cut-only without acceptance.
R6 Capture gap marking: turning capture off (or pausing) and on are recorded
   acts in the device seat's writer, and a run's capture gap is the stretch
   between them; the run's writer records a structural gap marker when
   capture resumes. Drop "pause" if nothing distinguishes it from off (then
   fix I1's E1 clause to "...while its principal had capture off, each such
   stretch shown as a capture gap" and log the change).
R7 Conflict order: "the more restrictive act wins (an unpin over an edit over
   a pin; a cut principal act over a widening one), then the lower
   commitment."
R8 Run seat's own pins after a key loss: its own pins are those it or the run
   seats it replaced in the same run wrote.
R9 The other g3 needs-fix findings on core and on features, smallest coherent
   change: admission and bars before PRV-10 (the certifying device key stands
   for the principal key), who may record an expire act ending a bar, mute or
   handover offer (only a node of the principal that set it or of the owner),
   and the rest. Skip wrong ones; never weaken an invariant.

When done: `python3 GA/check_annotation.py --no-orig` prints OK; assemble
the core genome `{"name":"v4-core","features":[]}` into GA/out/v4-core and
read it end to end: it must be a coherent single-node model. Report the new
genes' span counts, per ruling what you changed, and invariant text changes.
