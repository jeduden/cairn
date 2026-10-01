Feature: Assumptions register (ASM)

  Scenarios for SRS §2.3, one per assumption, tagged with its id. The
  rows carry neither priority nor traces, so a scenario here carries
  its id alone. Each is a contract test (ENG-17) that replays hook
  payloads and transcripts recorded from Claude Code by the M0 spike
  named in the row's "Verified in" column, and is re-run against every
  supported Claude Code release. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the spike's
  recorded fixtures land.

  @ASM-01 @pending
  Scenario Outline: hook payloads carry the documented input fields (S1)
    Given a recorded hook payload for Claude Code "<version>"
    When the hook "<Event>" runs with the recorded payload
    Then the payload carries "session_id", "transcript_path", "cwd" and "hook_event_name"
    And it also carries <extra>

    Examples:
      | version   | Event            | extra                                         |
      | supported | SessionStart     | "source" in startup, resume, clear or compact |
      | supported | PreCompact       | "trigger" and "custom_instructions"           |
      | supported | PostCompact      | "compact_summary"                             |
      | supported | UserPromptSubmit | "prompt"                                      |

  @ASM-02 @pending
  Scenario Outline: additionalContext is honoured on SessionStart and UserPromptSubmit only (S1)
    Given a recorded hook payload for Claude Code "<version>"
    And a recorded harness response to a hook output carrying "additionalContext"
    When the hook "<Event>" runs with the recorded payload
    Then the recorded transcript <shows> the additional context in the model's context

    Examples:
      | version   | Event            | shows       |
      | supported | SessionStart     | shows       |
      | supported | UserPromptSubmit | shows       |
      | supported | PostCompact      | never shows |

  @ASM-03 @pending
  Scenario: transcripts are one JSONL file per session with subagents nested (S3)
    Given a recorded "~/.claude/projects" tree for Claude Code "supported"
    When the transcript discovery walks the tree
    Then each session is one JSONL file under "~/.claude/projects/<project-slug>/"
    And each subagent transcript lies under "<session>/subagents/"

  @ASM-04 @pending
  Scenario Outline: transcripts are append-mostly but can be truncated or rewritten (S3)
    Given a recorded transcript before and after a "<flow>" flow for Claude Code "supported"
    When the operator runs "cairn ingest --all" on each recording in turn
    Then the second recording is detected as <change>
    And an audit entry records "transcript <change>"

    Examples:
      | flow   | change    |
      | rewind | truncated |
      | resume | rewritten |

  @ASM-05 @pending
  Scenario: transcripts can be deleted after cleanupPeriodDays (S3)
    Given a recorded "~/.claude/projects" tree before and after cleanup with "cleanupPeriodDays" of 30
    When the operator runs "cairn ingest --all" on each recording in turn
    Then the sessions whose transcripts were deleted remain recallable from the store
    And an audit entry records "transcript deleted by harness cleanup"

  @ASM-06 @pending
  Scenario: subagent compaction hooks carry the parent's identity (S1)
    Given a recorded hook payload for Claude Code "supported"
    And the payload was captured during a subagent's compaction
    When the hook "PreCompact" runs with the recorded payload
    Then its "session_id" and "transcript_path" are the parent session's
    And it carries no subagent-specific field

  @ASM-07 @pending
  Scenario Outline: a plugin bundling hooks and an MCP server loads on each target (S4)
    Given the Cairn plugin bundling its hooks and the "cairn mcp" server
    When it is installed on "<target>"
    Then the recorded session shows every Cairn hook firing
    And the recorded session lists the Cairn MCP tools

    Examples:
      | target           |
      | workstation      |
      | runner image     |
      | Agent SDK worker |

  @ASM-08 @pending
  Scenario: self-hosted runners seed ~/.claude into each session under one user account (S4)
    Given a recorded self-hosted runner session for Claude Code "supported"
    When the session's home and process owner are inspected
    Then the session's "~/.claude/" was seeded from the runner host's "~/.claude/"
    And every session on the runner ran as the same one user account

  @ASM-09 @pending
  Scenario: whether a PreCompact hook exiting 2 blocks compaction is recorded (S6)
    Given a recorded hook payload for Claude Code "supported"
    And a PreCompact hook that exits 2
    When compaction is triggered manually and at a full window
    Then the recorded transcript shows whether compaction was blocked or proceeded
    And the observed behaviour is written into the OQ-01 recommendation

  @ASM-10 @pending
  Scenario: hook timeouts are per hook but SessionEnd gets a short shared budget (S1)
    Given a recorded hook payload for Claude Code "supported"
    And hooks configured with per-hook timeouts of 10 s
    When the hook "SessionEnd" runs with a handler that sleeps past 1.5 s
    Then the recorded harness terminated the handler after about 1.5 s
    And the other hooks ran up to their configured 10 s timeout

  @ASM-11 @pending
  Scenario Outline: tool hooks carry tool inputs and can return context (S1)
    Given a recorded hook payload for Claude Code "supported"
    When the hook "<Event>" runs with the recorded payload and returns additionalContext "cairn-probe"
    Then the payload carries <fields>
    And the recorded transcript shows "cairn-probe" in Claude's context before its next step

    Examples:
      | Event       | fields                                                         |
      | PreToolUse  | "tool_name" and "tool_input"                                   |
      | PostToolUse | "tool_name", "tool_input" and "tool_response" with its failure |

