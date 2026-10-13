# Brief: answer questions about a record in SQL

You are writing code for a compute kernel. Each solution you write is
executed once, without you seeing the output, and its answer is
compared with a reference. Write each solution so that it is right the
first time.

## The record

The record is a list of events from six agent runs. Each run has its
own writer (`A1` to `A6`); a seventh writer, `D1`, holds the
operator's pin events. Every event has the same twelve columns. A column
that does not apply to an event is NULL.

| Field        | Type    | Meaning                                                                                                        |
| ------------ | ------- | -------------------------------------------------------------------------------------------------------------- |
| `writer`     | string  | The writer the event belongs to: `A1` to `A6`, `D1`                                                            |
| `seq`        | integer | The event's position in its writer: 1, 2, 3, … with no gaps                                                    |
| `run`        | string  | The run: `r1` to `r6` (writer `A1` holds run `r1`, and so on); NULL on `D1`'s events                           |
| `ts`         | integer | When the event happened, in Unix seconds, UTC; never decreases within a writer                                 |
| `provenance` | string  | What produced the content: `user`, `assistant`, `tool_result:<tool>`, `harness_meta`, `structural`, `operator` |
| `kind`       | string  | `message`, `tool_call`, `tool_result`, `compaction`, `recall` or `pin`                                         |
| `tool`       | string  | The tool's name, on tool calls and tool results: `Bash`, `Read`, `Edit`, `Write`, `Grep`, `Glob`, `WebFetch`   |
| `call_id`    | string  | On a tool call and on its tool result, the same id; links the two                                              |
| `status`     | string  | On tool results only: `ok` or `error`                                                                          |
| `tokens`     | integer | The event's size in model tokens; present on every event                                                       |
| `text`       | string  | The event's text; present on every event                                                                       |
| `args`       | string  | JSON text, on four kinds of event, described below                                                             |

The `args` field holds:

- on a tool call, the tool's input: `{"command": …}` for `Bash`,
  `{"file_path": …}` for `Read`, `Edit` and `Write`, `{"pattern": …,
  "path": …}` for `Grep`, `{"pattern": …}` for `Glob`, `{"url": …}`
  for `WebFetch`
- on a `compaction` event: `{"pre_tokens": <integer>, "trigger":
  "auto" | "manual"}`
- on a `recall` event: `{"query": <string>, "tool": "event_search"}`
- on a `pin` event: `{"action": "add" | "remove", "pin_id": <string>}`

Terms the questions use:

- A **tool call** is an event of kind `tool_call`; a **tool result**
  is an event of kind `tool_result`. A tool result answers the tool
  call with the same `call_id`. A tool call may have no result.
- A **user message** is an event of kind `message` with provenance
  `user`; an **assistant message** has kind `message` and provenance
  `assistant`.
- A run's events are the events whose `run` field names it.

## The language

SQL, run by SQLite 3.53 in a sandbox. The record is the view `events`,
one row per event, with the columns above. Integers are stored as
`INTEGER`, the rest as `TEXT`. SQLite's built-in functions are there,
among them the JSON functions (`json_extract`, `->`, `->>`,
`json_each`), the math functions (`floor`, `sqrt`, `ln`, …), the date
and time functions, window functions and common table expressions,
recursive ones included. The host adds two:

- `regexp(pattern, s)`: 1 if the regular expression matches anywhere
  in `s`, else 0, so `s REGEXP pattern` works
- `regexp_capture(s, pattern, n)`: the text of capture group `n` of
  the first match, or NULL

Patterns are regular expressions in Rust `regex` syntax (as in
Python's `re`: `\d`, `(…)`, `[^.]`, `$`).

A solution may hold several statements separated by `;`. It may create
`TEMP` tables and views. It may not change `events`, attach a database,
run a `PRAGMA`, or call `random()` or `randomblob()`.

## The answer

The rows of the last statement that returns rows are the answer. Each
question names a shape:

- **scalar**: one row with one column
- **list**: one column, one row per value, in the order the question
  asks
- **rows**: one row per answer row, with the columns in the order the
  question names them

Rows come back in no particular order unless the statement has an
`ORDER BY`. Integers may come back as integers or as floats with the
same value; floats are compared with a small tolerance. Do not round
unless the question says so.

## Your task

For each task you are given, write one file holding only SQL, at the
path you are given. Do not run any code, do not open any other file in
the repository, and do not look at the record's data: work from this
brief alone.
