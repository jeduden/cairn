Feature: Non-functional requirements (NFR)

  Scenarios for SRS §7, one per requirement, tagged with its id. The
  rows carry no priority column and trace no invariants, so a scenario
  here carries its id alone. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @NFR-01 @pending
  Scenario Outline: hooks meet their p95 wall-clock budgets on a 1M-event project
    Given an isolated Cairn home
    And a synthetic project store with 1M events on the reference hardware
    When the hook "<Event>" runs 1,000 times with a representative payload
    Then the p95 wall-clock time is at most <budget>

    Examples:
      | Event            | budget |
      | UserPromptSubmit | 50 ms  |
      | SessionStart     | 150 ms |
      | PostToolUse      | 100 ms |
      | Stop             | 100 ms |
      | PreCompact       | 2 s    |
      | SessionEnd       | 1 s    |

  @NFR-02 @pending
  Scenario: a hook stops at its internal deadline and hands off the rest through a work marker
    Given an isolated Cairn home
    And a project with a Claude Code transcript "large-backlog"
    When the hook "SessionEnd" runs with a harness timeout of 1.5 s
    Then the hook exits 0 before its internal deadline, below the harness timeout
    And a work marker records the unfinished ingestion
    And the next "cairn ingest --all" completes the ingestion from the marker

  @NFR-03 @pending
  Scenario Outline: MCP recall meets its p95 latency at 10M events
    Given an isolated Cairn home
    And a synthetic project store with 10M events on the reference hardware
    When Claude calls the MCP tool "<tool>" with representative arguments 1,000 times
    Then the p95 latency excluding large payload transfer is at most <budget>

    Examples:
      | tool   | budget |
      | search | 200 ms |
      | expand | 100 ms |

  @NFR-04 @pending
  Scenario: ingestion sustains 5,000 events per second on one core
    Given an isolated Cairn home
    And a synthetic transcript of 1M events
    When the operator runs "cairn ingest --all" pinned to one core
    Then the command exits 0
    And the measured ingestion rate is at least 5,000 events/s

  @NFR-05 @pending
  Scenario: a project scales to 10M events, 100 GiB of payloads and 50 concurrent writers
    Given an isolated Cairn home
    And a synthetic project with 10M events, 100 GiB of payloads and one session above 10M tokens
    When 50 sessions and subagents ingest concurrently
    Then every event is stored and recallable by its seq
    And "cairn verify" exits 0

  @NFR-06 @pending
  Scenario Outline: an internal error fails open unless it touches I2, I4 or I8
    Given an isolated Cairn home
    And an injected internal fault "<fault>"
    When the hook "SessionStart" runs with a valid payload
    Then the hook exits <exit> with <output>
    And an audit entry records "<fault>"

    Examples:
      | fault                           | exit | output                        |
      | store query error               | 0    | empty output and no injection |
      | landmark builder panic          | 0    | empty output and no injection |
      | untrusted text reaching restore | 1    | no injection (fail closed)    |
      | network access attempted        | 1    | no injection (fail closed)    |
      | CAIRN_HOME owned by another UID | 1    | no injection (fail closed)    |

  @NFR-07 @pending
  Scenario: the store stays consistent when a process is killed at any point
    Given an isolated Cairn home
    And the crash-consistency harness
    When it kills Cairn at randomized points during ingestion 10,000 times
    Then "cairn verify" exits 0 after every kill
    And at most the in-flight transaction is lost
    And the next ingestion re-ingests the lost transaction

  @NFR-08 @pending
  Scenario: concurrent writers never corrupt, lose, or duplicate events
    Given an isolated Cairn home
    And 50 writers each appending 10,000 events to one project
    When all writers run concurrently to completion
    Then the store holds exactly 500,000 events
    And no two events share a seq value
    And "cairn verify" exits 0

  @NFR-09 @pending
  Scenario: no resident process, bounded hook memory and bounded store overhead
    Given an isolated Cairn home
    And a synthetic project store with 1M events
    When every hook runs once and the session ends
    Then no Cairn process remains running
    And each hook's peak RSS is at most 50 MiB
    And the database plus index size is at most 1.5 times the stored text size

  @NFR-10 @pending
  Scenario Outline: a single static binary builds for each supported platform
    Given the release build
    When it builds cairn for "<platform>"
    Then the result is one statically linked executable with no dynamic library dependencies

    Examples:
      | platform     |
      | linux/amd64  |
      | linux/arm64  |
      | darwin/arm64 |

  @NFR-11 @pending
  Scenario: the supported Claude Code versions are accepted and incompatible formats are loud
    Given recorded transcripts for the latest Claude Code release and the previous two minor versions
    And a transcript line with an unknown field and one with an unknown event type
    When the operator runs "cairn ingest --all"
    Then every supported transcript ingests and unknown lines are stored as unparsed events
    And a transcript in an incompatible format makes the command exit 1 with an audit entry

  @NFR-12 @pending
  Scenario: a new tenant reaches a working setup with defaults in five minutes
    Given a clean workstation with Claude Code installed and no Cairn configuration
    When a new tenant follows the documented plugin install
    Then "cairn doctor" exits 0 within 5 minutes of starting
    And no configuration file had to be written by hand

  @NFR-13 @pending
  Scenario: a new transcript format version changes only the harness adapter package
    Given the change that added the most recent transcript format version
    When its changed files are listed
    Then every changed non-test source file lies in the harness adapter package

  @NFR-14 @pending
  Scenario: every release ships its operator documentation
    Given a release artifact set
    When its documentation is listed
    Then it contains the operator guide, threat model, configuration reference, MCP tool reference and upgrade notes
