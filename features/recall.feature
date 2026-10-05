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
      | query "deploy", scope "project"      | 10   |
      | query "deploy", provenance ["web"]   | 10   |
      | query "deploy", kind ["tool_result"] | 10   |
      | query "deploy", trust "trusted"      | 10   |
      | query "deploy", since "2026-01-01"   | 10   |
      | query "deploy", until "2026-06-30"   | 10   |

  @RCL-03 @P0 @I1 @pending
  Scenario: expand returns exact post-redaction content under the token cap
    Given an isolated Cairn home
    And a project with a Claude Code transcript "large-payloads"
    When Claude calls the MCP tool "expand" with seq_from "w-1:1", seq_to "w-1:400"
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
      | tool   | args                              |
      | search | query "deploy"                    |
      | expand | seq_from "w-1:1", seq_to "w-1:20" |
      | get    | seq "w-1:7"                       |

  @RCL-05 @P0 @I8 @pending
  Scenario: recall defaults to the current session and widening to lane or project is explicit and logged
    Given an isolated Cairn home
    And a project whose current lane has sessions on two worktrees and two writers this node holds, beside another lane of the project, a foreign lane and another project
    When Claude calls the MCP tool "search" with query "deploy" and no scope
    Then every hit belongs to the current session
    And with scope "lane" the hits come from every session of the current lane, and with scope "project" from every lane of the project, and an audit entry logs each widening
    And no scope returns a hit from the foreign lane or from another project
    When Claude calls the MCP tool "get" with seq "w-2:5", an event of another session of the current lane, and no scope
    Then the event is not returned, and the result says the address lies outside the current scope
    And with scope "lane" the event is returned, and an audit entry logs the widening

  @RCL-06 @P0 @I5 @pending
  Scenario: quarantined events are never recalled and purged ranges return a tombstone
    Given an isolated Cairn home
    And a project with a Claude Code transcript "poisoned-web"
    And the operator runs "cairn quarantine add --range w-1:12-12"
    And the operator runs "cairn purge --range w-1:30-40"
    When Claude calls the MCP tool "expand" with seq_from "w-1:1", seq_to "w-1:50"
    Then the result is wrapped in the recall envelope
    And no item has address w-1·12 and no item lies in w-1·30–40, which appears as a tombstone with reason "purged"

  @RCL-07 @P0 @I6 @pending
  Scenario: every recall call is appended to the record
    Given an isolated Cairn home
    And a project with a Claude Code transcript "short-session"
    When Claude calls the MCP tool "search" with query "deploy"
    Then the record gains one recall event with provenance "assistant" carrying the query
    And that event lists the addresses of the returned events
    And the counter "recall_calls" increases by 1

  @RCL-08 @P1 @I1 @pending
  Scenario Outline: every address Cairn shows, and its ASCII input form, resolves wherever an address is taken
    Given an isolated Cairn home
    And a project with a Claude Code transcript "short-session"
    And an event address shown to a person or an agent in <form> form
    When Claude calls the MCP tool "<tool>" with that address
    Then the result holds exactly the events the address names
    And an address of a purged or quarantined event resolves to its tombstone or quarantine notice
    And "cairn expand" given the same address in the same form returns the same events

    Examples:
      | form                                 | tool   |
      | short                                | get    |
      | range                                | expand |
      | full                                 | get    |
      | ASCII input, such as "w-1:7"         | get    |
      | ASCII input range, such as "w-1:3-9" | expand |

  @RCL-09 @P1 @I2 @I6 @pending
  Scenario Outline: every recalled item carries its writer, actor, trust, origin and chain status
    Given an isolated Cairn home
    And a project whose record holds <item>
    When Claude recalls that item with the MCP tool "get"
    Then the item carries its writer, its actor and its trust level
    And the item carries origin "<origin>" and chain status "<status>"

    Examples:
      | item                                               | origin    | status     |
      | a sealed event this node witnessed                 | witnessed | verified   |
      | an event past its writer's newest seal             | witnessed | unsigned   |
      | a sealed event imported from a transcript          | imported  | verified   |
      | a sealed event of a foreign lane                   | foreign   | verified   |
      | a peer's event after a break in its writer's chain | peer      | unverified |
      | a peer's event whose chain check fails             | peer      | broken     |

  @RCL-10 @P2 @I2 @I8 @pending
  Scenario: a foreign lane is recalled only by naming it in the call, enveloped, untrusted and tainting
    Given an isolated Cairn home
    And a project holding a foreign lane "vendor-lane" imported from a lane bundle
    When Claude calls the MCP tool "search" with query "deploy" and lane "vendor-lane"
    Then the hits come from "vendor-lane", wrapped in the recall envelope, each with trust "untrusted"
    And an audit entry logs the call and the session is tainted under SEC-13
    And a following call without the lane parameter, under any scope, returns no hit from "vendor-lane"

  @RCL-11 @P2 @I6 @pending
  Scenario: recalling another participant's post is recorded and shown in the lane timeline
    Given an isolated Cairn home
    And a lane owned by "owner-a" holding a post written by participant "bob"
    When Claude on this node calls the MCP tool "get" with the post's address
    Then this node's writer log gains the recall event, listing the post's address among the returned events
    And the lane timeline shows the recall with its recall address to "owner-a" and to "bob"
