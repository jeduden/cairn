Feature: Record (REC)

  Scenarios for SRS §5.1, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @REC-01 @P0 @I1 @pending
  Scenario Outline: session transcripts are ingested from the configured transcript roots
    Given an isolated Cairn home
    And the tenant configuration <roots>
    And a project with a Claude Code transcript "main-session"
    And "main-session" lies under "<root>"
    When the operator runs "cairn ingest --all"
    Then the command exits 0
    And every line of "main-session" is stored as an event of its session

    Examples:
      | roots                                             | root                 |
      | leaves transcript_roots unset                     | ~/.claude/projects   |
      | sets transcript_roots to ["~/runner-transcripts"] | ~/runner-transcripts |

  @REC-02 @P0 @I1 @pending
  Scenario: a subagent transcript is linked to its parent session
    Given an isolated Cairn home
    And a project with a Claude Code transcript "parent-session"
    And a subagent transcript "explore-agent" of "parent-session" naming the agent "Explore"
    When the operator runs "cairn ingest --all"
    Then the source "explore-agent" records "parent-session" as its parent session
    And the source "explore-agent" records the agent identity "Explore"

  @REC-03 @P0 @I1 @I10 @pending
  Scenario: re-ingesting a source creates no duplicate events
    Given an isolated Cairn home
    And a project with a Claude Code transcript "main-session"
    And "main-session" holds 40 lines and has been ingested, then grows by 10 lines
    When the operator runs "cairn ingest --all"
    And the operator runs "cairn ingest --all"
    Then the project holds exactly 50 events, one per line of "main-session"
    And the cursor of "main-session" holds its byte length and the SHA-256 of its consumed prefix

  @REC-04 @P0 @I1 @pending
  Scenario: events are attributed to the session of their transcript, not the hook payload
    Given an isolated Cairn home
    And a project with a Claude Code transcript "parent-session"
    And a subagent transcript "explore-agent" of "parent-session" naming the agent "Explore"
    When the hook "PostToolUse" runs with the session_id of "parent-session" and the transcript_path of "explore-agent"
    Then every event read from "explore-agent" is attributed to the subagent session of "explore-agent"
    And no event read from "explore-agent" is attributed to "parent-session"

  @REC-05 @P0 @I1 @I6 @pending
  Scenario: unreadable transcript lines are kept as redacted unparsed events
    Given an isolated Cairn home
    And a project with a Claude Code transcript "main-session"
    And "main-session" holds a malformed JSON line and a line of unknown type "future_kind", each containing an API key
    When the operator runs "cairn ingest --all"
    Then both lines are stored as events of kind "unparsed" holding the raw line with the API key redacted
    And no line of "main-session" is missing from the record
    And the counter "events_unparsed" increases by 2

  @REC-06 @P0 @I1 @I10 @pending
  Scenario: seq is strictly increasing, gap-free and never reused
    Given an isolated Cairn home
    And a project with a Claude Code transcript "main-session"
    When the operator runs "cairn ingest --all" while the hook "PostToolUse" ingests "main-session" concurrently
    Then the project's events carry seq 1 to N with no gap and no repeat
    And an append transaction that rolls back consumes no seq
    When the operator runs "cairn purge --range 5-9" and more lines of "main-session" are ingested
    Then the new events continue from seq N+1 and no seq is reused

  @REC-07 @P0 @I1 @I6 @pending
  Scenario Outline: a shrunk or rewritten source starts a new generation without losing events
    Given an isolated Cairn home
    And a project with a Claude Code transcript "main-session"
    And "main-session" holds 40 lines and has been ingested
    When "main-session" is <change>
    And the operator runs "cairn ingest --all"
    Then "main-session" starts generation 2 and all 40 events of generation 1 remain in the record
    And an audit entry records "new source generation for main-session"

    Examples:
      | change                                |
      | truncated to 10 lines                 |
      | rewritten with a different first line |

  @REC-08 @P0 @I1 @I6 @pending
  Scenario: the record outlives its sources and verify reports sources lost before full ingestion
    Given an isolated Cairn home
    And a project with a Claude Code transcript "main-session"
    And "main-session" holds 40 lines and has been ingested
    And "cut-short" holds 30 lines of which 20 were ingested before both transcript files were deleted
    When the operator runs "cairn verify --json"
    Then the report names "cut-short", and only it, as a source that disappeared before it was fully ingested
    And "cairn expand" still returns all 40 events of "main-session"

  @REC-09 @P0 @I1 @pending
  Scenario: content above the payload threshold goes to the content-addressed payload store
    Given an isolated Cairn home
    And a project with a Claude Code transcript "main-session"
    And "main-session" holds a Bash tool result of 20 KiB and one of 2 KiB, against the default 8 KiB threshold
    When the operator runs "cairn ingest --all"
    Then the 20 KiB content is stored at "payloads/ab/cd/<sha256>" keyed by its SHA-256, and its event holds a preview of at most 512 bytes and the payload reference
    And the 2 KiB content is stored inline in its event
    And each payload was written to a temporary file, fsynced, then renamed into place

  @REC-10 @P0 @I10 @pending
  Scenario: events form a per-project hash chain over their canonical encoding
    Given an isolated Cairn home
    And a project with a Claude Code transcript "main-session"
    And "main-session" holds 40 lines and has been ingested
    When the operator runs "cairn verify"
    Then the command exits 0
    And each event's hash is the SHA-256 of its prev_hash followed by the RFC 8785 encoding of the event, payload hash included
    And altering one byte of a stored payload makes "cairn verify" exit 3

  @REC-11 @P0 @pending
  Scenario: event and payload text is full-text indexed up to the indexing cap
    Given an isolated Cairn home
    And a project with a Claude Code transcript "main-session"
    And "main-session" holds a 2 MiB tool result with "quokka-early" in its first KiB and "quokka-late" beyond its first 1 MiB
    When the operator runs "cairn ingest --all"
    And Claude calls the MCP tool "search" with query "quokka-early"
    Then the tool result event is a hit
    And a search for "quokka-late" returns no hit, because indexing stops at the default 1 MiB cap

  @REC-12 @P0 @I10 @pending
  Scenario: events are appended in source order and seq alone defines order
    Given an isolated Cairn home
    And a project with a Claude Code transcript "main-session"
    And the timestamp of line 11 of "main-session" is earlier than that of line 10
    And line 12 of "main-session" carries no timestamp
    When the operator runs "cairn ingest --all"
    Then the events of "main-session" have seq in the order of their source lines
    And each event keeps its source timestamp as metadata
    And the event of line 12 carries no source timestamp
    And a second fresh home that ingests "main-session" holds the same chain head hash
    And "cairn expand" returns line 10 before line 11

  @REC-13 @P1 @I9 @pending
  Scenario Outline: tool-use and stop hooks ingest incrementally within budget and mark the remainder
    Given an isolated Cairn home
    And a project with a Claude Code transcript "main-session"
    And "main-session" holds 50,000 unread lines, more than one hook budget can ingest
    When the hook "<hook>" runs with the session_id and transcript_path of "main-session"
    Then the hook exits 0 within its 100 ms budget
    And the events ingested so far are a prefix of "main-session"
    And a work marker for the remainder exists under the project's "work/" directory

    Examples:
      | hook        |
      | PostToolUse |
      | Stop        |

  @REC-14 @P1 @I1 @pending
  Scenario: Claude Agent SDK transcripts are ingested from configurable roots
    Given an isolated Cairn home
    And the tenant configuration adds a root holding a Claude Agent SDK transcript "sdk-run"
    When the operator runs "cairn ingest --all"
    Then the command exits 0
    And every line of "sdk-run" is stored as an event of its session

  @REC-15 @P1 @I1 @I5 @pending
  Scenario: retention policies expire content per provenance class through tombstoned purges
    Given an isolated Cairn home
    And a project whose record holds "web" payloads and "user" events that are 40 days old
    And the tenant configuration sets "retention.web" to 30 days
    When the retention policy is applied
    Then the content of the "web" events is removed and a tombstone records each purged range with reason "retention"
    And the "user" events are kept and "cairn verify" exits 0
    And with no retention configured, applying the default policy removes nothing

  @REC-16 @P2 @I1 @pending
  Scenario: a Claude Managed Agents event history can be imported
    Given an isolated Cairn home
    And an exported Claude Managed Agents event history "managed-run"
    When the operator runs "cairn ingest --path managed-run"
    Then the command exits 0
    And every event of "managed-run" is stored with its session and a provenance class
