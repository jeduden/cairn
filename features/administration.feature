Feature: Administration and lifecycle (ADM)

  Scenarios for SRS §5.8, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @ADM-01 @P0 @I7 @pending
  Scenario: the plugin registers the hooks and the MCP server with the core executable
    Given an isolated Cairn home
    When the Claude Code plugin bundle is built
    Then the plugin manifest registers every hook of the hook contract as "cairn hook <event>"
    And the plugin manifest registers the MCP server "cairn mcp"
    And the bundle carries the cairn core executable for each supported platform

  @ADM-02 @P0 @I7 @I6 @pending
  Scenario: install asks before every change and uninstall lists, offers and audits every artifact
    Given an isolated Cairn home
    And a Claude Code settings file with unrelated user entries
    And "cairn install --scope user" showed a diff of every configuration change, was declined and left the settings file unchanged
    And "cairn install --scope user --yes" has run and every Cairn component has created its artifacts
    When the operator runs "cairn uninstall" and keeps only the device key
    Then the output lists the hooks, plugin and MCP registration, lane-view credentials, run-component endpoints, writer and device keys, enrolments, git-carrier refs and managed state Cairn wrote, each with an offer to remove it
    And the settings file is byte-identical to the one before install
    And an audit entry names the device key as left in place

  @ADM-03 @P0 @I7 @pending
  Scenario: managed settings are detected and never written
    Given an isolated Cairn home
    And a managed settings file that registers the Cairn hooks
    When the operator runs "cairn install --scope user --yes"
    Then the command exits 1
    And the managed settings file is byte-identical to before
    And the output states that managed settings are in force and names the documented managed install path

  @ADM-04 @P0 @I6 @I7 @pending
  Scenario Outline: configuration is validated strictly, managed policy overrides every layer, and a widening change waits for a recorded act
    Given an isolated Cairn home
    And <configuration>
    When a session runs and the operator starts "<component>"
    Then "<component>" <result>
    And the core records every event of the session

    Examples:
      | configuration                                                                                   | component               | result                                                                                        |
      | a managed policy file at the documented system path that the tenant can write                   | the lane-view component | refuses to start, and an audit entry and a counter record why                                 |
      | a managed policy file in a directory the tenant can write                                       | the run component       | refuses to start, and an audit entry and a counter record why                                 |
      | an unparsable managed policy file                                                               | the lane-view component | refuses to start, and an audit entry and a counter record why                                 |
      | a managed policy file with an unknown key                                                       | the run component       | refuses to start, and an audit entry and a counter record why                                 |
      | a project ".cairn.toml" that turns on the lane view with no widening act recording its digest   | the lane-view component | stays off                                                                                     |
      | a project ".cairn.toml" that turns on the lane view and a widening act that recorded its digest | the lane-view component | starts                                                                                        |
      | a managed policy that turns on the lane view and a tenant config that turns it off              | the lane-view component | starts                                                                                        |
      | a tenant config.toml containing "recal.max_k = 10"                                              | cairn status            | exits 2, and the error names the key "recal.max_k" and the problem "unknown key"              |
      | a tenant config.toml containing "recall.max_k = 'ten'"                                          | cairn status            | exits 2, and the error names the key "recall.max_k" and the problem "type error"              |
      | a tenant config.toml containing "recall.max_k = 500"                                            | cairn status            | exits 2, and the error names the key "recall.max_k" and the problem "out of range"            |
      | a tenant config.toml containing "payload_threshold_bytes = -1"                                  | cairn status            | exits 2, and the error names the key "payload_threshold_bytes" and the problem "out of range" |

  @ADM-05 @P0 @I1 @pending
  Scenario: segment and schema migrations run forward after a verified backup and newer versions are refused
    Given an isolated Cairn home
    And a project whose segments and derived state are at the previous version
    And a segment whose format version is newer than this node supports
    When the operator runs "cairn migrate"
    Then a verified backup with its audit chain exists from before the migration
    And the segments and derived state report the current version
    And the newer segment is refused, named in the output and left unchanged
    Given derived state whose schema version is newer than this node supports
    When the operator runs "cairn status"
    Then the command exits 3
    And the derived state is unchanged

  @ADM-06 @P0 @I1 @I6 @pending
  Scenario: backup and restore keep every later removal and never reuse a writer's log
    Given an isolated Cairn home
    And a project with sealed segments, an open segment, payloads, derived state and an audit log
    And a backup taken by "cairn backup", followed by a purge of session "session-a" and the end of a pin
    When the owner runs "cairn restore" as a widening act
    Then the backup held every segment sealed fresh, the payload store, derived state and the audit log with its chain, no writer or device key, and an audit entry recorded it
    And "cairn verify" passed on the copy and its audit chain before anything was restored
    And every event and payload outside session "session-a" recalled before the backup is recalled identically
    And session "session-a" stays purged and the ended pin stays ended
    And each restored local writer is followed by a new audited writer, and no restored writer's log gains an event or reuses a seq
    And "cairn verify" exits 0 on the restored home

  @ADM-07 @P0 @I1 @I5 @pending
  Scenario Outline: purge removes a scope everywhere, erases its commitment keys and leaves a tombstone
    Given an isolated Cairn home
    And a project with a Claude Code transcript "session-a" in lane "lane-a"
    When the operator runs "cairn purge <scope>"
    Then the command exits 0
    And the purged events are gone from the sealed segments, events, the FTS index, projections, the payload store and every copy of their content
    And the commitment key and payload reference of every purged event are erased
    And a tombstone event per purged range holds only addresses, counts, reason and commitments
    And the database is compacted and "cairn verify" confirms every rewritten segment's seals

    Examples:
      | scope                  |
      | --project              |
      | --lane lane-a          |
      | --session session-a    |
      | --writer writer-a      |
      | --actor alice          |
      | --range writer-a:10-20 |
      | --before 2026-01-01    |
      | --provenance web       |

  @ADM-08 @P0 @I10 @pending
  Scenario: rebuild regenerates derived state byte-identically whatever order the logs arrived in
    Given an isolated Cairn home
    And a second isolated Cairn home with the same node keys, the two holding the same writer logs with pins, quarantine and landmarks, received in opposite orders
    When the operator runs "cairn rebuild" in each home
    Then the command exits 0 in each home
    And "cairn verify" confirms every projection is byte-identical to its state before rebuild
    And the derived state of the two homes is byte-identical

  @ADM-09 @P0 @I6 @I10 @pending
  Scenario Outline: verify exits non-zero on any integrity failure
    Given an isolated Cairn home
    And a project with a Claude Code transcript "session-a"
    And <damage>
    When the operator runs "cairn verify"
    Then the command exits 3
    And the output names the failed check "<check>"

    Examples:
      | damage                                     | check                  |
      | the transcript has lines not in the record | source completeness    |
      | an event row's hash has been altered       | hash chain             |
      | a payload file's bytes have been altered   | payload integrity      |
      | a landmark row has been altered            | projection consistency |

  @ADM-10 @P0 @I7 @pending
  Scenario: doctor is read-only and doctor --fix shows each change first
    Given an isolated Cairn home
    And a Cairn home whose store file has mode 0644
    When the operator runs "cairn doctor"
    Then the store file still has mode 0644
    When the operator runs "cairn doctor --fix"
    Then the change "chmod 0600" is shown before it is applied
    And the store file has mode 0600

  @ADM-11 @P0 @I6 @pending
  Scenario: status shows locations, sizes, schema, mode and failure counters
    Given an isolated Cairn home
    And deployment mode "automation"
    And a project with a Claude Code transcript "session-a"
    And the counter "hook_timeout" is 2
    When the operator runs "cairn status --json"
    Then the command exits 0
    And the output shows the store location, its size, the schema version and mode "automation"
    And the output shows the counter "hook_timeout" with value 2

  @ADM-12 @P1 @I2 @pending
  Scenario: export writes only trusted events with provenance as JSONL
    Given an isolated Cairn home
    And a project with a Claude Code transcript "session-a" with user prompts and web tool results
    When the operator runs "cairn export --trusted-only"
    Then the command exits 0
    And every exported JSONL line is a trusted event carrying its seq, provenance and trust
    And no untrusted event appears in the export

  @ADM-13 @P0 @I7 @pending
  Scenario: Cairn writes to no git repository beyond the confirmed settings file and the carrier's location
    Given an isolated Cairn home
    And a project that is a git repository with a working tree, refs, notes, configuration and hooks
    And the owner has enabled the git carrier
    When a session runs, the operator confirms "cairn install --scope project" and every Cairn component does its work
    Then the only changed file in the working tree is the project settings file
    And the only new or changed refs lie in the namespaced location the owner enabled for the carrier, and every new object is reachable only from them
    And the repository's other refs, notes, configuration and hooks are byte-identical to before

  @ADM-14 @P1 @I1 @I5 @pending
  Scenario Outline: purge by writer key or actor removes one person's data with an audit trail on every node
    Given an isolated Cairn home
    And two nodes that both hold the events of the human principal "alice" written under the writer key "alice-key"
    When the operator runs "cairn purge <scope>" on each node
    Then neither node holds an event of the selected scope
    And each node's audit log records the purge with its scope and ranges

    Examples:
      | scope              |
      | --writer alice-key |
      | --actor alice      |

  @ADM-15 @P1 @I6 @I9 @I1 @pending
  Scenario Outline: disk quotas refuse and audit what exceeds them and never drop an accepted event
    Given an isolated Cairn home
    And managed policy sets every quota
    And the <quota> quota is reached
    When <arrival>
    Then <outcome>
    And every event accepted before is still stored and recallable

    Examples:
      | quota               | arrival                                    | outcome                                                                     |
      | imported writer     | a peer offers another event of that writer | the event is refused and an audit entry records it                          |
      | peer                | the peer offers another segment            | the segment is refused and an audit entry records it                        |
      | worktree checkpoint | the hook "Stop" records another checkpoint | the checkpoint is refused and an audit entry records it                     |
      | local writer        | the session appends another event          | the event is recorded, a failure counter rises and a Needs you item appears |

  @ADM-16 @P1 @I6 @pending
  Scenario Outline: status and doctor report every user-run component, peer lag, open chains and boundaries
    Given an isolated Cairn home
    And managed policy that permits the lane-view and peer components, forbids the run component and locks one boundary
    And the lane-view component is running, a bridge has failed twice, a peer lags behind one writer and a writer chain ended without a closed segment
    When the operator runs "cairn <command>"
    Then for the lane-view, run and peer components and the bridge the output shows whether policy permits it, whether it runs and its failure counters
    And the output shows the peer's sync lag for each writer
    And the output names the writer chain that ended without a closed segment
    And the output shows each boundary's state and whether managed policy locks it

    Examples:
      | command |
      | status  |
      | doctor  |
