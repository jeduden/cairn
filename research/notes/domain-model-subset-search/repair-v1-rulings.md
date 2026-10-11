# Version 1 repair: rulings for the recurring core findings

Input: GA/evals/g0-repair-input.json (104 findings from generation 0, 83 of
them on core text, many repeated across candidates and lenses). Follow
GA/repair-brief.md. Apply these rulings to the recurring core defects, so the
core is one coherent single-principal, single-node model whichever features
are added. Text you move out of core goes into the named feature's span;
nest spans where a clause needs two features.

R1 Principal surface (core): the CLI or TUI at a terminal that no run's tool
   process started. A CLI or TUI call from inside a run is the agent's own
   tool call: never a principal act and never a device seat's room act, and
   its reads are recall, enveloped, whatever its output.
R2 Device seat (core): it takes room acts only at a principal surface; an
   agent's tool call acts only through its run seat.
R3 Hook (core): events count as recorded by the hook handlers only for hook
   input the harness itself delivered; a hook call from inside a run's tool
   process is refused and counted (I6).
R4 `user` events (core): only the prompt a user turn's hook input reports
   becomes a `user` event; a user-role transcript line no hook input confirms
   is `harness_text`; a delegated task is recorded with its parent's
   `assistant` provenance, never `user`.
R5 Configuration (core): a configuration change takes effect only once the
   widening act accepting it records its digest, except a change that only
   cuts; configuration pins are added, edited or unpinned by that acceptance.
   Repository configuration declares no pins and may make only changes a cut
   act could make (never removing a pin from a restore block).
R6 Restore rule (core): a qualifying device-seat pin is one written from this
   node's own device seat, or from the device seat a newly minted device seat
   here names as its predecessor (node identity change, node clone, backup
   restore). The clause "a device seat a device key certified restores to its
   author's principal's agents" and every other cross-node trust clause go to
   principal-keys (nest trust-grants etc. as now).
R7 Origin (core): `witnessed` and `ingested`; `peer`, `bundle` and "received"
   go to segment-exchange (and export-bundles / bundle-import). A node clone's
   carried-over writers keep their recorded origin.
R8 Boundaries (core): only B0. B1 clauses go to launcher or browser-room-view
   (nest as needed), B2 to peer-sync or segment-exchange, B3 to git-carrier or
   bridges; managed policy's power to disable a boundary goes with that
   boundary's features. I4's text follows the same split.
R9 Device scope goes to principal-keys (and paired-phone where it limits a
   phone). Fixed template and principal-typed text go to the features whose
   paths use them (held-requests, launcher, endorsement, delegation-grants,
   trust-grants), and I2's mention of the OWN-04 and OWN-07 templates with them.
R10 Rooms (core): only the personal room. Creating other rooms, role
   assignment, the owner's capabilities in other rooms, the `rooms` recall
   scope and the foreign room go to work-rooms (foreign room also needs
   segment-exchange or bundle-import: nest). Core relations that name them
   get those spans.
R11 Room view (core): the TUI and the CLI show each run with its run status,
   its events by address with their trust marks, and the pins that restore,
   so a person sees what agents did with core alone.
R12 Capture off: keep "turn capture off" in core. In the candidate's I1, add
   to the exceptions "and the events of a run while its principal had capture
   off, each shown as a capture gap". This is an invariant rewording: list it
   in GA/invariant-edits.md (before, after, why).
R13 Run seat key (core): until its run's MCP server holds it, the core mints
   and keeps a run seat's key like a device seat's and seals its writer; a key
   lost with its MCP server starts a new seat that names the old one.
R14 Seat key revocation: revoking a run seat's key is a cut act; revoking a
   device seat's key is widening (its pins stop qualifying); after a
   revocation the seat's later acts and seals count for nothing.
R15 Quota: reaching a quota never drops, truncates or deletes an event; it is
   counted and shown (I6).
R16 Pin candidate confirmation (pin-candidates): only after the principal is
   shown its exact text, pin type and priority.

Other findings: repair those on core with the smallest coherent change;
repair feature findings only when clearly right and small, inside that
feature's span; skip findings that a ruling above already settles, findings
that are wrong, and findings about a missing mechanism the candidate already
cites a requirement for. Never weaken an invariant's protection.

Every invariant text change (beyond spans) goes in GA/invariant-edits.md.

When done: `python3 GA/check_annotation.py --no-orig` prints OK; assemble
g0-core, g0-all and g0-lowrisk into GA/out/v1-core, GA/out/v1-all,
GA/out/v1-lowrisk (same genomes) and read the core text: it must read as one
coherent single-node model. Report per ruling what you changed, then the
other findings repaired/skipped with reasons.
