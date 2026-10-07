Feature: Non-functional requirements (NFR)

  Scenarios for SRS §7, one per requirement, tagged with its id. The
  rows carry no priority column and trace no invariants, so a scenario
  here carries its id alone. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @NFR-01 @pending
  Scenario Outline: hook handlers meet their p95 wall-clock budgets on a 1M-event store under fleet load
    Given an isolated Cairn home
    And a synthetic store with 1M events on the reference hardware
    And the room view is open and ten harnesses are writing
    When the hook "<Event>" runs 1,000 times with a representative payload
    Then the p95 wall-clock time is at most <budget>

    Examples:
      | Event             | budget                                                                                                                                      |
      | UserPromptSubmit  | 50 ms                                                                                                                                       |
      | SessionStart      | 150 ms                                                                                                                                      |
      | PostToolUse       | 100 ms                                                                                                                                      |
      | Stop              | 100 ms                                                                                                                                      |
      | SubagentStop      | 100 ms                                                                                                                                      |
      | Notification      | 100 ms                                                                                                                                      |
      | PreCompact        | 2 s                                                                                                                                         |
      | SessionEnd        | 1 s                                                                                                                                         |
      | PermissionRequest | 50 ms of Cairn's own processing beyond the wait, which ends within the principal's hold window and at least 10 s before the harness timeout |

  @NFR-02 @pending
  Scenario: a hook handler stops at its internal deadline and hands off the rest through an ingest marker
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "large-backlog"
    When the hook "SessionEnd" runs with a harness timeout of 1.5 s
    Then the hook handler exits 0 before its internal deadline, below the harness timeout
    And an ingest marker records the unfinished ingestion
    And the next "cairn ingest --all" completes the ingestion from the marker

  @NFR-03 @pending
  Scenario Outline: MCP recall meets its p95 latency at 10M events
    Given an isolated Cairn home
    And a synthetic store with 10M events on the reference hardware
    When the agent calls the MCP tool "<tool>" with representative arguments 1,000 times
    Then the p95 latency excluding large payload transfer is at most <budget>

    Examples:
      | tool         | budget |
      | event_search | 200 ms |
      | event_expand | 100 ms |

  @NFR-04 @pending
  Scenario: ingestion sustains 5,000 events per second on one core
    Given an isolated Cairn home
    And a synthetic transcript of 1M events
    When the person runs "cairn ingest --all" bound to one CPU core
    Then the command exits 0
    And the measured ingestion rate is at least 5,000 events/s

  @NFR-05 @pending
  Scenario: a store scales to 10M events, 100 GiB of payloads and 50 concurrent runs
    Given an isolated Cairn home
    And a synthetic store with 10M events, 100 GiB of payloads and one run above 10M model tokens
    When 50 runs, subagents included, ingest concurrently
    Then every event is stored and recallable by its address (writer, seq)
    And "cairn verify" exits 0

  @NFR-06 @pending
  Scenario Outline: an internal error fails open unless it touches I2, I4 or I8
    Given an isolated Cairn home
    And an injected internal fault "<fault>"
    When the hook "SessionStart" runs with a valid payload
    Then the hook handler exits <exit> with <output>
    And an audit entry records "<fault>"

    Examples:
      | fault                                   | exit | output                        |
      | store query error                       | 0    | empty output and no injection |
      | landmark builder panic                  | 0    | empty output and no injection |
      | untrusted text reaching a restore block | 1    | no injection (fail closed)    |
      | network access attempted                | 1    | no injection (fail closed)    |
      | CAIRN_HOME of another UID               | 1    | no injection (fail closed)    |

  @NFR-07 @pending
  Scenario: the store stays consistent when a process is killed at any point
    Given an isolated Cairn home
    And the crash-consistency test
    When it kills Cairn at randomized points during ingestion 10,000 times
    Then "cairn verify" exits 0 after every kill
    And at most the in-flight transaction is lost
    And the next ingestion re-ingests the lost transaction

  @NFR-08 @pending
  Scenario: concurrent appends never corrupt, lose, or duplicate events
    Given an isolated Cairn home
    And 50 writers each appending 10,000 events to one store
    When all appends run concurrently to completion
    Then the store contains exactly 500,000 events
    And no two events share an address (writer, seq), and no seq is duplicated within a writer's log
    And "cairn verify" exits 0

  @NFR-09 @pending
  Scenario: the core leaves no resident process and the launcher and every other component outside the core stay within their footprint
    Given an isolated Cairn home
    And a synthetic store with 1M events on the reference hardware
    When every hook runs once, the run ends, and the room-view, peer, publish and bridge components and ten launcher instances run idle
    Then no Cairn process runs while no run is active except the components the person started
    And each hook handler's peak RSS is at most 50 MiB and the store overhead is at most 1.5 times the stored text
    And each of the room-view, peer, publish and bridge components peaks at most 256 MiB RSS and idles at most 5% of one core
    And the ten launcher instances together peak at most 256 MiB RSS and idle at most 5% of one core
    And the launcher adds at most 10 ms p95 to keystroke-to-echo latency

  @NFR-10 @pending
  Scenario Outline: the core executable builds for each supported platform
    Given the release build
    When it builds the cairn core executable for "<platform>"
    Then the result is one executable that links <linking>

    Examples:
      | platform     | linking                                          |
      | linux/amd64  | no dynamic library at all                        |
      | linux/arm64  | no dynamic library at all                        |
      | darwin/arm64 | no dynamic library beyond the operating system's |

  @NFR-11 @pending
  Scenario: the supported Claude Code versions are accepted and incompatible formats are loud
    Given recorded transcripts for the latest Claude Code release and the previous two minor versions
    And a transcript line with an unknown field and one with an unknown event type
    When the person runs "cairn ingest --all"
    Then every supported transcript ingests and unknown lines are stored as unparsed events
    And a transcript in an incompatible format makes the command exit 1 with an audit entry

  @NFR-12 @pending
  Scenario: a new person reaches a working setup with defaults in five minutes
    Given a clean workstation with Claude Code installed and no Cairn configuration
    When a new person follows the documented plugin install
    Then "cairn doctor" exits 0 within 5 minutes of starting
    And no configuration file had to be written by hand

  @NFR-13 @pending
  Scenario: a new transcript format version changes only the transcript and hook part of the harness adapter
    Given the change that added the most recent transcript format version
    When its changed files are listed
    Then every changed non-test code file lies in that harness adapter's transcript and hook part, in the core

  @NFR-14 @pending
  Scenario: every release ships its documentation
    Given a release artifact set
    When its documentation is listed
    Then it contains the administration guide, threat model, settings reference, MCP tool reference and upgrade notes

  @NFR-15 @pending
  Scenario Outline: the room view's surfaces meet their p95 targets on a 10M-event node and say when they miss
    Given an isolated Cairn home
    And a node holding 10M events across 50 rooms on the reference hardware
    When "<action>" is measured 1,000 times
    Then the p95 time is at most <budget>
    And any surface that misses its target says so on screen

    Examples:
      | action                                     | budget |
      | an ingested event appears in the room view | 1 s    |
      | Catch up paints                            | 1 s    |
      | search shows its first results             | 300 ms |
      | a hit opens in context                     | 150 ms |
      | replay steps from one event to the next    | 50 ms  |
      | replay rebuilds a worktree                 | 500 ms |
