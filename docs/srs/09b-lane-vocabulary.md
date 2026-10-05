---
title: "9.7 Room vocabulary"
summary: >-
  The one vocabulary every surface uses (VIEW-14): harness status and
  freshness, room status, evidence and proof classes, the Needs you
  order, integrity seals, trust marks and the keymap. Part of §9.
---
# 9.7 Room vocabulary

One vocabulary for every surface (VIEW-14): status words, marks, queue
order and keys. The reasons behind each choice are in plan 2610012322's
proposal, §8.

## 9.7.1 Harness status and freshness

| Status        | Meaning                                                | Sub-labels                                    |
| ------------- | ------------------------------------------------------ | --------------------------------------------- |
| **Starting**  | launched, no first event yet                           | —                                             |
| **Working**   | a turn is running                                      | —                                             |
| **Needs you** | blocked on a person                                    | `approval`, `question`, `hand-off`            |
| **Idle**      | turn ended, nothing pending                            | `claims done` when the agent says so (a mark) |
| **Paused**    | held by the owner at a safe point, acknowledged        | —                                             |
| **Blocked**   | cannot proceed without something other than a decision | `rate limit`, `error`                         |
| **Ended**     | session closed, acknowledged                           | `by agent`, `by owner`, `crashed`             |

| Freshness mark | Meaning                                                                          |
| -------------- | -------------------------------------------------------------------------------- |
| (none)         | live: events current on this node                                                |
| `behind`       | on a writer whose log has not reached this node lately: "as of 14:02 on desktop" |
| `unrecorded`   | the harness runs but Cairn sees no hook events since a time                      |
| `imported`     | ingested from a transcript Cairn did not watch (REC-22)                          |
| `stuck?`       | viewer-side only: Working with no event past a threshold                         |

## 9.7.2 Room status

First match wins, in this order: **Abandoned** and **Landed** (closed
rooms only), **Needs you**, **Failing** (a required check failed on
the head, evidence `own run` or stronger), **Blocked**, **Running**,
**Ready for review** (only after the owner act of OWN-21), **Quiet**
(never while a harness is `unrecorded` or `behind`, VIEW-04). Review
states (Draft, In review, Changes requested, Approved, Queued,
Landing) live on the gate pill (P2), not in the room status. Approval
states: Current, Stale, Carried, Revoked, Void. Check states: Pending,
Running, Passed, Failed, Stale, Unbound, Absent.

## 9.7.3 Evidence and proof classes

| Evidence class | Glyph | Satisfies a required check                    |
| -------------- | ----- | --------------------------------------------- |
| `claim`        | `○`   | never                                         |
| `own run`      | `◐`   | only where the gate policy says so explicitly |
| `witness run`  | `◑`   | yes, unless the policy excludes it            |
| `CI attested`  | `●`   | yes                                           |

| Proof class              | Counts as proven |
| ------------------------ | ---------------- |
| `same commit`            | yes              |
| `same patch`             | yes              |
| `same tree`              | yes              |
| `contains approved diff` | yes              |
| `likely`                 | no               |
| `asserted`               | no               |
| `not proven` + reason    | no               |

## 9.7.4 Needs you order

Queue classes, named Q1–Q4 so they never read as priorities P0–P2:
**Q1 blocking now** (permission, question, hand-off), **Q2 blocking
the room** (failed required check, crash, rate limit with no resume,
refused segment, overlap), **Q3 waiting on you** (review requested,
outcome ready to judge, co-author request, parked request, quota
crossed), **Q4 for your
record** (never alerts). Order: class; inside Q1, the number of agents
blocked on the same answer, then causal order of raising; inside Q2
and Q3, rooms in the focus set first, then causal order. Causal order,
not wall-clock age, because age differs between nodes and reads a
clock (I10); it is oldest first wherever clocks agree. Review requests
join the same queue.

## 9.7.5 Integrity seals

`verified` ◆ (chains and seals check), `unsigned` (hash-chained,
before REC-18 or past the newest seal), `incomplete` ◇ (a writer's
events are missing here), `unverified` (an event after a break),
`broken` ✕ (a check failed), `refused` (a segment was refused),
`equivocated` (PEER-10). The UI never says "secure".

## 9.7.6 Trust marks

No mark on this node's own trusted events and on certified owner acts
(the latter show a device glyph). `○` plus petname on anything
untrusted from another person, agent, node or bundle; a key with no
petname shows its fingerprint. `⚑` flagged (PRV-07). `▒` quarantined.
`▬` removed. `imported`. `new key`. "from a removed participant",
"from a revoked node". Timeline rails: solid for the agent's
principal, hollow for other people, dotted for agents and the forge.
Sandbox events are untrusted on every other node (PRV-02). Unsandboxed
agents are one line on Health, not a mark on every tile.

## 9.7.7 Keymap

One map, no key bound to two actions. Typing in a composer or text
field sends text, not shortcuts.

| Key                   | Action                                   | Key             | Action                              |
| --------------------- | ---------------------------------------- | --------------- | ----------------------------------- |
| `?`                   | all shortcuts                            | `a` / `A`       | allow once / allow for session      |
| `/`                   | search and filter                        | `d`             | deny                                |
| `⌘K` / `Ctrl+K`       | command palette                          | `r`             | reply                               |
| `⌘G` / `Ctrl+G`       | go to address                            | `e` / `E`       | endorse / edit and send             |
| `g i`                 | Needs you                                | `z`             | snooze                              |
| `g f`                 | Fleet                                    | `x`             | dismiss, Q3, Q4 and overlap items   |
| `g c`                 | Catch up                                 | `y`             | copy address                        |
| `g q` / `g v`         | quarantine list / verify panel           | `o`             | open the room at this item          |
| `g h` / `g p`         | Health / Peers                           | `n` / `N`       | next / previous item that needs you |
| `1` / `2` / `3`       | room tabs: Timeline / Review / Replay    | `m`             | point an agent at this address      |
| `j` / `k`             | next / previous row, item or file        | `q`             | quarantine selection                |
| `Enter`               | open                                     | `⌘S` / `Ctrl+S` | save search                         |
| `Space`               | preview or peek                          | `+` / `-`       | more / less context around a hit    |
| `Esc`                 | close sheet                              | `w` / `b`       | why panel / blame gutter            |
| `I`                   | interrupt (harness pane)                 | `W`             | witness run                         |
| `p`                   | pause / resume                           | `c` / `C`       | comment / request-changes sheet     |
| `S`                   | stop (opens the stop sheet)              | `Y`             | approve sheet                       |
| `f`                   | fork room from here                      | `l`             | land sheet                          |
| `t`                   | take over / release (run component only) | `V` / `s`       | compare versions / since my verdict |
| `h`                   | hand back                                | `(` / `)`       | previous / next comment             |
| `Q`                   | away mode                                | `<` / `>`       | down / up the stack                 |
| `F`                   | follow a person                          | `Ctrl+T`        | add the room to the focus set       |
| `Shift+Space`         | replay play / pause                      | `←` / `→`       | replay previous / next event        |
| `Shift+←` / `Shift+→` | replay previous / next span              | `,` / `.`       | replay previous / next turn         |
| `[` / `]`             | replay previous / next edit              | `{` / `}`       | replay previous / next error        |
| `;` / `:`             | replay next / previous request           | `v`             | replay context lens                 |
| `L`                   | replay jump to live                      | `Home` / `End`  | replay start / end                  |

Interrupt moved from `Esc Esc` to `I`, so no key repeats into a
second act. Composer: `Enter` steer, `Ctrl+Enter` queue for the next turn,
`Shift+Enter` redirect. Clashes resolved: `x` (dismiss, expand, mark
seen) is dismiss only, with context on `+`/`-`; `e` (snooze, endorse,
next error) is endorse, snooze moves to `z`, errors to `{`/`}`; `]`/`[`
(next room, next edit, stack) are edits, rooms move to `n`/`N`, the
stack to `<`/`>`; `a` (allow, approve, next approval) is allow,
approve moves to `Y`, replay requests to `;`/`:`; `r` (reply, request
changes, replay) is reply, request changes moves to `C`, replay to tab
`3`; `c` (copy, comment) is comment, copy moves to `y`; `p` (pause,
previous comment, point agent) is pause; `.` (next turn, follow) is
next turn, follow moves to `F`; `Space` (select, peek, play) is peek,
play moves to `Shift+Space`; `Shift+A` (adopt) is gone with Adopt.
