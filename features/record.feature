Feature: Record (REC)

  Scenarios for SRS §5.1, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @REC-01 @P0 @I1 @pending
  Scenario Outline: harness transcripts are ingested from the configured transcript roots
    Given an isolated Cairn home
    And the person's configuration <roots>
    And an agent run with a Claude Code transcript "main-run"
    And "main-run" lies under "<root>"
    When the person runs "cairn ingest --all"
    Then the command exits 0
    And every line of "main-run" is stored as an event of its run

    Examples:
      | roots                                             | root                 |
      | leaves transcript.roots unset                     | ~/.claude/projects   |
      | sets transcript.roots to ["~/runner-transcripts"] | ~/runner-transcripts |

  @REC-02 @P0 @I1 @pending
  Scenario: a subagent is ingested as its own run, tied to its parent's run by a parent link
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "parent-run"
    And a subagent transcript "explore-agent" of "parent-run" naming the agent "Explore"
    When the person runs "cairn ingest --all"
    Then the run ingested from "explore-agent" is tied to the run of "parent-run" by a parent link, not a delegation link
    And the run ingested from "explore-agent" records the harness's agent id "Explore"

  @REC-03 @P0 @I1 @I10 @pending
  Scenario: re-ingesting a transcript source creates no duplicate events
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "main-run"
    And "main-run" has 40 lines and has been ingested, then grows by 10 lines
    When the person runs "cairn ingest --all"
    And the person runs "cairn ingest --all"
    Then the record contains exactly 50 events, one per line of "main-run"
    And the cursor of "main-run" stores its byte length and the SHA-256 of its consumed prefix

  @REC-04 @P0 @I1 @pending
  Scenario: events are attributed to the run of their transcript, not the hook input
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "parent-run"
    And a subagent transcript "explore-agent" of "parent-run" naming the agent "Explore"
    When the hook "PostToolUse" runs with the session_id of "parent-run" and the transcript_path of "explore-agent"
    Then every event read from "explore-agent" is attributed to the subagent run of "explore-agent"
    And no event read from "explore-agent" is attributed to "parent-run"

  @REC-05 @P0 @I1 @I6 @pending
  Scenario: unreadable transcript lines are kept as redacted unparsed events
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "main-run"
    And "main-run" contains a malformed JSON line and a line of unknown type "future_kind", each containing an API key
    When the person runs "cairn ingest --all"
    Then both lines are stored as events with provenance "unparsed" carrying the raw line with the API key redacted
    And no line of "main-run" is missing from the record
    And the counter "events_unparsed" increases by 2

  @REC-06 @P0 @I1 @I10 @pending
  Scenario: every event's address is its writer and a gap-free seq that writer alone assigns
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "main-run" whose writer has appended seq 1 to 20 and purged seq 5-9
    And an append transaction of that writer that rolled back
    When the person runs "cairn ingest --all" while the hook handler for "PostToolUse" ingests "main-run" concurrently
    Then every event carries the address (writer, seq) of the writer that appended it
    And that writer's new events continue from seq 21 with no gap and no repeat, and no seq is reused
    And each seq was assigned by its writer inside the append transaction that wrote its event
    And a local index position is never shown or accepted as an address

  @REC-07 @P0 @I1 @I6 @pending
  Scenario Outline: a shrunk or rewritten transcript source starts a new transcript generation without losing events
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "main-run"
    And "main-run" has 40 lines and has been ingested
    When "main-run" is <change>
    And the person runs "cairn ingest --all"
    Then "main-run" starts transcript generation 2 and all 40 events of transcript generation 1 remain in the record
    And an audit entry records "new transcript generation of main-run"

    Examples:
      | change                                |
      | truncated to 10 lines                 |
      | rewritten with a different first line |

  @REC-08 @P0 @I1 @I6 @pending
  Scenario: the record outlives its transcript sources and verify reports transcript sources lost before full ingestion
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "main-run"
    And "main-run" has 40 lines and has been ingested
    And "cut-short" has 30 lines of which 20 were ingested before both transcript files were deleted
    When the person runs "cairn verify --json"
    Then the output names "cut-short", and only it, as a transcript source that disappeared before it was fully ingested
    And "cairn event expand" still returns all 40 events of "main-run"

  @REC-09 @P0 @I1 @pending
  Scenario: content above the payload threshold goes to a payload store under a name that reveals nothing off the node
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "main-run"
    And "main-run" contains a Bash tool result of 20 KiB and one of 2 KiB, against the default 8 KiB threshold
    When the person runs "cairn ingest --all"
    Then the 20 KiB content is stored in the payload store under a name from which no one off this node can confirm a guess at the content, and its event carries a preview of at most 512 bytes, the payload reference and the commitment
    And the 2 KiB content is stored inline in its event
    And nothing in an export, room bundle or replicated structure lets a reader off this node confirm a guess at the content from a payload name
    And each payload was written to a temporary file, fsynced, then renamed into place

  @REC-10 @P0 @I10 @pending
  Scenario: each writer's events form their own hash chain over a header that carries no content
    Given an isolated Cairn home
    And a record in which two writers have each appended 20 events, each having seen the other's log, the second writer's clock running an hour behind
    When the person runs "cairn verify"
    Then the command exits 0
    And each event's hash is a collision-resistant hash of prev_hash, its writer's previous event, followed by the canonical encoding of its header
    And the header contains only the address, kind, provenance class, prev_hash, the heads of the other writers' logs the writer had seen, and the commitment
    And events of the two writers are ordered by the heads each cites, never by wall-clock time

  @REC-11 @P0 @pending
  Scenario: event and payload text is full-text indexed up to the indexing cap, with ranked search
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "main-run"
    And "main-run" contains a 2 MiB tool result with "quokka-early" in its first KiB and "quokka-late" beyond its first 1 MiB
    And "main-run" contains one "user" event naming "wombat" once and another naming it five times
    When the person runs "cairn ingest --all"
    And the agent calls the MCP tool "event_search" with query "quokka-early"
    Then the tool result event is a hit
    And a search for "quokka-late" returns no hit, because indexing stops at the default 1 MiB cap
    When the agent calls the MCP tool "event_search" with query "wombat"
    Then the hits are ranked by score, the "user" event naming "wombat" five times above the one naming it once

  @REC-12 @P0 @I10 @pending
  Scenario: events are appended in transcript order and seq alone defines order
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "main-run"
    And the timestamp of line 11 of "main-run" is earlier than that of line 10
    And line 12 of "main-run" carries no timestamp
    When the person runs "cairn ingest --all"
    Then the events of "main-run" have seq in the order of their transcript lines
    And each event whose line carries a timestamp keeps it as metadata
    And the event of line 12 carries no transcript timestamp
    And a second fresh home that ingests "main-run" under the same writer id derives the same chain head hash
    And "cairn event expand" returns line 10 before line 11

  @REC-13 @P1 @I9 @pending
  Scenario Outline: hook handlers for tool-use, stop and permission hooks ingest incrementally within their hook budget and mark the remainder
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "main-run"
    And "main-run" has 50,000 unread lines, more than one hook budget can ingest
    When the hook "<hook>" runs with the session_id and transcript_path of "main-run"
    Then the hook handler exits 0 within its hook budget
    And the events ingested so far are a prefix of "main-run"
    And an ingest marker records the remainder of "main-run"

    Examples:
      | hook              |
      | PostToolUse       |
      | Stop              |
      | SubagentStop      |
      | PermissionRequest |

  @REC-14 @P1 @I1 @pending
  Scenario: Claude Agent SDK transcripts are ingested from configurable roots
    Given an isolated Cairn home
    And the person's configuration adds a transcript root containing a Claude Agent SDK transcript "sdk-run"
    When the person runs "cairn ingest --all"
    Then the command exits 0
    And every line of "sdk-run" is stored as an event of its run

  @REC-15 @P1 @I1 @I5 @pending
  Scenario: retention policies purge content per room and provenance class, leaving tombstones
    Given an isolated Cairn home
    And a home whose record contains "web" payloads and "user" events that are 40 days old
    And the person's configuration sets "retention_policy.*.web" to 30 days
    When the retention policy is applied
    Then the node purges the content of the "web" events, records each purge naming the policy "retention_policy.*.web", and leaves a tombstone, a structural event with provenance "structural", with reason "retention" in each purged range
    And the "user" events are kept and "cairn verify" exits 0
    And with no retention policy configured, applying the default policy removes nothing

  @REC-16 @P2 @I1 @pending
  Scenario: a Claude Managed Agents event history can be ingested
    Given an isolated Cairn home
    And an exported Claude Managed Agents event history "managed-run"
    When the person runs "cairn ingest --path managed-run"
    Then the command exits 0
    And every event of "managed-run" is stored with its run and a provenance class

  @REC-17 @P0 @I1 @I5 @I10 @pending
  Scenario: event content is reached only through a keyed commitment whose key is erased with it
    Given an isolated Cairn home
    And a home whose record contains 40 events, of which seq 10-14 have since been purged
    When the person runs "cairn verify"
    Then the command exits 0
    And every event carries a commitment to its canonical content under a random per-event key of at least 256 bits, stored with the content and absent from its chained header
    And the hash chain, every seal, every tombstone and every exported structure refer to event content, structural fields included, only through the commitment
    And the commitment keys of seq 10-14 were erased with their content
    And no retained or exported value confirms a guess at the purged content

  @REC-18 @P1 @I6 @I10 @pending
  Scenario: writer seals are verified and events past the newest seal show as unsigned
    Given an isolated Cairn home
    And a writer whose log contains 30 events and is sealed at seq 20 with its seat key
    And a second writer whose seal at seq 10 has one altered byte
    When the person runs "cairn verify"
    Then the command exits 3 and names the second writer's seal at seq 10 as a mismatch
    And the first writer's seal, over its writer id, seq 20 and the chain head at seq 20, verifies against the seat key that made it
    And events 21 to 30 of the first writer are shown and recalled as "unsigned"

  @REC-19 @P1 @I1 @I6 @pending
  Scenario Outline: a witnessed run's MCP server seals each of its run seats' writers within 2 s of any unsealed append and when the run stops, and the open segment closes at every stop and every 30 s
    Given an isolated Cairn home
    And an active run with seats in its personal room and in room "L1", whose hook handlers have appended events to both run seats' writers in "PostToolUse" hooks for 31 s, while its MCP server served calls
    And another writer whose chain ended without a closed segment
    And a paired phone whose device seat's writer this node holds
    And an ingested run whose run seat's writer this node holds
    When the hook "<hook>" runs and appends events
    Then the run's MCP server sealed each of its run seats' writers with that seat's key within 2 s of any unsealed append, covering what the hook handlers had appended
    And it seals those writers again when the run stops, and events after the newest seal are shown as "unsigned"
    And what "cairn ingest" later appends for the run, whether or not its MCP server still runs, goes to a new seat and writer in a run seat's room, which the core seals, naming that run seat, taking over its role and sharing its add, so a kick, leave or bar of either seat ends that one add for both
    And a segment was closed once 30 s had passed though no hook closed one, and the open segment is closed at "<hook>" within the hook budgets
    And "cairn verify" and "cairn status" each report the other writer's chain as ended without a closed segment
    And the paired phone sealed its device seat's writer with that seat's key after each append to it
    And the core sealed every writer but the witnessed run's run seats' and the paired phone's, the ingested run's run seat's writer and this node's device seat's writer included, after each append to it

    Examples:
      | hook         |
      | Stop         |
      | SubagentStop |
      | SessionEnd   |

  @REC-20 @P1 @I1 @pending
  Scenario Outline: worktree checkpoints record the commit, branch and redacted diff at every Stop, SessionEnd, hand-back and branch-head rewrite
    Given an isolated Cairn home
    And a run on a git worktree with a previous worktree checkpoint, a tracked change containing an API key, an untracked file and an ignored file
    When <moment>
    Then the record gains an event with provenance "file" carrying the checked-out commit id, the branch and a payload diff against the previous worktree checkpoint
    And the diff covers the tracked change and the untracked file but not the ignored file, with the API key redacted
    And a worktree checkpoint taken for a rewrite of the branch head keeps the replaced head
    And Cairn obtained the worktree state and git data inside the core, with no socket opened and no program started, within the hook budget, leaving any rest to an ingest marker

    Examples:
      | moment                                        |
      | the hook "Stop" runs                          |
      | the hook "SessionEnd" runs                    |
      | the principal hands control back to the agent |
      | the branch head is rewritten by a rebase      |

  @REC-21 @P1 @I6 @pending
  Scenario: a segment of an unsupported format version is refused, audited and left untouched
    Given an isolated Cairn home
    And a writer log containing a segment of a format version this node supports and a segment of format version 99
    When the person runs "cairn rebuild"
    Then every segment this node wrote carries a format version
    And the supported segment is read
    And the version 99 segment is refused and an audit entry records its unsupported format version
    And the version 99 segment file is byte-identical to before

  @REC-22 @P1 @I1 @I2 @pending
  Scenario: events from transcripts the hook handlers did not watch, or read past an ingest marker, are marked ingested and untrusted
    Given an isolated Cairn home
    And deployment mode "interactive"
    And a transcript "pre-install" under the configured transcript roots, written before Cairn's hook handlers were installed, containing a user turn and lifecycle metadata
    And a transcript "elsewhere" from outside the configured transcript roots
    And a watched transcript "cut-short" whose hook handler left an ingest marker
    When the person runs "cairn ingest --path" on "pre-install", on "elsewhere" and on "cut-short"
    Then every event of "pre-install" carries the "ingested" origin, freshness mark and trust mark with its transcript source and ingest position, shown on every surface, and none is shown as witnessed
    And the "user" event and the "harness_meta" event from "pre-install" have trust "untrusted"
    And this node records "elsewhere" as an ingested run in the principal's personal room, shown as "ingested", and every event of it has trust "untrusted"
    And every event "cairn ingest" read past the ingest marker of "cut-short" carries the "ingested" origin and has trust "untrusted"

  @REC-23 @P2 @I2 @I4 @I6 @pending
  Scenario Outline: room bundles are imported only from local files and fetched git refs, verified, redacted and audited
    Given an isolated Cairn home
    And this node holds the chain of a writer of a foreign room under the principal key "principal-b"
    And a room bundle in a local file that <bundle>, carrying a withheld event and an event with an API key the redaction rules of this node's principal match
    When the person imports the bundle
    Then the import is <expected> and an audit entry records it
    And an accepted bundle had its signature by the exporter's device key, chaining to the bundle's principal key, and its seals and chains verified as received, across the withheld event from its retained header, and the API key redacted with its event's commitment key erased, both verifications recorded
    And accepted events form a foreign room, which this node's principal neither owns nor has a seat in
    And a bundle named by a URL is refused without network access, while one at a git ref already fetched into a local clone is read

    Examples:
      | bundle                                                             | expected |
      | continues that writer chain under "principal-b"                    | accepted |
      | forks that writer chain under "principal-b"                        | refused  |
      | claims a room of this node's principal under "principal-b"         | refused  |
      | claims a writer of this node                                       | refused  |
      | is signed by a device key that does not chain to its principal key | refused  |

  @REC-24 @P1 @I1 @I8 @I10 @pending
  Scenario Outline: a home whose node identity changed mints a new device key and new seat keys, keeps the personal room's id and appends nothing under an old key
    Given an isolated Cairn home
    And a device key and a seat key bound to the node identity read, from outside the home, on the node the home was created on
    And <change>
    When the hook "SessionStart" runs and appends its first event after the start
    Then Cairn mints a new device key and a new seat key before that append, the new seat key starting a new seat and writer that names the old seat
    And the new seat inherits no add, role or appointment of the old seat, and, outside the personal room, joins a room only as any seat does, while a personal-room seat, a device seat's included, is a member from its first event
    And the personal room keeps its room id
    And an audit entry records the node identity change
    And no event is appended under the old device key or the old seat key

    Examples:
      | change                                                       |
      | the home is part of a cloned runner image on another machine |
      | the home is a copied volume mounted on another machine       |
      | the home is recovered from a snapshot on another machine     |
      | the node identity value is unavailable                       |

  @REC-25 @P1 @I1 @I10 @pending
  Scenario: closed segments are merged later, outside every hook budget, so a writer has at most 48 segments a day
    Given an isolated Cairn home
    And a writer that closed 200 sealed segments during one day of activity
    When the deferred merge runs outside every hook budget
    Then the writer has at most 48 stored segments for that day
    And every address, commitment and chain head is unchanged
    And "cairn verify" checks every seal and exits 0
