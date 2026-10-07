Feature: Observability (OPS)

  Scenarios for SRS §5.10, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @OPS-01 @P0 @I6 @pending
  Scenario Outline: every degraded operation is audited and counted
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "run-a"
    When <operation>
    Then an audit entry records "<outcome>"
    And the counter "<counter>" increases by 1

    Examples:
      | operation                                                                         | outcome   | counter          |
      | the hook "PostToolUse" runs with a malformed JSON object                          | rejected  | hook_rejected    |
      | the transcript contains an AWS secret key and is ingested                         | redacted  | redactions       |
      | the hook "SessionStart" runs past its 150 ms hook budget                          | timed-out | hook_timeout     |
      | the agent calls the MCP tool "event_expand" with a range above 8,000 model tokens | truncated | recall_truncated |
      | the transcript contains an unparseable line and is ingested                       | dropped   | ingest_dropped   |
      | the hook "PreCompact" runs with an unreadable transcript path                     | failed    | hook_failed      |

  @OPS-02 @P0 @I6 @pending
  Scenario: the audit log is an append-only hash-chained JSONL file that rotates with chain continuation
    Given an isolated Cairn home
    And an audit log that has reached its rotation size
    When the person runs "cairn counter ack"
    Then the file "audit/audit-000002.jsonl" is created with mode 0600
    And its first entry's previous hash equals the hash of the last entry in "audit/audit-000001.jsonl"
    And every entry is RFC 8785 canonical JSON whose SHA-256 chain verifies with "cairn verify"

  @OPS-03 @P0 @I6 @pending
  Scenario: doctor fails while a failure counter has risen since the counters were last acknowledged
    Given an isolated Cairn home
    And the person has acknowledged the counters with "cairn counter ack", a neutral principal act
    When the hook "PostToolUse" runs with a malformed JSON object
    And the person runs "cairn doctor"
    Then the command exits 1
    When the person runs "cairn counter ack"
    And the person runs "cairn doctor"
    Then the command exits 0

  @OPS-04 @P1 @I6 @pending
  Scenario: the canary writes through the hook path and recalls through MCP
    Given an isolated Cairn home
    And deployment mode "automation"
    When the person runs "cairn canary"
    Then the command exits 0
    And the canary event was written through a hook handler and recalled through the MCP server's tool "event_search"
    Given the MCP recall path is broken
    When the person runs "cairn canary"
    Then the command exits 1

  @OPS-05 @P1 @I4 @pending
  Scenario: structured JSON logs go to a local file or stderr and nowhere else
    Given an isolated Cairn home
    And logging is configured to the file "logs/cairn.log"
    When the person runs "cairn ingest --all"
    Then every line of "logs/cairn.log" is a JSON object with "time", "level" and "msg"
    And the process opened no network connection

  @OPS-06 @P1 @I6 @pending
  Scenario Outline: every failure of a component outside the core reaches the home's audit log and a named counter
    Given an isolated Cairn home
    And "<component>" is running
    When an operation of "<component>" is <outcome>
    Then the home's audit log records the operation as "<outcome>"
    And a named counter for it increases by 1
    And "cairn status" shows the failure outside any browser

    Examples:
      | component           | outcome   |
      | room-view component | rejected  |
      | room-view component | coalesced |
      | launcher            | timed-out |
      | peer component      | dropped   |
      | publish component   | failed    |
      | bridge component    | failed    |
