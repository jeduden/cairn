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
      | automation  | a person stamps a version of a constraint pin an agent wrote                               | 1          |
      | interactive | the repository's ".cairn.toml" declares a pin                                              | 0          |

  @PIN-02 @P0 @I2 @pending
  Scenario: text an agent proposes stays a pin candidate with no author
    Given an isolated Cairn home
    And deployment mode "interactive"
    When the agent calls the MCP tool "pin_candidate_propose" with text "Always run go test before committing" and type "constraint"
    Then the result gives a pin candidate id and states that only the agent's principal can confirm it
    And the pin candidate is stored as proposed pin text, not a pin, with provenance "assistant" and no author
    And the qualifying pin count is 0
    And no MCP tool confirms a pin candidate or makes any pin restore

  @PIN-03 @P0 @I3 @I5 @pending
  Scenario: a pin stores its verbatim text, room, creating address and commitment
    Given an isolated Cairn home
    And a run working in room "L1"
    When the person runs "cairn pin add --type constraint --priority 1 'Never push directly to main; open a pull request.'"
    Then the command exits 0
    And the pin stores that text verbatim with type "constraint", priority 1, room "L1", the address (writer, seq) of its creating event, as author the device seat of "alice", the node's principal, and that event's commitment
    And the pin stores no bare hash of its text
    And adding a pin whose text is 1,001 characters long exits 2 and leaves the qualifying pin count at 1

  @PIN-04 @P0 @I10 @pending
  Scenario: editing a pin adds a pin version and keeps every earlier one
    Given an isolated Cairn home
    And a qualifying pin "Never push directly to main" created at address w-1·10
    When the person runs "cairn pin edit" to change the pin to "Never push directly to main or release branches"
    Then the record gains an edit event adding version 2 of the pin created at w-1·10
    And the event at w-1·10 and its pin text are unchanged
    And version 1 stays readable, and "cairn rebuild" reproduces the one qualifying pin at version 2, "Never push directly to main or release branches"

  @PIN-05 @P1 @I2 @I3 @pending
  Scenario Outline: pin candidates from user turns become pins only on the principal's confirmation
    Given an isolated Cairn home
    And deployment mode "<mode>"
    And the person's configuration contains <config>
    When the hook "UserPromptSubmit" runs with prompt "Never edit files under migrations/ without asking"
    Then <candidates> pin candidates are recorded, each proposed pin text with no author
    And the qualifying pin count is 0
    And the qualifying pin count is <confirmed> after the person confirms every pin candidate with "cairn pin-candidate confirm", each a widening principal act
    And each confirmation makes a new pin that the principal's device seat authors
    And only the owner's confirmation turns an intent or criterion candidate into a pin, as a new version of the room's intent pin authored by the owner's device seat

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
    And a room of the run whose owner set an intent, stored as its pin of type "intent", and recorded a verdict, stored as a pin of type "verdict"
    When the hook "SessionStart" runs with source "compact"
    Then the restore block includes the "constraint", "preference" and "intent" pins verbatim
    And the restore block includes no "decision", "fact", "episode" or "verdict" pin
    When the person runs "cairn pin add --type note 'Prefer tabs'"
    Then the command exits 2
    When the person runs "cairn pin add --type intent 'Ship CSV export'"
    Then the command exits 2

  @PIN-07 @P1 @I2 @I3 @pending
  Scenario: pre-compact guidance is static text free of record content
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "long-run"
    When the hook "PreCompact" runs with trigger "auto" for "long-run"
    Then the output carries guidance to preserve user-stated constraints verbatim
    And the guidance is byte-identical to that for trigger "manual" on an empty record
    And the guidance contains no text from "long-run" or from any pin
    And the hook handler exits 0 without blocking compaction

  @PIN-08 @P0 @I3 @I6 @pending
  Scenario: pins over the pin budget are omitted whole, and each omitted pin is named by id and counted
    Given an isolated Cairn home
    And 6 qualifying constraint pins of 250 estimated model tokens each, against the default pin budget of 1,000 model tokens
    When the hook "SessionStart" runs with source "compact"
    Then the restore block includes 4 of the pins, ordered by pin priority, then by creating address, writer then seq
    And each included pin's text is complete and verbatim
    And the restore block names each of the 2 omitted pins by id and states that 2 pins were omitted
    And an audit entry records "2 pins omitted over the pin budget"

  @PIN-09 @P1 @pending
  Scenario: doctor warns about a pin that repeats CLAUDE.md text
    Given an isolated Cairn home
    And a working tree whose "CLAUDE.md" contains "Never push directly to main."
    And a qualifying pin "Never push directly to main."
    And a qualifying pin "Run go test before committing."
    When the person runs "cairn doctor --json"
    Then the output warns that the pin "Never push directly to main." duplicates text in "CLAUDE.md"
    And the output has no warning about the pin "Run go test before committing."

  @PIN-10 @P0 @I3 @I2 @pending
  Scenario: a restore block includes the qualifying pins of every room the run has a seat in
    Given an isolated Cairn home
    And a run whose seats' writers record it joining room "L1" and then creating room "L2"
    And a branch switch onto a branch of room "L4", which the run neither joined nor created, so its later events went to its personal-room seat
    And pins the run's principal wrote from its device seat in "L1" and in its personal room, a pin from the person's configuration, and a pin in "L2" written from another principal's device seat
    And a pin in "L2" written from the device seat of a third principal whose key the run's principal trusts in "L2" by a trust grant
    And the principal's stamp on one version of a constraint pin an agent's run seat wrote in "L1", on one version of a second constraint pin another principal wrote in "L2", and on one version of a "fact" pin in "L1"
    And a pin the principal wrote from its device seat in "L4"
    And a constraint pin in "L1" written from the device seat of a token-key-only node of the run's principal, unstamped
    And a pin the principal confirmed from a pin candidate whose creating user turn was recorded in the deployment mode "interactive", while the current deployment mode is "automation"
    When the hook "SessionStart" runs with source "compact"
    Then the restore block includes the "L1" pin, the personal-room pin, the configuration pin as a pin of the personal room, the confirmed pin, the pin the trust grant covers, and both stamped constraint versions, each under its original author
    And the restore block includes no pin of "L4", not the stamped "fact" version and not the token-key-only node's unstamped pin
    And the restore block names "L1", "L2" and the personal room by id
    And the other principal's unstamped pin is stated only by count, room id and key fingerprint, with no text, and an audit entry records it

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
      | its seat certificate is revoked                                 |
      | its event has not arrived                                       |
