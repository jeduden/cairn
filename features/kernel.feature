Feature: Compute kernel (CMP)

  Scenarios for SRS §5.7, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @CMP-01 @P1 @pending
  Scenario: the kernel tools are listed by the MCP server
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "run-a"
    When the MCP server "cairn mcp" is asked to list its tools
    Then the tool list contains "kernel_exec", "kernel_variable_list" and "kernel_reset"
    And each kernel tool is backed by the Starlark interpreter

  @CMP-02 @P1 @pending
  Scenario: kernel variables persist across executions until reset
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "run-a"
    When the agent calls the MCP tool "kernel_exec" with code "x = 41"
    And the agent calls the MCP tool "kernel_exec" with code "print(x + 1)"
    Then the printed output is "42"
    When the agent calls the MCP tool "kernel_reset" with no arguments
    And the agent calls the MCP tool "kernel_variable_list" with no arguments
    Then the namespace lists no variables

  @CMP-03 @P1 @pending
  Scenario: read-only recall built-ins return structured values
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "run-a" containing the word "migration"
    When the agent calls the MCP tool "kernel_exec" with code "hits = cairn.event_search(query='migration'); print(type(hits), hits[0]['address'])"
    Then the printed output names a list and an address (writer, seq)
    And the built-ins "cairn.event_expand", "cairn.event_get", "cairn.landmark_list", "json", "re", "math" and "time" are callable
    And every global name the kernel exposes, the interpreter's universal built-ins included, is on the kernel's allow-list
    And a built-in added to the interpreter's universe is unavailable to kernel code
    And the record contains the same number of events as before the execution

  @CMP-04 @P1 @I4 @pending
  Scenario Outline: the kernel has no access to host resources
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "run-a"
    When the agent calls the MCP tool "kernel_exec" with code "<code>"
    Then the result reports an error naming "<resource>" as unavailable
    And no file, socket, process or environment variable was touched by the worker

    Examples:
      | code                            | resource    |
      | open('/etc/passwd')             | filesystem  |
      | http.get('https://example.com') | network     |
      | os.system('ls')                 | process     |
      | os.getenv('HOME')               | environment |

  @CMP-05 @P1 @I9 @pending
  Scenario Outline: exceeding a kernel limit restarts the worker and reports the lost namespace
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "run-a"
    And the agent has set the kernel variable "kept" with "kernel_exec"
    When the agent calls the MCP tool "kernel_exec" with code "<code>"
    Then the result is an error stating the <limit> limit was exceeded and the namespace was lost
    And the "cairn kernel-worker" child process has been restarted
    And an audit entry records "kernel <limit> limit exceeded"

    Examples:
      | code                                | limit           |
      | while True: pass                    | step budget     |
      | sleep_forever()                     | 10 s wall clock |
      | big = ['x' * 1048576] * 1024 * 1024 | 512 MiB memory  |

  @CMP-06 @P1 @I2 @pending
  Scenario: only printed output returns to the agent, capped, enveloped and tainted
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "run-a" containing an untrusted web result
    When the agent calls the MCP tool "kernel_exec" with code "r = cairn.event_search(query='web'); print(r * 10000)"
    Then the result is wrapped in the recall envelope
    And the printed output is capped at 8,000 model tokens with a truncation marker stating that variables persist
    And the envelope is tainted "untrusted" by the web event read during the execution
    And no value other than printed output is returned

  @CMP-07 @P1 @I5 @I8 @pending
  Scenario: kernel reads honour quarantine and recall scope
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "run-a"
    And an agent run with a Claude Code transcript "run-b"
    And the person has quarantined w-1·7 with "cairn quarantine add --range w-1:7-7"
    When the agent calls the MCP tool "kernel_exec" with code "print(cairn.event_get(address='w-1:7'), cairn.event_search(query='x', scope='rooms'))"
    Then the kernel returns exactly what the MCP tools "event_get" and "event_search" return for the same arguments
    And w-1·7, foreign rooms and rooms the run has no seat in are absent

  @CMP-08 @P2 @I4 @pending
  Scenario: the opt-in Python kernel is sandboxed, network-less and read-only
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "run-a"
    And the external Python kernel is enabled in the person's configuration
    When the agent calls the MCP tool "kernel_exec" with code "import socket; socket.create_connection(('example.com', 80))"
    Then the result reports that the network is unavailable
    And a write to the store from the Python worker is rejected
    And the record is unchanged

  @CMP-09 @P2 @I4 @pending
  Scenario: cairn never calls model APIs itself
    Given an isolated Cairn home
    And an agent run with a Claude Code transcript "run-a"
    When the shipped binary's imports and outbound network calls are inspected
    Then no model API client or model endpoint is referenced
    And parallel sub-calls over record data are only offered through harness subagents
