Feature: Pins (PIN)

  Scenarios for SRS §5.3, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @PIN-01 @P0 @I2 @I3 @pending
  Scenario Outline: only trusted actions create active pins
    Given an isolated Cairn home
    And deployment mode "<mode>"
    When <action> with the text "Never push directly to main"
    Then the active pin count is <active>

    Examples:
      | mode        | action                                      | active |
      | automation  | the operator runs "cairn pin add"           | 1      |
      | automation  | the person's configuration declares a pin     | 1      |
      | interactive | the user issues the slash command "/pin"    | 1      |
      | automation  | the user issues the slash command "/pin"    | 0      |
      | automation  | a person stamps an agent's room pin version | 1      |
      | interactive | the repository's ".cairn.toml" declares a pin  | 0      |

  @PIN-02 @P0 @I2 @pending
  Scenario: a pin Claude proposes stays an inactive assistant candidate
    Given an isolated Cairn home
    And deployment mode "interactive"
    When Claude calls the MCP tool "pins_propose" with text "Always run go test before committing" and type "constraint"
    Then the result gives a candidate ID and states that activation requires the user
    And the candidate is stored inactive with provenance "assistant"
    And the active pin count is 0
    And no MCP tool makes any pin active

  @PIN-03 @P0 @I3 @I5 @pending
  Scenario: a pin stores its verbatim text, room, creating address and commitment
    Given an isolated Cairn home
    And a run working in room "L1"
    When the operator runs "cairn pin add --type constraint --priority 1 'Never push directly to main; open a pull request.'"
    Then the command exits 0
    And the pin stores that text verbatim with type "constraint", priority 1, room "L1", the address (writer, seq) of its creating event, author "operator" and that event's commitment
    And the pin stores no bare hash of its text
    And adding a pin whose text is 1,001 characters long exits 2 and leaves the active pin count at 1

  @PIN-04 @P0 @I10 @pending
  Scenario: changing a pin records a removal followed by an addition
    Given an isolated Cairn home
    And an active pin "Never push directly to main" created at seq 10
    When the pin's text is changed to "Never push directly to main or release branches"
    Then the record gains a pin removal event for seq 10 followed by a pin addition event
    And the event at seq 10 and its pin text are unchanged
    And "cairn rebuild" reproduces the one active pin "Never push directly to main or release branches"

  @PIN-05 @P1 @I2 @I3 @pending
  Scenario Outline: pin candidates from user prompts activate only on trusted confirmation
    Given an isolated Cairn home
    And deployment mode "<mode>"
    And the person's configuration holds <config>
    When the hook "UserPromptSubmit" runs with prompt "Never edit files under migrations/ without asking"
    Then <candidates> inactive pin candidates are recorded
    And the active pin count is 0
    And the active pin count is <confirmed> after the operator runs "cairn pin confirm" on every candidate

    Examples:
      | mode        | config                    | candidates | confirmed |
      | interactive | no extra keys             | 1          | 1         |
      | automation  | no extra keys             | 0          | 0         |
      | automation  | pins.auto_activate = true | 0          | 0         |
      | automation  | inject.on_prompt = true   | 0          | 0         |

  @PIN-06 @P0 @I3 @pending
  Scenario: only constraint, preference and intent pins are injected automatically
    Given an isolated Cairn home
    And one active pin of each type "constraint", "preference", "decision", "fact" and "episode"
    And a room of the run whose owner set an intent, stored as its pin of type "intent", and recorded a verdict, stored as a pin of type "verdict"
    When the hook "SessionStart" runs with source "compact"
    Then the restore block includes the "constraint", "preference" and "intent" pins verbatim
    And the restore block includes no "decision", "fact", "episode" or "verdict" pin
    When the operator runs "cairn pin add --type note 'Prefer tabs'"
    Then the command exits 2
    When the operator runs "cairn pin add --type intent 'Ship CSV export'"
    Then the command exits 2

  @PIN-07 @P1 @I2 @I3 @pending
  Scenario: pre-compact guidance is static text free of record content
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "long-run"
    When the hook "PreCompact" runs with trigger "auto" for "long-run"
    Then the output carries guidance to preserve user-stated constraints verbatim
    And the guidance is byte-identical to that for trigger "manual" on an empty record
    And the guidance contains no text from "long-run" or from any pin
    And the hook exits 0 without blocking compaction

  @PIN-08 @P0 @I3 @I6 @pending
  Scenario: pins over the pin budget are omitted whole and the omission is stated
    Given an isolated Cairn home
    And 6 active constraint pins of 250 estimated tokens each, against the default pin budget of 1,000 tokens
    When the hook "SessionStart" runs with source "compact"
    Then the restore block holds 4 of the pins, ordered by priority, then by creating address, writer then seq
    And each included pin's text is complete and verbatim
    And the restore block states that 2 pins were omitted
    And an audit entry records "2 pins omitted over the pin budget"

  @PIN-09 @P1 @pending
  Scenario: doctor warns about a pin that repeats CLAUDE.md text
    Given an isolated Cairn home
    And a working tree whose "CLAUDE.md" contains "Never push directly to main."
    And an active pin "Never push directly to main."
    And an active pin "Run go test before committing."
    When the operator runs "cairn doctor --json"
    Then the output warns that the pin "Never push directly to main." duplicates text in "CLAUDE.md"
    And the output has no warning about the pin "Run go test before committing."

  @PIN-10 @P0 @I3 @I2 @pending
  Scenario: a restore block holds the principal's trusted pins of every room the run holds a seat in
    Given an isolated Cairn home
    And a run whose seats' writers record its joins to room "L1" and then room "L2"
    And a branch switch onto a branch of room "L4", which the run never joined, so its later events went to its personal-room seat
    And trusted pins of the run's principal in "L1" and in its personal room, a pin from the person's configuration, and a pin in "L2" written by another principal
    And the principal's stamp on one version of an agent's pin in "L1", and on one version of a second pin another principal wrote in "L2"
    And a trusted pin of the principal in "L4"
    And a principal's "/pin" whose creating event was recorded in interactive mode, while the current mode is "automation"
    When the hook "SessionStart" runs with source "compact"
    Then the restore block holds the "L1" pin, the personal-room pin, the configuration pin as a pin of the personal room, the interactive-mode pin, and both stamped versions, each under its original author
    And the restore block holds no pin of "L4"
    And the restore block names "L1", "L2" and the personal room by id
    And the other principal's unstamped pin is stated only by count, room id and key fingerprint, with no text, and an audit entry records it

  @PIN-11 @P2 @I3 @I6 @pending
  Scenario Outline: a pin active on another of the owner's nodes but not here is stated by count and reason
    Given an isolated Cairn home
    And a pin active on another of the owner's nodes that is not active on this node because <reason>
    When the hook "SessionStart" runs with source "compact"
    Then the restore block states that 1 such pin exists because <reason>
    And the restore block holds none of that pin's text
    And an audit entry records the pin not active on this node

    Examples:
      | reason                             |
      | its device key lacks the pin scope |
      | its certificate is revoked         |
      | its event has not arrived          |
