Feature: Recall (RCL)

  Scenarios for SRS §5.4, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @RCL-01 @P0 @I1 @pending
  Scenario: the mcp server lists exactly the six recall tools
    Given an isolated Cairn home
    And a project with a Claude Code transcript "short-session"
    When Claude lists the tools of the MCP server started by "cairn mcp"
    Then the tool list is "search", "expand", "get", "landmarks", "pins_list" and "stats"
    And each tool declares the parameters specified in SRS section 9.2

  @RCL-02 @P0 @pending
  Scenario Outline: search honours its filters and caps hits at k
    Given an isolated Cairn home
    And a project with a Claude Code transcript "mixed-provenance"
    When Claude calls the MCP tool "search" with <args>
    Then the result is wrapped in the recall envelope
    And the envelope holds at most <hits> items, each matching the filter, ranked by BM25 score
    And a relevant hit from a short session still ranks above the repeated hits of a long session

    Examples:
      | args                                 | hits |
      | query "deploy"                       | 10   |
      | query "deploy", k 50                 | 50   |
      | query "deploy", k 500                | 50   |
      | query "deploy", session "all"        | 10   |
      | query "deploy", provenance ["web"]   | 10   |
      | query "deploy", kind ["tool_result"] | 10   |
      | query "deploy", trust "trusted"      | 10   |
      | query "deploy", since "2026-01-01"   | 10   |
      | query "deploy", until "2026-06-30"   | 10   |

  @RCL-03 @P0 @I1 @pending
  Scenario: expand returns exact post-redaction content under the token cap
    Given an isolated Cairn home
    And a project with a Claude Code transcript "large-payloads"
    When Claude calls the MCP tool "expand" with seq_from 1, seq_to 400
    Then the result is wrapped in the recall envelope
    And the items hold the exact post-redaction content with payload references resolved
    And the envelope is at most 8,000 tokens with "truncated" true and a "next_cursor"
    And calling "expand" with that cursor returns the following events without gap or overlap

  @RCL-04 @P0 @I2 @pending
  Scenario Outline: every recall tool wraps its content in the envelope
    Given an isolated Cairn home
    And a project with a Claude Code transcript "mixed-provenance"
    When Claude calls the MCP tool "<tool>" with <args>
    Then the result is wrapped in the recall envelope
    And the envelope carries "cairn_envelope" 1 and the fixed untrusted-data notice

    Examples:
      | tool   | args                  |
      | search | query "deploy"        |
      | expand | seq_from 1, seq_to 20 |
      | get    | seq 7                 |

  @RCL-05 @P0 @I8 @pending
  Scenario: recall stays in the current project and widening is logged
    Given an isolated Cairn home
    And a project with a Claude Code transcript "project-a"
    And another project with a Claude Code transcript "project-b"
    When Claude calls the MCP tool "search" with query "deploy" in project "project-a"
    Then every hit belongs to the current session of "project-a"
    And no parameter accepts a hit from "project-b"
    And calling "search" with session "all" returns hits from every session of "project-a" and an audit entry records "recall scope widened to all sessions"

  @RCL-06 @P0 @I5 @pending
  Scenario: quarantined events are never recalled and purged ranges return a tombstone
    Given an isolated Cairn home
    And a project with a Claude Code transcript "poisoned-web"
    And the operator runs "cairn quarantine add --seq 12"
    And the operator runs "cairn purge --range 30-40"
    When Claude calls the MCP tool "expand" with seq_from 1, seq_to 50
    Then the result is wrapped in the recall envelope
    And no item has seq 12 and no item lies in seq 30-40, which appear as a tombstone with reason "purged"

  @RCL-07 @P0 @I6 @pending
  Scenario: every recall call is appended to the record
    Given an isolated Cairn home
    And a project with a Claude Code transcript "short-session"
    When Claude calls the MCP tool "search" with query "deploy"
    Then the record gains one recall event with provenance "assistant" carrying the query
    And that event lists the returned seq set
    And the counter "recall_calls" increases by 1
