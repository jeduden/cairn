# Spike S5 report: Lua or SQL for the kernel

Spike S5 asks which language the hermetic compute kernel (CMP-01 to
CMP-07) speaks by default. ADR-04 had chosen Starlark and named its
risk, a dialect Claude writes less well than the languages it knows.
The stakeholder narrowed the candidates to Lua and SQL before the
spike began.

## Decision

SQL, on SQLite over a read-only connection to the record, by the
stakeholder's decision of 13 October 2026. ADR-04 records it and OQ-10
is answered.

Claude wrote correct Lua and correct SQL equally often: every one of
the 180 solutions passed on its first run. The two languages part on
everything else:

- Claude's SQL was half the size of its Lua, so a question costs about
  half the context.
- SQLite is in the core already (ADR-07). Lua would add `mlua` and the
  Lua interpreter's C source to the core's worker.
- SQLite's authorizer, progress handler and heap limit gave the
  allow-list, the read-only connection, the step budget and the memory
  limit (CMP-03 to CMP-05) without new code around the interpreter.

## Setup

- [gen.py](spike/fixture/gen.py) generates the fixture record from a
  fixed seed: 3,488 events in seven writers, six runs (`A1` to `A6`)
  and the operator's pin events (`D1`). Each event has the same twelve
  fields: writer, seq, run, ts, provenance, event kind, tool, call id,
  status, tokens, text and args. The record holds parallel tool calls
  whose results come back out of order, retries of failed calls, tool
  calls with no result, compactions, recall events and pins added and
  removed. `python3 fixture/gen.py > fixture/record.json` rebuilds it.
- [tasks.json](spike/tasks/tasks.json) holds 30 tasks in six groups of
  five or six: counting, grouping, joining a call to its result, time,
  text and JSON, and stateful tasks (streaks, sessions, pin replay,
  turns, retries, running sums). [reference.py](spike/tasks/reference.py)
  computes [answers.json](spike/tasks/answers.json) in plain Python.
- [The runner](spike/runner/src/main.rs) executes one solution under
  the kernel's limits and prints its answer as JSON. Lua runs in Lua
  5.4 through `mlua` 0.12.2, with only the allow-listed globals: no
  `io`, `os`, `require`, `load` or `debug`, no `math.random`. SQL runs
  on SQLite 3.53.2 through `rusqlite` 0.40.2, built with
  `LIBSQLITE3_FLAGS=-DSQLITE_ENABLE_MATH_FUNCTIONS`. Its authorizer
  allows reads and temporary tables and views, and denies writes to the
  record, `ATTACH`, `PRAGMA`, `random()` and `randomblob()`. Both get a
  step budget, a 10 s wall clock and a 512 MiB memory limit, and the
  same helpers: JSON, a regular-expression test and capture.
- The solvers were 18 Claude agents, each fresh: two languages, three
  repetitions, three batches of ten tasks. Each read only its brief,
  [lua.md](spike/briefs/lua.md) or [sql.md](spike/briefs/sql.md), and
  the task list, then wrote its ten solutions without running any code.
  None saw the record's data or the reference answers.
- [score.py](spike/score.py) runs each solution, compares its answer
  with the reference (floats within 1e-6), and writes
  [results.json](spike/results.json).

## Results

| Measure                        | Lua           | SQL           |
| ------------------------------ | ------------- | ------------- |
| Correct on the first run       | 90 of 90      | 90 of 90      |
| 95% Wilson interval            | 95.9% to 100% | 95.9% to 100% |
| Runtime errors                 | 0             | 0             |
| Total size of the 90 solutions | 43,823 chars  | 20,505 chars  |
| Median size of a solution      | 480 chars     | 190 chars     |

Every task passed in all three repetitions in both languages, so the
paired comparison has no discordant pair. No solution failed with an
error, so the repair round the plan foresaw had nothing to repair.

Lua's solutions were larger than SQL's in every group:

| Group    | Lua ÷ SQL, characters |
| -------- | --------------------- |
| Counting | 2.22                  |
| Grouping | 2.60                  |
| Joining  | 2.52                  |
| Time     | 2.58                  |
| Text     | 2.01                  |
| Stateful | 1.63                  |

The stateful tasks were where SQL was expected to struggle. It solved
them all, every solution with window functions (`LAG`, running sums,
`ROW_NUMBER`) and none with a recursive query; there its lead in size
shrinks.

## Checks on the harness

- The scorer was fed five wrong solutions, an off-by-one count, a
  missing `ORDER BY`, a wrong list order and an unknown column among
  them; it rejected each.
- The runner refused, in Lua, `os.time()`, `load()` and an endless loop
  (step budget), and in SQL a `DELETE` on the record, `random()` and an
  endless recursive query (step budget).

## Limits

- One model wrote every solution. A tie at 100% says nothing about a
  smaller model, where a gap would show first. The stakeholder decided
  without that measurement.
- The tasks read one fixed record through a flat view or array. The
  kernel M4 builds reads through recall functions (CMP-03), whose shape
  the briefs did not test.
- The record is 3,488 events; neither language's speed at 10M events
  was measured.
