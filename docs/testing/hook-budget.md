---
title: "The hook budget row"
summary: >-
  How the NFR-02 scenario holds a hook handler to its hook budget relative to the core's start-up, the limits and their derivation, and where the absolute budget is held.
---
# The hook budget row

How the `@NFR-02` scenario holds a hook handler to its hook budget, and
why it compares times instead of reading the clock. The page belongs to
[testing.md](../testing.md).

## Why the row is relative

The core runs under `strace` in a network namespace (ENG-12). That adds
25 to 30 ms to every run. A hook handler needs 5 to 10 ms, so a 50 ms
budget was overrun on a loaded machine. The core stops a handler at its
own budget and counts `hook_budget_exceeded`. That happened in two runs
of thirteen idle, and in three of six under load.

The row now times 11 pairs of runs under that same confinement. Each
pair is the release core started with `version`, then the hook handler
on its recorded-shape input. The order swaps every round
(`testkit::relative::interleaved`). Whatever slows one run slows the
other.

`within_multiple` then compares the two medians. The hook's median must
be at most a limit times the reference's. Each run must also exit 0 with
an empty answer and no stderr. The slowest hook must stay below its
harness timeout. The start-marker steps are unchanged.

## The limits

A limit is the hook's budget in units of the reference start-up:
`budget / REFERENCE_START_UP_MS`, with the reference at 2 ms.

| Hook              | Budget (§9.1) | Limit | Measured, unconfined | Measured, confined |
| ----------------- | ------------- | ----- | -------------------- | ------------------ |
| SessionStart      | 150 ms        | 75    | 5.8                  | not measured       |
| UserPromptSubmit  | 50 ms         | 25    | 5.1                  | not measured       |
| PermissionRequest | 50 ms         | 25    | not measured         | about 2            |

The 2 ms is `core_start_up`'s median on the development machine that
measured it, rounded up. That was 1.8 ms over 30 samples of
`benchmark run`. The hook handlers took 10.5 ms and 9.1 ms there.

This constant is for the stakeholder to confirm. No committed benchmark
evidence holds a reference time yet. A limit is the SRS budget expressed
on the reference machine. A handler four to five times costlier than
today's still passes. One past its budget on that machine fails.

## Where the absolute budget is held

The confined row cannot also require that no overrun was counted. The
confinement alone can reach the core's own 50 ms stop. The absolute
budget is held on the unconfined release core instead. The benchmark
suite's hook samples (`hook_session_start`, `hook_user_prompt_submit`)
fail when the handler counted `hook_budget_exceeded` in the sample's
home (ENG-15).

## What proves the check

A unit test with timing doubles shows a slowed hook fails the relative
check and a healthy one passes. A drift case (ENG-27) shows the test
fails when the check is loosened.
