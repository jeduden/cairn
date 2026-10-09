Feature: Pins (PIN)

  Scenarios for SRS §5.3, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @PIN-01 @P0 @I2 @I3 @pending
  Scenario Outline: only the principal's widening principal acts create qualifying pins
    Given an isolated Cairn home
    And deployment mode "<mode>"
    When <action> with the text "Never push directly to main"
    Then the qualifying pin count is <qualifying>

    Examples:
      | mode        | action                                                                                     | qualifying |
      | automation  | the person runs "cairn pin add"                                                            | 1          |
      | automation  | the person's configuration declares a pin and a widening principal act recorded its digest | 1          |
      | interactive | the person runs "cairn pin add"                                                            | 1          |
      | automation  | a harness skill calls the MCP tool "pin_candidate_propose"                                 | 0          |
      | interactive | the repository's ".cairn.toml" declares a pin                                              | 0          |

  @PIN-02 @P0 @I2 @pending
  Scenario: text an agent proposes stays a pin candidate with no pin author
    Given an isolated Cairn home
    And deployment mode "interactive"
    When the agent calls the MCP tool "pin_candidate_propose" with text "Always run go test before committing" and type "constraint"
    Then what the tool returns gives a pin candidate id and states that only the agent's principal can confirm it
    And the pin candidate is stored as proposed pin text, not a pin, with provenance "assistant" and no pin author
    And the qualifying pin count is 0
    And no MCP tool confirms a pin candidate or makes any pin restore
    And "cairn pin list" lists the pin candidate with provenance "assistant", marked untrusted

  @PIN-03 @P0 @I3 @I5 @pending
  Scenario: a pin stores its verbatim text, room, creating address and commitment
    Given an isolated Cairn home
    And a run working in room "L1"
    When the person runs "cairn pin add --room L1 --type constraint --priority 1 'Never push directly to main; open a pull request.'"
    Then the command exits 0
    And the pin stores that text verbatim with type "constraint", priority 1, room "L1", the address (writer, seq) of its creating event, as the author of its first version, and so of the pin, the device seat of "alice", the node's principal, and that event's commitment
    And the pin stores no bare hash of its text
    And adding a pin whose text is 1,001 characters long exits 2 and leaves the qualifying pin count at 1

  @PIN-04 @P0 @I10 @pending
  Scenario: editing a pin adds a pin version and keeps every earlier one
    Given an isolated Cairn home
    And a qualifying pin "Never push directly to main" created at address A1·10
    When the person runs "cairn pin edit" to change the pin to "Never push directly to main or release branches"
    Then the record gains an edit event adding version 2 of the pin created at A1·10
    And the event at A1·10 and its pin text are unchanged
    And version 2's author is the seat that ran the edit, version 1 keeps its own author, and the pin's author stays version 1's
    And version 2 keeps the pin's type and priority, and changing either takes an unpin and a new pin
    And version 1 stays readable, and "cairn rebuild" reproduces the one qualifying pin at version 2, "Never push directly to main or release branches"

  @PIN-05 @P1 @I2 @I3 @pending
  Scenario Outline: pin candidates from "user" events become pins only on the principal's confirmation
    Given an isolated Cairn home
    And deployment mode "<mode>"
    And the person's configuration contains <config>
    When the hook "UserPromptSubmit" runs with prompt "Never edit files under migrations/ without asking"
    Then <candidates> pin candidates are derived, each over its creating "user" event, holding its text by address, with no provenance of its own and no pin author
    And the qualifying pin count is 0
    And the qualifying pin count is <confirmed> after the person confirms every pin candidate with "cairn pin-candidate confirm", each its own widening principal act
    And before recording each confirmation, "cairn pin-candidate confirm" shows the candidate's exact text, pin type and priority
    And each confirmation makes a new pin that the principal's device seat authors
    And only the owner's confirmation turns an intent or criterion candidate into a pin, as a new version of the room's intent pin authored by the owner's device seat
    And "cairn pin-candidate confirm" on a pin candidate the agent proposed through "pin_candidate_propose" also shows its provenance "assistant", marked untrusted

    Examples:
      | mode        | config                                               | candidates | confirmed |
      | interactive | no extra keys                                        | 1          | 1         |
      | automation  | no extra keys                                        | 0          | 0         |
      | automation  | a setting that confirms pin candidates automatically | 0          | 0         |
      | automation  | restore_block.on_prompt = true                       | 0          | 0         |

  @PIN-06 @P0 @I3 @pending
  Scenario: only constraint, preference and intent pins are injected automatically
    Given an isolated Cairn home
    And one pin of each type "constraint", "preference", "decision", "fact" and "episode", each written from the device seat of the run's principal
    And a room of the run whose owner set an intent, stored as its pin of type "intent"
    When the hook "SessionStart" runs with source "compact"
    Then the restore block includes the "constraint", "preference" and "intent" pins verbatim
    And the restore block includes no "decision", "fact" or "episode" pin
    When the person runs "cairn pin add --type note 'Prefer tabs'"
    Then the command exits 2
    When the person runs "cairn pin add --type intent 'Ship CSV export'"
    Then the command exits 2

  @PIN-07 @P1 @I2 @I3 @pending
  Scenario: compaction guidance is fixed text Cairn ships, free of record content
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "long-run"
    When the hook "PreCompact" runs with trigger "auto" for "long-run"
    Then the output carries compaction guidance to preserve constraints stated in trusted "user" events verbatim
    And the compaction guidance is byte-identical to that for trigger "manual" on an empty record
    And the compaction guidance is TrustedText, built only from fixed text Cairn ships, and contains no text from "long-run" or from any pin
    And the hook handler exits 0 without blocking compaction

  @PIN-08 @P0 @I3 @I6 @pending
  Scenario: pins over the pin budget are omitted whole, and each omitted pin is named by id and counted
    Given an isolated Cairn home
    And 6 qualifying constraint pins of 250 estimated model tokens each, against the default pin budget of 1,000 model tokens: 4 in the run's personal room, and 2, each with a lower pin priority number than any of those 4, in a room only an earlier run of the same agent had a seat in
    When the hook "SessionStart" runs with source "compact"
    Then the restore block includes the 4 personal-room pins, ordered by pin priority, then by creating address, writer then seq, and not the 2 of the earlier run's room
    And each included pin's text is complete and verbatim
    And the restore block names each of the 2 omitted pins by id and states that 2 pins were omitted
    And an audit entry records "2 pins omitted over the pin budget"

  @PIN-09 @P1 @pending
  Scenario: doctor warns about a pin that repeats CLAUDE.md text
    Given an isolated Cairn home
    And a worktree whose "CLAUDE.md" contains "Never push directly to main."
    And a qualifying pin "Never push directly to main."
    And a qualifying pin "Run go test before committing."
    When the person runs "cairn doctor --json"
    Then the output warns that the pin "Never push directly to main." duplicates text in "CLAUDE.md"
    And the output has no warning about the pin "Run go test before committing."

  @PIN-10 @P0 @I3 @I2 @pending
  Scenario: a restore block includes the qualifying pins of every room the run has had a seat in during the run
    Given an isolated Cairn home
    And a run whose seats' writers record it joining room "L1", then creating room "L2"
    And a branch switch onto a branch of room "L4", which the run neither joined nor created, so its later events went to its personal-room seat
    And pins the run's principal wrote from its device seat in "L1" and in its personal room, and a pin from the person's configuration
    And a pin the principal wrote from its device seat in "L4"
    And a constraint pin the principal wrote from its device seat in "L1" and then edited from that seat, each version's own event trusted on this node
    And a pin the principal confirmed from a pin candidate whose creating "user" event was recorded in the deployment mode "interactive", while the current deployment mode is "automation"
    When the hook "SessionStart" runs with source "compact"
    Then the restore block includes the "L1" pin, the personal-room pin, the configuration pin as a pin of the personal room and the confirmed pin
    And of the edited constraint pin the restore block includes only its newest version, or, once a widening principal act quarantines or purges that version, its earlier one
    And the restore block includes no pin of "L4"
    And the restore block names "L1", "L2" and the personal room by id
    And a later run of the same agent, by the harness's stable agent id, with no seat in "L1" or "L2", gets the same pins of those rooms in its restore block

  @PIN-11 @P2 @I3 @I6 @pending
  Scenario Outline: a pin that qualifies on another of the principal's nodes but not here is stated by count and reason
    Given an isolated Cairn home
    And a pin that qualifies on another of the principal's nodes but not on this node because <reason>
    When the hook "SessionStart" runs with source "compact"
    Then the restore block states that 1 such pin exists because <reason>
    And the restore block includes none of that pin's text
    And an audit entry records that the pin does not qualify on this node

    Examples:
      | reason                                                          |
      | its seat certificate or token key does not cover the pin's room |
      | a certificate it chains through is revoked                      |
      | its event has not arrived                                       |
