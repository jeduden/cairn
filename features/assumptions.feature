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
  Scenario: transcripts are one JSONL file per harness session with subagents nested (S3)
    Given a recorded "~/.claude/projects" tree for Claude Code "supported"
    When the transcript discovery walks the tree
    Then each harness session is one JSONL file under "~/.claude/projects/<project-slug>/"
    And each subagent transcript lies under "<session_id>/subagents/"

  @ASM-04 @pending
  Scenario Outline: transcripts are append-mostly but can be truncated or rewritten (S3)
    Given a recorded transcript before and after a "<flow>" flow for Claude Code "supported"
    When the person runs "cairn ingest --all" on each recording in turn
    Then the second recording is detected as <change>
    And an audit entry records "transcript <change>"

    Examples:
      | flow   | change    |
      | rewind | truncated |
      | resume | rewritten |

  @ASM-05 @pending
  Scenario: transcripts can be deleted after cleanupPeriodDays (S3)
    Given a recorded "~/.claude/projects" tree before and after cleanup with "cleanupPeriodDays" of 30
    When the person runs "cairn ingest --all" on each recording in turn
    Then the runs whose transcripts were deleted remain recallable from the store
    And an audit entry records "transcript deleted by harness cleanup"

  @ASM-06 @pending
  Scenario: subagent compaction hooks carry the parent's identity (S1)
    Given a recorded hook payload for Claude Code "supported"
    And the payload was captured during a subagent's compaction
    When the hook "PreCompact" runs with the recorded payload
    Then its "session_id" and "transcript_path" are the parent's
    And it carries no subagent-specific field

  @ASM-07 @pending
  Scenario Outline: a plugin bundling hooks and an MCP server loads on each target (S4)
    Given the Cairn plugin bundling its hook handlers and the "cairn mcp" server
    When it is installed on "<target>"
    Then the recorded harness session shows every hook Cairn registered firing
    And the recorded harness session lists the Cairn MCP tools

    Examples:
      | target           |
      | workstation      |
      | runner image     |
      | Agent SDK worker |

  @ASM-08 @pending
  Scenario: self-hosted runners seed ~/.claude into each harness session under one user account (S4)
    Given a recorded self-hosted runner harness session for Claude Code "supported"
    When the harness session's home and the user account of its process are inspected
    Then the harness session's "~/.claude/" was seeded from the runner host's "~/.claude/"
    And every harness session on the runner ran as the same one user account

  @ASM-09 @pending
  Scenario: whether a PreCompact hook exiting 2 blocks compaction is recorded (S6)
    Given a recorded hook payload for Claude Code "supported"
    And a PreCompact hook that exits 2
    When compaction is triggered manually and at a full window
    Then the recorded transcript shows whether compaction was blocked or proceeded
    And the observed behaviour is written into the OQ-01 recommendation

  @ASM-10 @pending
  Scenario: hook timeouts are per hook but SessionEnd gets a short shared hook budget (S1)
    Given a recorded hook payload for Claude Code "supported"
    And hooks configured with per-hook timeouts of 10 s
    When the hook "SessionEnd" runs with a handler that sleeps past 1.5 s
    Then the recorded harness terminated the handler after about 1.5 s
    And the other hooks ran up to their configured 10 s timeout

  @ASM-11 @pending
  Scenario: context the harness adds is written as attachment lines (S3)
    Given a recorded transcript for Claude Code "supported" from a working tree with a CLAUDE.md file, an MCP server and a skill
    When each line of the recording is read
    Then the CLAUDE.md file, the MCP server instructions and the skill listing appear as "attachment" lines
    And each of those lines carries "attachment.type" and "renderedRole"

  @ASM-12 @pending
  Scenario: one API message spans several lines, one content block each (S3)
    Given a recorded transcript for Claude Code "supported" with an assistant reply carrying text and two tool calls
    When each line of the recording is read
    Then each assistant line carries exactly one content block
    And the reply's lines share one "message.id" and repeat its "usage"

  @ASM-13 @pending
  Scenario: a tool-result line repeats the output and names its call (S3)
    Given a recorded transcript for Claude Code "supported" with a Bash tool call and its result
    When each line of the recording is read
    Then the result line carries the output in "message.content" and again in "toolUseResult"
    And its "sourceToolAssistantUUID" names the line carrying the Bash tool call

  @ASM-14 @pending
  Scenario Outline: some transcript line types carry no uuid or no timestamp (S3)
    Given a recorded transcript for Claude Code "supported"
    When each line of the recording is read
    Then every line of type "<type>" carries <missing>

    Examples:
      | type            | missing                        |
      | last-prompt     | neither "uuid" nor "timestamp" |
      | queue-operation | no "uuid"                      |
      | ai-title        | neither "uuid" nor "timestamp" |

  @ASM-15 @pending
  Scenario: thinking blocks keep their signature but not their text (S3)
    Given a recorded transcript for Claude Code "supported" with extended thinking enabled
    When each line of the recording is read
    Then every thinking block carries a signature and empty thinking text

  @ASM-16 @pending
  Scenario: how working directories map to transcript directories is recorded (S3)
    Given a recorded "~/.claude/projects" tree for Claude Code "supported" after one harness session in each of "/w/my_app", "/w/my.app" and "/w/my-app"
    When the transcript discovery walks the tree
    Then the three harness sessions share the transcript directory "-w-my-app"

  @ASM-17 @pending
  Scenario: transcript lines are appended within 1 s of their event while an agent runs (S9)
    Given a recorded harness session from Claude Code "supported" with the time of each hook event
    When the recorded transcript is replayed with the write time of each line
    Then each transcript line was appended within 1 s of its event

  @ASM-18 @pending
  Scenario: the harness prompt stays answerable during PermissionRequest and accepts no decision (S9)
    Given a recorded hook payload for Claude Code "supported"
    When the hook "PermissionRequest" runs with a handler that waits and then returns no decision
    Then the recorded harness kept its own permission prompt answerable while the handler waited
    And the recorded harness accepted the handler's exit without a decision and left the choice to its prompt

  @ASM-19 @pending
  Scenario Outline: each agent harness exposes inputs for steer, interrupt and stop (S10)
    Given the recorded input interface of "<harness>"
    When its inputs are listed
    Then it exposes an input for steer, one for interrupt and one for stop

    Examples:
      | harness          |
      | Agent SDK        |
      | ACP              |
      | Codex app-server |

  @ASM-20 @pending
  Scenario: a clone's root commit, HEAD, refs and trees are obtainable inside the core, with no socket and no program start (S11)
    Given a recorded git clone
    When Cairn reads the clone with process creation and the network forbidden
    Then it resolves the root commit, HEAD, every ref and the tree of every ref
    And no process was started and no connection was opened

  @ASM-21 @pending
  Scenario: the harness hands a run seat's private key to its own MCP server outside the model's context (S4)
    Given a recorded plugin launch of Claude Code "supported" with its MCP server
    When the harness starts the MCP server for a run with a run seat key
    Then the MCP server receives the key at launch through a channel the harness keeps out of the model's context
    And no transcript line, hook payload or tool result of the run carries the key
