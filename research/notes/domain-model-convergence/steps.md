# Steps of the second process

The record of each step of the [process](README.md). Model words are
`wc -w docs/domain-model/*.md`: 18,193 when the process began, against a
target of 14,826.

| Step | Model words | Opened | Closed | On the work's own text |
| ---- | ----------- | ------ | ------ | ---------------------- |
| 1    | 18,177      | 0      | 2      | 0                      |

## Step 1: the open questions inside M1–M5

The stakeholder decided on 9 October 2026, and the SRS records each
decision:

- **OQ-43, where a witnessed run's events go while no MCP server holds
  its run-seat key.** The events are held, then ingested. The hook
  handlers append nothing until the server holds the key, leaving an
  ingest marker. A run no MCP server serves reaches the record only
  through `cairn ingest`, and is counted and audited until then
  (REC-19). This closes DM-CD.
- **OQ-43, a lost run-seat key, and a resume with no stable agent id.**
  The interim rules stand for M1, and spike S4 reopens them. DM-D and
  DM-BK stay deferred.
- **OQ-49.** The pin goes to its creating event's room. Where its
  principal has no member seat left there, it goes to the personal
  room, which the confirmation shows (PIN-05). This closes DM-DT.

§5.2 (PRV) moved unchanged to its own file,
`05e-provenance-requirements.md`, so `05-functional-requirements.md`
has room under its token budget for the steps that follow. The ledger's
citations of PRV rows moved with them.

Step 2 sweeps these decisions through the grid's rows for run-seat key
loss, harness resume and ingest.

## Step 2: the lifecycle grid

Six agents read the SRS for 27 M1–M5 lifecycle events, one per row, and
named for each derived artifact the requirement that decides it, or the
gap. The grid itself lands in §8.5. The gaps closed as follows.

The stakeholder decided four questions:

- **Quarantine selectors** stay live for recall, landmark text and
  trusted-only exports. A pin version or landmark recorded after a cut
  quarantine leaves restore only through a widening one. A seat selector
  covers the seat `cairn ingest` starts beside it, never a successor seat
  (SEC-12).
- **Retention** ages events by the node's structural time marks, never by
  a transcript timestamp (REC-15, SEC-22, PRV-01).
- **A copy another node made:** the rooms it brings that the node did not
  hold stay foreign until PRV-10 ships or a verified seal keeps their
  events (ADM-06).
- **A restore fork:** each segment of the copy that differs from the
  node's event at its address is refused (ADM-06, REC-21).

The stakeholder's rule that a recommendation decides settled the rest:

- After a node identity change or a clone, the home's rooms stay the
  principal's and its old seats count as member seats, until PRV-10
  ships (REC-24).
- A purge whose scope holds a pending ingest marker or a run REC-19
  counts is refused until ingest brings them in (ADM-07).
- An act on more than one room goes to the personal room (LANE-01).
- Each subagent's run is its own agent (REC-02).
- A restore keeps the copy's audit log as a read-only archive, and no
  counter derives from it (ADM-06).
- A run's last span closes at its end, recorded as a structural hook
  observation (LMK-01).
- `cairn status` names each setting waiting for acceptance (ADM-11).
- A principal may dismiss a pin candidate by a neutral act (PIN-05).
- A device key or seat key lost while the node identity is unchanged is
  minted anew without counting as a changed identity (REC-24, SEC-10).

Tightenings that follow from the invariants and existing rules landed in
SEC-10, SEC-12, SEC-13, SEC-22, SEC-27, REC-02, REC-13, REC-15, REC-19,
REC-21, REC-24, PIN-05, PIN-10, INJ-05, ADM-04, ADM-06, ADM-07, PRV-01,
PRV-02, PRV-07, PRV-09, LANE-01, LANE-16, LANE-23 and §12.2. The latter
schedules the step-1 decision and SEC-27's lost-key seat in M1. The model
gained the term time mark and the subagent clause; it also dropped
restatements of SEC-12 and ADM-02, so it shrank to 18,147 words.
