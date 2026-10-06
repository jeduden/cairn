Feature: Recall (RCL)

  Scenarios for SRS §5.4, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @RCL-01 @P0 @I1 @pending
  Scenario: the mcp server lists exactly the six recall tools
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "short-run"
    When Claude lists the tools of the MCP server started by "cairn mcp"
    Then the tool list is "event_search", "event_expand", "event_get", "landmark_list", "pin_list" and "stat_list"
    And each tool declares the parameters specified in SRS section 9.2

  @RCL-02 @P0 @pending
  Scenario Outline: event search honours its filters and caps hits at k
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "mixed-provenance"
    When Claude calls the MCP tool "event_search" with <args>
    Then the result is wrapped in the recall envelope
    And the envelope contains at most <hits> items, each matching the filter, ranked by BM25 score
    And a relevant hit from a short run still ranks above the repeated hits of a long run

    Examples:
      | args                                 | hits |
      | query "deploy"                       | 10   |
      | query "deploy", k 50                 | 50   |
      | query "deploy", k 500                | 50   |
      | query "deploy", scope "rooms"        | 10   |
      | query "deploy", provenance ["web"]   | 10   |
      | query "deploy", kind ["tool_result"] | 10   |
      | query "deploy", trust "trusted"      | 10   |
      | query "deploy", since "2026-01-01"   | 10   |
      | query "deploy", until "2026-06-30"   | 10   |

  @RCL-03 @P0 @I1 @pending
  Scenario: event expand returns exact post-redaction content under the token cap
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "large-payloads"
    When Claude calls the MCP tool "event_expand" with range "w-1:1-400"
    Then the result is wrapped in the recall envelope
    And the items carry the exact post-redaction content with payload references resolved
    And the envelope is at most 8,000 tokens with "truncated" true and a "next_cursor"
    And calling "event_expand" with that cursor returns the following events without gap or overlap

  @RCL-04 @P0 @I2 @pending
  Scenario Outline: every recall tool wraps its content in the envelope
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "mixed-provenance"
    When Claude calls the MCP tool "<tool>" with <args>
    Then the result is wrapped in the recall envelope
    And the envelope carries "cairn_envelope" 1 and the fixed envelope warning in its field "warning"

    Examples:
      | tool         | args                           |
      | event_search | query "deploy"                 |
      | event_expand | range from "w-1:1" to "w-1:20" |
      | event_get    | address "w-1:7"                |

  @RCL-05 @P0 @I8 @pending
  Scenario: recall defaults to the agent's current run and widening to its rooms is explicit and logged
    Given an isolated Cairn home
    And a run with seats in its personal room and in room "L1", whose writers this node holds beside those of other runs in "L1" on two worktrees, of room "L2" where the run has no seat, and of a foreign room
    When Claude calls the MCP tool "event_search" with query "deploy" and no scope
    Then every hit belongs to the calling run, across the writers of both its seats
    And with scope "room" and room "L1" the hits come from every writer of "L1", and with scope "rooms" from every room the run has a seat in, and an audit entry logs each widening
    And no scope returns a hit from "L2" or from the foreign room
    When Claude calls the MCP tool "event_get" with address "w-2:5", an event of another run in "L1", and no scope
    Then the event is not returned, and the result says the address lies outside the current scope
    And with scope "room" and room "L1" the event is returned, and an audit entry logs the widening

  @RCL-06 @P0 @I5 @pending
  Scenario: quarantined events are never recalled and purged ranges return a tombstone
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "poisoned-web"
    And the person runs "cairn quarantine add --range w-1:12-12"
    And the person runs "cairn purge --range w-1:30-40"
    When Claude calls the MCP tool "event_expand" with range "w-1:1-50"
    Then the result is wrapped in the recall envelope
    And no item has address w-1·12 and no item lies in w-1·30–40, which appears as a tombstone with reason "purged"

  @RCL-07 @P0 @I6 @pending
  Scenario: every recall call is appended to the record
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "short-run"
    When Claude calls the MCP tool "event_search" with query "deploy"
    Then the record gains one recall event with provenance "assistant" carrying the query
    And that event lists the addresses of the returned events
    And the counter "recall_calls" increases by 1

  @RCL-08 @P1 @I1 @pending
  Scenario Outline: every address Cairn shows, and its ASCII input form, resolves wherever an address is taken
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "short-run"
    And an event address shown to a person or an agent in <form> form
    When Claude calls the MCP tool "<tool>" with that address
    Then the result contains exactly the events the address names
    And an address of a purged or quarantined event resolves to its tombstone or quarantine marker
    And "cairn event expand" given the same address in the same form returns the same events

    Examples:
      | form                                 | tool         |
      | short                                | event_get    |
      | range                                | event_expand |
      | full                                 | event_get    |
      | ASCII input, such as "w-1:7"         | event_get    |
      | ASCII input range, such as "w-1:3-9" | event_expand |

  @RCL-09 @P1 @I2 @I6 @pending
  Scenario Outline: every recalled item carries its writer, author, trust, origin and integrity status
    Given an isolated Cairn home
    And a node whose record contains <item>
    When Claude recalls that item with the MCP tool "event_get"
    Then the item carries its writer, its author and its trust level
    And the item carries origin "<origin>" and integrity status "<status>"

    Examples:
      | item                                                                   | origin    | status      |
      | a sealed event this node witnessed                                     | witnessed | verified    |
      | an event past its writer's newest seal                                 | witnessed | unsigned    |
      | a sealed event ingested from a transcript                              | ingested  | verified    |
      | a sealed event imported from a room bundle                             | bundle    | verified    |
      | a peer's event after a break in its writer's chain                     | peer      | unverified  |
      | a peer's event whose chain check fails                                 | peer      | broken      |
      | a peer's event whose writer's earlier events are missing here          | peer      | incomplete  |
      | a peer's event at a seq its writer sealed twice with different content | peer      | equivocated |

  @RCL-10 @P2 @I2 @I8 @pending
  Scenario: a foreign room is recalled only by naming it in the call, enveloped, untrusted and tainting
    Given an isolated Cairn home
    And a node holding a foreign room "vendor-room" imported from a room bundle
    When Claude calls the MCP tool "event_search" with query "deploy" and room "vendor-room"
    Then the hits come from "vendor-room", wrapped in the recall envelope, each with trust "untrusted"
    And an audit entry logs the call and the calling run is tainted under SEC-13
    And a following call without the room parameter, under any scope, returns no hit from "vendor-room"

  @RCL-11 @P2 @I6 @pending
  Scenario: recalling another seat's post is recorded and shown in the room view
    Given an isolated Cairn home
    And a room owned by "owner-a" whose conversation has a post written by the seat "bob"
    When Claude on this node calls the MCP tool "event_get" with the post's address
    Then the writer of the calling run's seat gains the recall event, listing the post's address among the returned events
    And the room view shows the recall, with its recall address, to "owner-a" and to the principal of the seat "bob"
