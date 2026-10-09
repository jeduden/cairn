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
