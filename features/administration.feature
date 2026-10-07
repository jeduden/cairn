Feature: Administration and lifecycle (ADM)

  Scenarios for SRS §5.8, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @ADM-01 @P0 @I7 @pending
  Scenario: the plugin registers the hooks and the MCP server with the core executable
    Given an isolated Cairn home
    When the Claude Code plugin bundle is built
    Then the plugin manifest registers every hook of the hook contract as "cairn hook <hook>"
    And the plugin manifest registers the MCP server "cairn mcp"
    And the bundle carries the cairn core executable for each supported platform

  @ADM-02 @P0 @I7 @I6 @pending
  Scenario: install asks before every change, and uninstall needs a terminal, records turning capture off before it removes the hook registrations, and lists, offers and audits every artifact
    Given an isolated Cairn home
    And a Claude Code settings file with unrelated user entries
    And "cairn install --scope user" showed a diff of every harness configuration change, was declined and left the settings file unchanged
    And "cairn install --scope user --yes" has run and every Cairn component has created its artifacts
    And that install created the node's personal room as the first act of the principal's device seat, that seat's add there
    And "cairn uninstall" run without a terminal refused and changed nothing
    When the person runs "cairn uninstall" at a terminal and keeps only the device key
    Then before it removed the hook registrations, it recorded the widening principal act turning capture off
    And the output lists the hook, plugin and MCP registrations, room-view credentials, launcher endpoints, seat and device keys, enrollments, git-carrier refs and every other file Cairn wrote outside the store, each with an offer to remove it, and offers purge as the only removal of the store's content
    And the settings file is byte-identical to the one before install
    And an audit entry names the device key as left in place

  @ADM-03 @P0 @I7 @pending
  Scenario: the harness's managed settings are detected and never written
    Given an isolated Cairn home
    And a file of the harness's managed settings that registers Cairn's hook handlers
    When the person runs "cairn install --scope user --yes"
    Then the command exits 1
    And that file of the harness's managed settings is byte-identical to before
    And the output states that the harness's managed settings are in force and names the documented managed install path

  @ADM-04 @P0 @I6 @I7 @pending
  Scenario Outline: every settings layer is validated strictly, managed policy overrides the others but never turns a component on, repository configuration only tightens, and a settings change that needs a widening principal act waits for one
    Given an isolated Cairn home
    And <settings>
    When an agent runs and the person starts "<component>"
    Then "<component>" <expected>
    And the core records every event of the run

    Examples:
      | settings                                                                                                                                            | component               | expected                                                                                      |
      | a managed policy file at the documented system path that the person can write                                                                       | the room-view component | refuses to start, and an audit entry and a counter record why                                 |
      | a managed policy file in a directory the person can write                                                                                           | the launcher            | refuses to start, and an audit entry and a counter record why                                 |
      | an unparsable managed policy file                                                                                                                   | the room-view component | refuses to start, and an audit entry and a counter record why                                 |
      | a managed policy file with an unknown key                                                                                                           | the launcher            | refuses to start, and an audit entry and a counter record why                                 |
      | a repository ".cairn.toml" that turns on the room-view component and a widening act that recorded its digest                                        | the room-view component | stays off, and an audit entry and a counter record the ignored key                            |
      | the person's config.toml that turns on the room-view component with no widening act recording its digest                                            | the room-view component | stays off                                                                                     |
      | the person's config.toml that turns on the room-view component and a widening act that recorded its digest                                          | the room-view component | starts                                                                                        |
      | a managed policy that turns off the room-view component and the person's config.toml that turns it on, with a widening act that recorded its digest | the room-view component | stays off                                                                                     |
      | the person's config.toml containing "recal.max_k = 10"                                                                                              | cairn status            | exits 2, and the error names the key "recal.max_k" and the problem "unknown key"              |
      | the person's config.toml containing "recall.max_k = 'ten'"                                                                                          | cairn status            | exits 2, and the error names the key "recall.max_k" and the problem "type error"              |
      | the person's config.toml containing "recall.max_k = 500"                                                                                            | cairn status            | exits 2, and the error names the key "recall.max_k" and the problem "out of range"            |
      | the person's config.toml containing "payload.threshold_bytes = -1"                                                                                  | cairn status            | exits 2, and the error names the key "payload.threshold_bytes" and the problem "out of range" |

  @ADM-05 @P0 @I1 @pending
  Scenario: segment and schema migrations run forward after a verified backup and newer versions are refused
    Given an isolated Cairn home
    And a home whose segments and derived artifacts are at the previous version
    And a segment whose format version is newer than this node supports
    When the person runs "cairn migrate"
    Then a verified backup with its audit chain exists from before the migration
    And the segments and derived artifacts report the current version
    And the newer segment is refused, named in the output and left unchanged
    Given derived artifacts whose schema version is newer than this node supports
    When the person runs "cairn status"
    Then the command exits 3
    And the derived artifacts are unchanged

  @ADM-06 @P0 @I1 @I6 @pending
  Scenario: a backup restore keeps every later removal and never reuses a writer's log
    Given an isolated Cairn home
    And a home with sealed segments, an open segment, payloads, derived artifacts and an audit log
    And a backup taken by "cairn backup create", followed by a purge of run "run-a" and the unpin of a pin
    When the person runs "cairn backup restore" as a widening principal act
    Then the backup contained every segment, the open segment up to a fresh seal, the payload store, derived artifacts and the audit log with its chain, no seat, device, token or at-rest key, and an audit entry recorded it
    And "cairn verify" passed on the copy and its audit chain before anything was reinstated
    And every event and payload outside run "run-a" recalled before the backup is recalled identically
    And run "run-a" stays purged and the unpinned pin stays unpinned
    And each local seat the copy contains is followed by a new seat and writer under a newly minted seat key, which names the old seat and inherits no add, role or appointment of it, audited, and no reinstated writer's log gains an event or reuses a seq
    And "cairn verify" exits 0 on the home after the backup restore

  @ADM-07 @P0 @I1 @I5 @pending
  Scenario Outline: purge removes a scope everywhere, erases its commitment keys and leaves a tombstone
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "run-a" in room "room-a"
    When the person runs "cairn purge <scope>"
    Then the command exits 0
    And the purged events are gone from the sealed segments, events, the search index, derived artifacts, the payload store and every copy of their content
    And the commitment key and payload reference of every purged event are erased
    And a tombstone event per purged range carries only addresses, counts, reason and commitments
    And the store is vacuumed and "cairn verify" confirms every rewritten segment's seals

    Examples:
      | scope                  |
      | --room room-a          |
      | --run run-a            |
      | --writer writer-a      |
      | --author s-1           |
      | --range writer-a:10-20 |
      | --before 2026-01-01    |
      | --provenance web       |

  @ADM-08 @P0 @I10 @pending
  Scenario: rebuild regenerates derived artifacts byte-identically whatever order the logs arrived in
    Given an isolated Cairn home
    And a second isolated Cairn home with the same key set, the two nodes holding the same writer logs with pins, quarantine and landmarks, received in opposite orders
    When the person runs "cairn rebuild" in each home
    Then the command exits 0 in each home
    And "cairn verify" confirms every derived artifact is byte-identical to its state before rebuild
    And the derived artifacts of the two homes are byte-identical

  @ADM-09 @P0 @I6 @I10 @pending
  Scenario Outline: verify exits non-zero on any integrity failure
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "run-a"
    And <damage>
    When the person runs "cairn verify"
    Then the command exits 3
    And the output names the failed check "<check>"

    Examples:
      | damage                                     | check                          |
      | the transcript has lines not in the record | transcript source completeness |
      | an event row's hash has been altered       | hash chain                     |
      | a payload file's bytes have been altered   | payload integrity              |
      | a landmark row has been altered            | derived artifact consistency   |

  @ADM-10 @P0 @I7 @pending
  Scenario: doctor is read-only and doctor --fix shows each change first
    Given an isolated Cairn home
    And a Cairn home whose store file has mode 0644
    When the person runs "cairn doctor"
    Then the store file still has mode 0644
    When the person runs "cairn doctor --fix"
    Then the change "chmod 0600" is shown before it is applied
    And the store file has mode 0600

  @ADM-11 @P0 @I6 @pending
  Scenario: status shows locations, sizes, schema, deployment mode and failure counters
    Given an isolated Cairn home
    And deployment mode "automation"
    And an agent run with a Claude Code transcript "run-a"
    And the counter "hook_budget_exceeded" is 2
    When the person runs "cairn status --json"
    Then the command exits 0
    And the output shows the store location, its size, the schema version and the deployment mode "automation"
    And the output shows the counter "hook_budget_exceeded" with value 2

  @ADM-12 @P1 @I2 @pending
  Scenario: export writes only trusted events with provenance as JSONL
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "run-a" with user turns and web tool results
    When the person runs "cairn export --trusted-only"
    Then the command exits 0
    And every exported JSONL line is a trusted event carrying its address (writer, seq), provenance and trust
    And no untrusted event appears in the export

  @ADM-13 @P0 @I7 @pending
  Scenario: Cairn writes to no git repository beyond the confirmed settings file, the commit hook of LANE-28, the git carrier's location and the launcher's fresh checkouts
    Given an isolated Cairn home
    And a git repository with a worktree, refs, notes, git configuration and git hooks
    And the node's principal has enabled the git carrier for the repository's remote, and the room's owner for the room
    When an agent runs, the person confirms "cairn install --scope project", every Cairn component runs and the launcher runs a witness check
    Then the only changed file in the worktree is the harness settings file that install wrote
    And the only new or changed refs lie in the namespaced location the node's principal enabled for the git carrier, and every new object is reachable only from them
    And the only changed git hook is the commit hook for room trailers that the confirmed install set up
    And the witness check's fresh checkout lies outside the run's worktree and added no ref to the run's repository
    And the repository's other refs, notes, git configuration and git hooks are byte-identical to before

  @ADM-14 @P1 @I1 @I5 @pending
  Scenario Outline: purge by seat or principal removes one principal's data with an audit trail on every node
    Given an isolated Cairn home
    And two nodes that both hold the events that the seat "s-alice" of the principal "alice" wrote
    When the person runs "cairn purge <scope>" on each node
    Then neither node holds an event of the selected scope
    And each node's audit log records the purge with its scope and ranges

    Examples:
      | scope             |
      | --seat s-alice    |
      | --principal alice |

  @ADM-15 @P1 @I6 @I9 @I1 @pending
  Scenario Outline: quotas refuse and audit what is received over them and never drop an accepted event
    Given an isolated Cairn home
    And managed policy sets every quota
    And the <quota> quota is reached
    When <arrival>
    Then <expected>
    And every event accepted before is still stored and recallable

    Examples:
      | quota               | arrival                                             | expected                                                                                  |
      | received writer     | a peer offers another event of that writer          | the event is refused and an audit entry records it                                        |
      | peer                | the peer offers another segment                     | the segment is refused and an audit entry records it                                      |
      | worktree checkpoint | the hook "Stop" records another worktree checkpoint | the worktree checkpoint is recorded, a failure counter rises and a Needs you item appears |
      | node                | the run appends another event                       | the event is recorded, a failure counter rises and a Needs you item appears               |

  @ADM-16 @P1 @I6 @pending
  Scenario Outline: status and doctor report the launcher and every other component outside the core, peer lag, open chains and boundaries
    Given an isolated Cairn home
    And managed policy that permits the room-view, peer and publish components, forbids the launcher and locks one boundary
    And the room-view component is running, the bridge component has failed twice, a peer lags behind one writer and a writer chain ended without a closed segment
    When the person runs "cairn <command>"
    Then for the room-view component, the launcher, the peer component, the publish component and the bridge component the output shows whether managed policy permits it, whether it runs and its failure counters
    And the output shows the peer's sync lag for each writer
    And the output names the writer chain that ended without a closed segment
    And the output shows each boundary's state and whether managed policy locks it

    Examples:
      | command |
      | status  |
      | doctor  |
