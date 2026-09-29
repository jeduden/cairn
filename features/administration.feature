Feature: Administration and lifecycle (ADM)

  Scenarios for SRS §5.8, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @ADM-01 @P0 @I7 @pending
  Scenario: the plugin registers the hooks and the MCP server with the static binary
    Given an isolated Cairn home
    When the Claude Code plugin bundle is built
    Then the plugin manifest registers every hook of the hook contract as "cairn hook <event>"
    And the plugin manifest registers the MCP server "cairn mcp"
    And the bundle carries the statically linked cairn binary

  @ADM-02 @P0 @I7 @pending
  Scenario: install shows a diff and asks, uninstall reverses exactly
    Given an isolated Cairn home
    And a Claude Code settings file with unrelated user entries
    When the operator runs "cairn install --scope user" and declines the confirmation
    Then a diff of every configuration change is shown and the settings file is unchanged
    When the operator runs "cairn install --scope user --yes"
    And the operator runs "cairn uninstall"
    Then the settings file is byte-identical to the one before install

  @ADM-03 @P0 @I7 @pending
  Scenario: managed settings are detected and never written
    Given an isolated Cairn home
    And a managed settings file that registers the Cairn hooks
    When the operator runs "cairn install --scope user --yes"
    Then the command exits 1
    And the managed settings file is byte-identical to before
    And the output states that managed settings are in force and names the documented managed install path

  @ADM-04 @P0 @I6 @pending
  Scenario Outline: invalid configuration fails with a precise error
    Given an isolated Cairn home
    And a tenant config.toml containing "<line>"
    When the operator runs "cairn status"
    Then the command exits 2
    And the error names the key "<key>" and the problem "<problem>"

    Examples:
      | line                         | key                     | problem      |
      | recal.max_k = 10             | recal.max_k             | unknown key  |
      | recall.max_k = "ten"         | recall.max_k            | type error   |
      | recall.max_k = 500           | recall.max_k            | out of range |
      | payload_threshold_bytes = -1 | payload_threshold_bytes | out of range |

  @ADM-05 @P0 @I1 @pending
  Scenario: migrations run forward after a verified backup and newer schemas are refused
    Given an isolated Cairn home
    And a project store at the previous schema version
    When the operator runs "cairn migrate"
    Then a verified backup exists from before the migration
    And the store reports the current schema version
    Given a project store whose schema version is newer than this binary supports
    When the operator runs "cairn status"
    Then the command exits 3
    And the store file is unchanged

  @ADM-06 @P0 @I1 @pending
  Scenario: backup and restore round-trip a verified store
    Given an isolated Cairn home
    And a project with a Claude Code transcript "session-a" with a payload above 8192 bytes
    When the operator runs "cairn backup"
    And the store and payloads are deleted
    And the operator runs "cairn restore"
    Then "cairn verify" exits 0
    And every event and payload recalled before the backup is recalled identically

  @ADM-07 @P0 @I1 @I5 @pending
  Scenario Outline: purge removes a scope everywhere and leaves a tombstone
    Given an isolated Cairn home
    And a project with a Claude Code transcript "session-a"
    When the operator runs "cairn purge <scope>"
    Then the command exits 0
    And the purged events are gone from events, the FTS index, projections and the payload store
    And a tombstone event records each purged range with its reason
    And the database has been compacted and "cairn verify" exits 0

    Examples:
      | scope               |
      | --project           |
      | --session session-a |
      | --range 10-20       |
      | --before 2026-01-01 |
      | --provenance web    |

  @ADM-08 @P0 @I10 @pending
  Scenario: rebuild regenerates derived state byte-identically
    Given an isolated Cairn home
    And a project with a Claude Code transcript "session-a" with pins, quarantine and landmarks
    When the operator runs "cairn rebuild"
    Then the command exits 0
    And "cairn verify" confirms every projection is byte-identical to its state before rebuild

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
