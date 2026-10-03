# Round 4: fleet developer

Target: proposal.md at ca209ea. Six round-3 findings resolved, two
partly, one not; one new blocker.

## Round-3 findings

- Resolved: owner acts with only the core (OWN-12, D5, `/pin` in
  OWN-11); rows and columns; OWN-15 and OQ-17; the `Notification`
  budget; holds only while connected; the phone allows once only.
- Partly resolved: answering from the shell works with only the core,
  but needs a code from another surface once `cairn-ui` or `cairn-run`
  runs (N1); the pitch leftover (the reviewer found the sentence in
  plan.md, where it is now fixed).
- Not resolved: `cairn-run -- <harness>` is still a step at every
  launch.

## New findings

N1. Blocking: running the view makes the shell worse; each allow-once
    or pin from the shell needs a trip to another surface, and with ten
    `cairn-run` instances "there" is undefined. If terminal
    confirmation is safe enough alone, it is safe with the view open.
    Draft: a CLI owner act MUST be confirmable at the terminal where it
    was typed, by the same check whether or not `cairn-ui` or
    `cairn-run` runs; a code from another surface MUST be opt-in or
    required only by managed policy.
N2. Important: on the default install an agent can fake a terminal and
    confirm an OWN-11 act itself. Draft: every OWN-11 act confirmed by
    terminal alone MUST raise a non-dismissible Needs you item and a
    Catch up line naming the act, its target and the confirming
    terminal.
N3. Important: ten `cairn-run` instances may take 2.5 GiB and half a
    core idle. Draft: `cairn-run`'s budgets MUST hold in total for ten
    instances, and it MUST add at most 10 ms† p95 to keystroke echo.
N4. Important: M7's exit MUST answer a held request and add a pin from
    the CLI with only the core installed, and again with `cairn-ui`
    running.
N5. Minor: `cairn install` MUST offer, as a shown diff, an opt-in that
    routes every harness launch through `cairn-run`, and an unavailable
    control MUST name the launch path that offers it.
N6. Minor: the pitch MUST carry every replacement §10.11 lists.

Verdict: much closer; N1 trips "the UI is a required step" whenever the
view runs, N2 "cannot be found again", N3 "slows the terminal".
