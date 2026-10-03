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
  Scenario: every event is addressed by its writer and a gap-free seq that writer alone assigns
    Given an isolated Cairn home
    And a project with a Claude Code transcript "main-session" whose writer has appended seq 1 to 20 and purged seq 5-9
    And an append transaction of that writer that rolled back
    When the operator runs "cairn ingest --all" while the hook "PostToolUse" ingests "main-session" concurrently
    Then every event carries the address (writer, seq) of the writer that appended it
    And that writer's new events continue from seq 21 with no gap and no repeat, and no seq is reused
    And each seq was assigned by its writer inside the append transaction that wrote its event
    And a local index position is never shown or accepted as an address

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
  Scenario: content above the payload threshold goes to a payload store named by a keyed hash
    Given an isolated Cairn home
    And a project with a Claude Code transcript "main-session"
    And "main-session" holds a Bash tool result of 20 KiB and one of 2 KiB, against the default 8 KiB threshold
    When the operator runs "cairn ingest --all"
    Then the 20 KiB content is stored in the payload store under a keyed hash made with the tenant's local storage key, and its event holds a preview of at most 512 bytes, the payload reference and the commitment
    And the 2 KiB content is stored inline in its event
    And the tenant's local storage key appears in no export, lane bundle or replicated structure
    And each payload was written to a temporary file, fsynced, then renamed into place

  @REC-10 @P0 @I10 @pending
  Scenario: each writer's events form their own hash chain over a header that holds no content
    Given an isolated Cairn home
    And a project in which two writers have each appended 20 events, each having seen the other's log, the second writer's clock running an hour behind
    When the operator runs "cairn verify"
    Then the command exits 0
    And each event's hash is a collision-resistant hash of prev_hash, its writer's previous event, followed by the canonical encoding of its header
    And the header holds only the address, kind, provenance class, prev_hash, the heads of the other writers' logs the writer had seen, and the commitment
    And events of the two writers are ordered by the heads each cites, never by wall-clock time

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
    When the operator runs "cairn ingest --all"
    Then the events of "main-session" have seq in the order of their source lines
    And each event keeps its source timestamp as metadata
    And "cairn expand" returns line 10 before line 11

  @REC-13 @P1 @I9 @pending
  Scenario Outline: tool-use, stop and permission hooks ingest incrementally within budget and mark the remainder
    Given an isolated Cairn home
    And a project with a Claude Code transcript "main-session"
    And "main-session" holds 50,000 unread lines, more than one hook budget can ingest
    When the hook "<hook>" runs with the session_id and transcript_path of "main-session"
    Then the hook exits 0 within its budget
    And the events ingested so far are a prefix of "main-session"
    And a work marker records the remainder of "main-session"

    Examples:
      | hook              |
      | PostToolUse       |
      | Stop              |
      | SubagentStop      |
      | PermissionRequest |

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

  @REC-17 @P0 @I1 @I5 @I10 @pending
  Scenario: event content is reached only through a keyed commitment whose key is erased with it
    Given an isolated Cairn home
    And a project whose record holds 40 events, of which seq 10-14 have since been purged
    When the operator runs "cairn verify"
    Then the command exits 0
    And every event carries a commitment to its canonical content under a random per-event key of at least 256 bits, stored with the content and absent from its chained header
    And the hash chain, every seal, every tombstone and every exported structure refer to event content, structural fields included, only through the commitment
    And the commitment keys of seq 10-14 were erased with their content
    And no retained or exported value confirms a guess at the purged content

  @REC-18 @P1 @I6 @I10 @pending
  Scenario: writer seals are verified and events past the newest seal show as unsigned
    Given an isolated Cairn home
    And a writer whose log holds 30 events and is sealed at seq 20 with its writer key
    And a second writer whose seal at seq 10 has one altered byte
    When the operator runs "cairn verify"
    Then the command exits 3 and names the second writer's seal at seq 10 as a mismatch
    And the first writer's seal, over its writer id, seq 20 and the chain head at seq 20, verifies against its public key
    And events 21 to 30 of the first writer are shown and recalled as "unsigned"

  @REC-19 @P1 @I1 @I6 @pending
  Scenario Outline: a writer seals after every appending hook and closes its segment at every stop and every 30 s
    Given an isolated Cairn home
    And a running session whose writer has appended events in "PostToolUse" hooks for 31 s
    And another writer whose chain ended without a closed segment
    When the hook "<hook>" runs and appends events
    Then the writer sealed its log at the end of every hook invocation that appended events, within the hook budgets
    And a segment was closed once 30 s had passed though no seal closed one, and the open segment is closed at "<hook>"
    And "cairn verify" and "cairn status" each report the other writer's chain as ended without a closed segment

    Examples:
      | hook         |
      | Stop         |
      | SubagentStop |
      | SessionEnd   |

  @REC-20 @P1 @I1 @pending
  Scenario Outline: worktree checkpoints record the commit, branch and redacted diff at every hand-off point
    Given an isolated Cairn home
    And a session on a git worktree with a previous checkpoint, a tracked change holding an API key, an untracked file and an ignored file
    When <moment>
    Then the record gains an event with provenance "file" holding the checked-out commit id, the branch and a payload diff against the previous checkpoint
    And the diff covers the tracked change and the untracked file but not the ignored file, with the API key redacted
    And a checkpoint taken for a rewrite of the branch head keeps the replaced head
    And Cairn read the worktree and git's files without starting a process, within the hook budget, leaving any rest to a work marker

    Examples:
      | moment                                   |
      | the hook "Stop" runs                     |
      | the hook "SessionEnd" runs               |
      | the agent hands the lane back            |
      | the branch head is rewritten by a rebase |

  @REC-21 @P1 @I6 @pending
  Scenario: a segment of an unsupported format version is refused, audited and left untouched
    Given an isolated Cairn home
    And a writer log holding a segment of a format version this node supports and a segment of format version 99
    When the operator runs "cairn rebuild"
    Then every segment this node wrote carries a format version
    And the supported segment is read
    And the version 99 segment is refused and an audit entry records its unsupported format version
    And the version 99 segment file is byte-identical to before

  @REC-22 @P1 @I1 @I2 @pending
  Scenario: events from transcripts the hooks did not observe are marked imported and untrusted
    Given an isolated Cairn home
    And deployment mode "interactive"
    And a transcript "pre-install" of this project, written before Cairn's hooks were installed, holding a user prompt
    And a transcript "elsewhere" from outside the harness's own transcript directory for this project
    When the operator runs "cairn ingest --path" on "pre-install" and on "elsewhere"
    Then every event of "pre-install" carries an "imported" mark with its source and ingest position, shown on every surface, and none is shown as witnessed
    And the user prompt from "pre-install" has trust "untrusted"
    And "elsewhere" is held as a foreign lane shown as "imported", with no publisher key and every writer "unbound"

  @REC-23 @P2 @I2 @I4 @I6 @pending
  Scenario Outline: lane bundles are imported only from local sources, verified, redacted and audited
    Given an isolated Cairn home
    And this node holds a foreign writer's chain under the owner key "owner-b"
    And a lane bundle in a local file that <bundle>, holding a withheld event and an event with an API key the tenant's redaction rules match
    When the operator imports the bundle
    Then the import is <outcome> and an audit entry records it
    And an accepted bundle had its seals and chains verified as received, across the withheld event from its retained header, and the API key redacted with its event's commitment key erased, both results recorded
    And accepted events form a foreign lane
    And a bundle named by a URL is refused without network access, while one at a git ref already fetched into a local clone is read

    Examples:
      | bundle                                               | outcome  |
      | continues the foreign writer's chain under "owner-b" | accepted |
      | forks the foreign writer's chain under "owner-b"     | refused  |
      | claims a lane of this tenant                         | refused  |
      | claims a writer of this tenant                       | refused  |

  @REC-24 @P1 @I1 @I8 @I10 @pending
  Scenario Outline: a home that moved to another node gets a new writer key and appends nothing under the old one
    Given an isolated Cairn home
    And a writer key bound to the node identity read, from outside the home, on the node the home was created on
    And <change>
    When the hook "SessionStart" runs and appends its first event after the start
    Then Cairn mints a new writer key before that append
    And an audit entry records the node identity change
    And no event is appended under the old writer key

    Examples:
      | change                                                    |
      | the home is part of a cloned runner image on another node |
      | the home is a copied volume mounted on another node       |
      | the home is restored from a snapshot on another node      |
      | the node identity value is unavailable                    |

  @REC-25 @P1 @I1 @I10 @pending
  Scenario: closed segments are merged in the background so a writer holds at most 48 segment files a day
    Given an isolated Cairn home
    And a writer that closed 200 sealed segments during one day of activity
    When a later hook's work marker runs in the background
    Then the writer holds at most 48 segment files for that day
    And every address, commitment and chain head is unchanged
    And "cairn verify" checks every seal and exits 0
