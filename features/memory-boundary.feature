Feature: Boundary to durable memory (MEM)

  Scenarios for SRS §5.9, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @MEM-01 @P0 @I2 @pending
  Scenario: record content is never promoted into durable memory
    Given an isolated Cairn home
    And a project with a Claude Code transcript "memory-canary"
    When every hook runs over the whole transcript and the operator runs "cairn rebuild"
    Then no file outside the Cairn home record store contains the canary text
    And no memory, summary or cross-project store under the Cairn home or HOME contains the canary text

  @MEM-02 @P1 @I2 @pending
  Scenario: downstream memory consumes only the trusted export
    Given an isolated Cairn home
    And a project with a Claude Code transcript "mixed-provenance"
    When the operator runs "cairn export --trusted-only"
    Then the command exits 0
    And the JSONL output holds only trusted events, each with full provenance
    And the documented integration path for downstream memory systems is that export alone
