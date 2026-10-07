Feature: Room view (VIEW)

  Scenarios for SRS §5.12, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @VIEW-01 @P1 @I1 @I6 @pending
  Scenario: Fleet shows every run of the principal's rooms grouped by room
    Given an isolated Cairn home
    And two rooms of "alice" with one live and one recorded run each
    When the person opens Fleet in the room view and selects one run
    Then every run appears grouped under its room
    And the selected run shows its user turns, tool calls, held requests, output and each edit as a diff
    And each check shows its result and evidence class, and each event shows its author
    And every other run appears as a tile that opens full size

  @VIEW-02 @P1 @I9 @pending
  Scenario: the room view shows an active run's lines within 2 s without slowing hook handlers
    Given an isolated Cairn home
    And ten local runs in one room whose harness writes their transcripts
    When the person watches one active run in the room view
    Then each transcript line appears within 2 s of the harness writing it
    And no hook handler exceeds its NFR-01 budget while the view reads

  @VIEW-03 @P1 @I9 @I10 @pending
  Scenario: every room-view surface is an optional, read-only client of the core
    Given an isolated Cairn home
    And a room with recorded runs
    When the room-view component serves the room view and is then stopped
    Then the view reads the record only through the core's read path and never writes to it
    And the view keeps no state the record cannot rebuild beyond conveniences for the person viewing
    And every capability the view offers also exists in the CLI or MCP
    And hook handlers, ingestion and recall keep working with the view stopped

  @VIEW-04 @P1 @I6 @I10 @pending
  Scenario: statuses come from structural fields only and unrecorded runs surface
    Given an isolated Cairn home
    And a room with one run whose transcript is longer than its ingested position
    And a run the launcher started with no hook observation, sitting only in its personal room
    When the person opens Fleet and runs "cairn room list"
    Then every run shows exactly one status from the closed set of §9.7.1 with its freshness mark
    And the room shows one room status and the worst freshness mark of its runs, never Quiet
    And the launched run appears under its personal room on Fleet and in "cairn room list"
    And text an agent wrote changes no status, and time-relative marks are computed in the room view from an explicit starting point

  @VIEW-05 @P1 @I6 @I10 @pending
  Scenario: the Needs you queue is deterministic and clears everywhere once answered
    Given an isolated Cairn home
    And a room with open items of classes Q1 and Q2 waiting on the person, one of them an overlap
    When the person answers one item in the CLI
    Then every surface lists the open items in the order of §9.7.4, the same for the same record
    And no Q1 or Q2 item can be dismissed, except the overlap item, which clears on acknowledgement
    And the answered item clears on every other surface as soon as the answer's event arrives there

  @VIEW-06 @P1 @I2 @I6 @pending
  Scenario: written text is shown apart, marked untrusted and never sets a status
    Given an isolated Cairn home
    And a run whose agent wrote "all tests pass" in its output
    When the person opens that run in the room view
    Then the text is shown apart from derived lines and marked untrusted
    And it is marked as a "claim"
    And no status, evidence class or proof class changes because of it

  @VIEW-07 @P1 @I2 @pending
  Scenario: items from outside the trusted sources carry a trust mark and the petname the person viewing chose
    Given an isolated Cairn home
    And synced posts by a key the person gave a petname, a key with none, and a new key using a known name
    And one post with zero-width, bidirectional and tag characters and an HTML comment
    And a "user" turn this node ingested from a transcript its hook handlers did not watch
    When the person opens the room view
    Then each post carries the trust mark of §9.7.6 and the petname the person chose for its author's key, never the name the peer sent
    And the key with no petname is shown by its fingerprint and the new key is marked "new key"
    And the ingested "user" turn carries the trust mark of §9.7.6 too, since only the "user" turns this node witnessed while its deployment mode is "interactive" are trusted
    And the invisible characters and the HTML comment render as visible placeholders with a count

  @VIEW-08 @P1 @I6 @I10 @pending
  Scenario: Catch up is derived deterministically from the record and a starting point
    Given an isolated Cairn home
    And rooms with capture gaps, an unsynced writer, a tombstone, a voided branch link and finished runs since a starting point
    And a head receipt of chain heads outside CAIRN_HOME
    When the person opens Catch up at that starting point
    Then it shows the starting point it used and where it came from, and links every line to its events
    And it lists capture gaps, uningested transcripts, risen failure counters and unsynced writers apart from rooms with no activity
    And it checks each writer's chain head against the named head receipt and shows the result per writer
    And its lines run integrity and capture gaps, Needs you, failures, then finished runs, with the newest seq it covers per writer and no model-written line

  @VIEW-09 @P1 @I6 @I8 @pending
  Scenario Outline: the person's search uses the scope they select and states its coverage
    Given an isolated Cairn home
    And "alice", with two rooms, an unsynced writer and a quarantined match
    When "alice" searches "fix (login" with scope "<scope>"
    Then the result list shows the scope "<scope>" it used
    And the typed text is matched as literal terms
    And recall events are left out of ranking
    And below the results it states what it covered and left out, naming the unsynced writer and the quarantined match
    And a foreign room this node holds is searched only when "alice" names it in the search

    Examples:
      | scope |
      | run   |
      | room  |
      | rooms |

  @VIEW-10 @P1 @I6 @I10 @pending
  Scenario: every room shows its integrity status and the view writes a head receipt outside the home
    Given an isolated Cairn home
    And a room whose writer's chain breaks at one event
    When the person opens the room's Room page and, from its verify panel, writes a head receipt to a path outside CAIRN_HOME
    Then the room shows its integrity status at all times, one of §9.7.5's seven values, here "broken"
    And every later event of that writer is marked "unverified" wherever it is shown, including in recall results
    And the head receipt is shown as a short code carrying at least 80 bits of the heads' digest
    And the view states that verification proves the sealed record unchanged up to its newest seal, not the unsigned tail and not its content true
    When the person turns an away policy on
    Then the view offers to write a head receipt

  @VIEW-11 @P1 @I1 @I5 @I6 @pending
  Scenario: gaps, quarantines and tombstones stay in place and forensic views are recorded
    Given an isolated Cairn home
    And a room with a missing segment, a quarantined range and a tombstone
    When the person opens the Timeline tab of its Room page, then the quarantine list from it, and opens from that list the forensic view of the quarantined content
    Then each gap is shown in place, never closed up
    And the quarantined content is shown only after that explicit forensic view
    And the forensic view is recorded as a neutral principal act, an "operator" event
    And the tombstone reads "removed from this node", never "erased"

  @VIEW-12 @P1 @I1 @I10 @pending
  Scenario: replay reconstructs any event from the record alone
    Given an isolated Cairn home
    And a room with two writers, edits, tool calls and a recall
    When the person replays the room to one event on the Replay tab of its Room page
    Then the conversation, worktree and results at that event come from the record alone and nothing is re-executed
    And the worktree states its fidelity as "exact", "approximate" or "unavailable", and why
    And the context lens marks itself a reconstruction and shows recalled content inside its envelope
    And events of the two writers appear in the same causal order on every replay

  @VIEW-13 @P2 @I10 @pending
  Scenario: comparing two branches shows exposure beside results and picks no winner
    Given an isolated Cairn home
    And a room naming two branches for its intent, where the run on one of them read a flagged untrusted item
    When the person opens the comparison of the two branches from the room's Room page
    Then each branch's exposure, results and evidence are shown side by side
    And the paths are aligned the same way on every comparison
    And no branch is marked the winner, and choosing a branch to compare is a neutral principal act of the room's owner

  @VIEW-14 @P1 @I6 @pending
  Scenario: every surface and reduced client uses the same words and a reduced client says what it left out
    Given an isolated Cairn home
    And a room with runs in several statuses and open Needs you items
    When the person views the room on its Room page in the browser, and in the TUI, the CLI, a paired phone and the harness strip
    Then the Room page and every reduced client show the same status words, marks, room ids and Needs you order
    And the TUI, the CLI, the paired phone and the harness strip, as reduced clients, each say what they left out and on which surface to see it

  @VIEW-15 @P1 @I2 @pending
  Scenario: room_get without an id returns only a closed structural set
    Given an isolated Cairn home
    And a room with a title, a key the person gave a petname, and 25 posts waiting for the caller's room
    When the agent calls "room_get" naming the room and no id
    Then the result is enveloped as structural and carries only the fields of the closed set
    And the waiting posts appear as addresses in range form, capped at 20 with a count of 5 left out
    And no field carries a title, label, branch name or petname
    And the harness strip and the banners the harness shows the person never enter the model's context

  @VIEW-16 @P1 @I4 @I6 @pending
  Scenario: each run shows the model tokens it used and an estimated cost from a local price table
    Given an isolated Cairn home
    And a run whose model token use is recorded, and a local price table
    When the person opens the run in the room view
    Then it shows the model tokens the run used
    And it shows a cost labelled as an estimate
    And no price is fetched

  @VIEW-17 @P1 @I2 @I4 @I6 @pending
  Scenario: notifications stay on the device, carry no room text and are counted when dropped
    Given an isolated Cairn home
    And an open loopback tab of the room view and a terminal, with the notification bridge (SEC-28) off
    When a Needs you item opens while one notification is suppressed
    Then the desktop notification is raised on the same device by the open loopback room view, and no vendor push service is used
    And it carries only the queue class, the room's petname, else its id, and a count
    And it never reaches a model and accepts no answer
    And the suppressed notification is counted
    And the terminal signal goes only to the terminal, never into hook output the harness adds to the model's context

  @VIEW-18 @P1 @I7 @pending
  Scenario: with no run recorded the room view opens on Setup
    Given an isolated Cairn home
    And no run recorded, so only the personal room, and harness transcripts due for deletion within 7 days
    When the person opens the room view
    Then it opens on Setup with the ingest command and the number of transcripts the harness will delete within 7 days
    And it shows the capture status and a statement that nothing leaves the machine, and Cairn creates no room for the person to browse
    And every change to harness configuration or configuration it offers is shown as a diff with the CLI command that applies it

  @VIEW-19 @P2 @I6 @I10 @pending
  Scenario: a needs-changes verdict shows what changed since it
    Given an isolated Cairn home
    And a room where "alice" recorded a "needs changes" verdict on C2, after which an agent edited two files
    When the person opens the verdict on the Review tab of the room's Room page
    Then it shows the diff and events since the branch heads that verdict was bound to

  @VIEW-20 @P1 @I6 @pending
  Scenario: the room view shows every delegation by its delegation link or parent link
    Given an isolated Cairn home
    And a room where an agent delegated one task to a subagent and one to another agent of the same principal under a delegation grant
    When the person opens the room view
    Then the delegation to the other agent shows as a delegation link from the delegating run to the delegate's run, naming the delegation grant
    And the subagent's delegation shows as the parent link from its run to the delegating run, naming no grant
    And the delegation link and the parent link each show the delegated task's address and its state
    And the other agent's spend as a delegate under the delegation grant shows against that grant's budget

  @VIEW-21 @P1 @I2 @I10 @pending
  Scenario: the room view puts the outcome beside the intent so a person can record a verdict
    Given an isolated Cairn home
    And a room whose intent names C1, C2 and C3, with a result linked to C1 by a criterion link and a result of evidence class claim, an agent's statement, linked to C2
    When its agents go idle and the person opens the Review tab of the room's Room page
    Then C1 shows its result and that result's evidence class, C2 shows its result of class claim and C3 reads "no evidence"
    And every criterion reads "no verdict"
    And the view shows edits outside the intent, the agents' exposure and the diff since the last verdict
    And a Q3 item "outcome awaiting a verdict" is raised
    And no verdict, score or suggestion derived by Cairn is shown

  @VIEW-22 @P1 @I2 @I10 @pending
  Scenario: the outcome window follows the room's pick, else the latest presentation
    Given an isolated Cairn home
    And a room with no facilitator, where seats "p-1" and "p-2" each have the present capability
    When "p-1" presents a dev server and then "p-2" presents a diff
    Then the outcome window shows "p-2"'s diff, as every node holding the record derives it
    When a moderator records a signed pick of "p-1"'s presentation
    Then the window shows "p-1"'s dev server
    And a pick by a seat with only present is refused
    And the person viewing can follow "p-2" in their own view, with no capability, without changing the window
    When the owner's device seat and the moderator concurrently pick different presentations
    Then the moderator's pick stands, by pick order: the facilitator's seat, then any other moderator, then the owner
    And Cairn chooses no branch and records no verdict of its own
