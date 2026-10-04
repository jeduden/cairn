Feature: Lane view (VIEW)

  Scenarios for SRS §5.12, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @VIEW-01 @P1 @I1 @I6 @pending
  Scenario: the lane view shows every harness of the tenant's lanes grouped by lane
    Given an isolated Cairn home
    And two lanes with one live and one recorded harness each
    When the person opens the lane view and selects one harness
    Then every harness appears grouped under its lane
    And the selected harness shows its prompts, tool calls, permission requests, output and each edit as a diff
    And each tool run shows its result and evidence class, and each event shows its actor
    And every other harness appears as a tile that opens full size

  @VIEW-02 @P1 @I9 @pending
  Scenario: the lane view shows a running session's lines within 2 s without slowing hooks
    Given an isolated Cairn home
    And ten local harnesses writing transcripts in one lane
    When the person watches one running session in the lane view
    Then each transcript line appears within 2 s of the harness writing it
    And no hook exceeds its NFR-01 budget while the view reads

  @VIEW-03 @P1 @I9 @I10 @pending
  Scenario: every lane-view surface is an optional, read-only client of the core
    Given an isolated Cairn home
    And a lane with recorded harnesses
    When the lane view runs and is then stopped
    Then the view reads the record only through the core's read path and never writes to it
    And the view holds no state the record cannot rebuild beyond per-viewer conveniences
    And every capability the view offers also exists in the CLI or MCP
    And hooks, ingestion and recall keep working with the view stopped

  @VIEW-04 @P1 @I6 @I10 @pending
  Scenario: statuses come from structural events only and unrecorded sessions surface
    Given an isolated Cairn home
    And a lane with one harness whose transcript is longer than its ingested position
    And a run-component launch with no hook event that belongs to no lane
    When the person opens Fleet and runs "cairn lanes"
    Then every harness shows exactly one status from the closed set of §9.7.1 with its freshness mark
    And the lane shows one lane status and the worst freshness mark of its harnesses, never Quiet
    And the unattached session appears as its own row on Fleet and in "cairn lanes"
    And text an agent wrote changes no status, and time-relative marks are computed in the viewer from an explicit boundary

  @VIEW-05 @P1 @I6 @I10 @pending
  Scenario: the Needs you queue is deterministic and clears everywhere once answered
    Given an isolated Cairn home
    And a lane with open items of classes Q1 and Q2 waiting on the person, one of them an overlap
    When the person answers one item in the CLI
    Then every surface lists the open items in the order of §9.7.4, the same for the same record
    And no Q1 or Q2 item can be dismissed, except the overlap item, which clears on acknowledgement
    And the answered item clears on every other surface as soon as the answer's event arrives there

  @VIEW-06 @P1 @I2 @I6 @pending
  Scenario: written text is shown apart, marked untrusted and never sets a status
    Given an isolated Cairn home
    And a harness whose agent wrote "all tests pass" in its output
    When the person opens that harness in the lane view
    Then the text is shown apart from derived lines and marked untrusted
    And it is marked as a "claim"
    And no status, evidence or proof mark changes because of it

  @VIEW-07 @P1 @I2 @pending
  Scenario: items from outside the trusted boundary carry a trust mark and the owner's petname
    Given an isolated Cairn home
    And synced posts from a peer key with a petname, a key with none, and a new key using a known name
    And one post holding zero-width, bidirectional and tag characters and an HTML comment
    When the person opens the lane view
    Then each post carries the trust mark of §9.7.6 and its author's petname, never the name the peer sent
    And the key with no petname is shown by its fingerprint and the new key is marked "new key"
    And the hidden characters and the HTML comment render as visible tokens with a count

  @VIEW-08 @P1 @I6 @I10 @pending
  Scenario: Catch up is one deterministic projection of the record and a boundary
    Given an isolated Cairn home
    And lanes with capture gaps, an unsynced writer, a tombstone, a lane merge and finished work since a boundary
    And a receipt of chain heads outside CAIRN_HOME
    When the person opens Catch up at that boundary
    Then it shows the boundary it used and where it came from, and links every line to its events
    And it lists capture gaps, uningested transcripts, risen failure counters and unsynced writers apart from quiet lanes
    And it checks each writer's chain head against the named receipt and shows the result per writer
    And its lines run integrity and capture gaps, Needs you, failures, then finished work, with its frontier per writer and no model-written line

  @VIEW-09 @P1 @I6 @I8 @pending
  Scenario Outline: operator search uses the scope the person selects and states its coverage
    Given an isolated Cairn home
    And a tenant with two lanes, an unsynced writer and a quarantined match
    When the person searches "fix (login" with scope "<scope>"
    Then the result list shows the scope "<scope>" it used
    And the typed text is matched as literal terms
    And recall events are left out of ranking
    And below the results it states what it covered and left out, naming the unsynced writer and the quarantined match

    Examples:
      | scope      |
      | session    |
      | lane       |
      | project    |
      | every lane |

  @VIEW-10 @P1 @I6 @I10 @pending
  Scenario: every lane shows its integrity seal and the view writes a receipt outside the home
    Given an isolated Cairn home
    And a lane whose writer's chain breaks at one event
    When the person opens the lane and writes a receipt to a path outside CAIRN_HOME
    Then the lane shows its seal from §9.7.5 at all times
    And every later event of that writer is marked unverified wherever it is shown, including in recall results
    And the receipt is shown as a short code carrying at least 80 bits of the heads' digest
    And the view states that verification proves the sealed record unchanged up to its newest seal, not the unsigned tail and not its content true

  @VIEW-11 @P1 @I1 @I5 @I6 @pending
  Scenario: gaps, quarantines and tombstones stay in place and forensic reads are recorded
    Given an isolated Cairn home
    And a lane with a missing segment, a quarantined range and a tombstone
    When the person opens its timeline and asks to see the quarantined content
    Then each gap is shown in place, never closed up
    And the quarantined content is shown only after the explicit forensic request
    And the request is recorded as an "operator" event
    And the tombstone reads "removed from this node", never "erased"

  @VIEW-12 @P1 @I1 @I10 @pending
  Scenario: replay reconstructs any event from the record alone
    Given an isolated Cairn home
    And a lane with two writers, edits, tool runs and a recall
    When the person replays the lane to one event
    Then the conversation, worktree and results at that event come from the record alone and nothing is re-executed
    And the worktree states its fidelity as "exact", "approximate" or "unavailable", and why
    And the context lens marks itself a reconstruction and shows recalled content inside its envelope
    And events of the two writers appear in the same causal order on every replay

  @VIEW-13 @P2 @I10 @pending
  Scenario: comparing two lanes shows exposure beside outcome and picks no winner
    Given an isolated Cairn home
    And two lanes for the same task, one of which read a flagged untrusted item
    When the person compares the two lanes
    Then exposure, outcome and evidence are shown side by side for each lane
    And the paths are aligned the same way on every comparison
    And no lane is marked the winner until the owner keeps one

  @VIEW-14 @P1 @I6 @pending
  Scenario: every surface uses the same words and a reduced surface says what it left out
    Given an isolated Cairn home
    And a lane with harnesses in several statuses and open Needs you items
    When the person views the lane in the lane view, the TUI, the CLI and the harness strip
    Then every surface shows the same status words, marks, lane ids and Needs you order
    And each reduced surface says what it left out and where to see it

  @VIEW-15 @P1 @I2 @pending
  Scenario: lane_status returns only a closed structural set
    Given an isolated Cairn home
    And a lane with a titled post, a petnamed peer and 25 posts waiting for the caller's lane
    When the agent calls "lane_status"
    Then the result is enveloped as structural and holds only the fields of the closed set
    And the waiting posts appear as recall addresses in range form, capped at 20 with a count of 5 left out
    And no field holds a title, label, branch name or petname
    And the status line and banners shown to the human never enter the model's context

  @VIEW-16 @P1 @I4 @I6 @pending
  Scenario: each harness shows its token use and an estimated cost from a local price table
    Given an isolated Cairn home
    And a harness with recorded token use and a local price table
    When the person opens the harness in the lane view
    Then it shows the harness's token use
    And it shows a cost labelled as an estimate
    And no price is fetched

  @VIEW-17 @P1 @I2 @I4 @I6 @pending
  Scenario: notifications stay local, carry no lane text and are counted when dropped
    Given an isolated Cairn home
    And an open loopback tab of the lane view and a terminal session
    When a Needs you item opens while one notification is suppressed
    Then the desktop notification is raised on the same device by the open loopback lane view, with no vendor push service
    And it carries only the queue class, the owner's lane alias and a count
    And the suppressed notification is counted
    And the terminal signal goes only to the terminal, never into hook output the harness adds to the model's context

  @VIEW-18 @P1 @I7 @pending
  Scenario: with zero lanes the lane view opens on Setup
    Given an isolated Cairn home
    And no lanes and harness transcripts due for deletion within 7 days
    When the person opens the lane view
    Then it opens on Setup with the import command and the number of transcripts the harness will delete within 7 days
    And it shows the recording status, a read-only sample lane and a statement that nothing leaves the machine
    And every configuration change it offers is shown as a diff with the CLI command that applies it

  @VIEW-19 @P2 @I6 @I10 @pending
  Scenario: a request for changes shows its delivery state and what changed since its verdict
    Given an isolated Cairn home
    And a lane with a request for changes addressed to two agents
    When the person opens the request in the lane
    Then it shows the delivery state to each agent's principal
    And it shows the diff and events since that verdict's head

  @VIEW-20 @P1 @I6 @pending
  Scenario: the lane view shows every delegation as a link
    Given an isolated Cairn home
    And a lane where an agent delegated one task to a subagent and one to another session under a grant
    When the person opens the lane view
    Then each delegation shows as a link from the delegating agent to its delegate
    And each link shows its grant, the task's address and its state
    And the delegate's spend shows against the grant's budget
