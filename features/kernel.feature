Feature: Compute kernel (CMP)

  Scenarios for SRS §5.7, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @CMP-01 @P1 @pending
  Scenario: the kernel tools are listed by the MCP server
    Given an isolated Cairn home
    And a project with a Claude Code transcript "session-a"
    When the MCP server "cairn mcp" is asked to list its tools
    Then the tool list contains "kernel_exec", "kernel_vars" and "kernel_reset"
    And each kernel tool is backed by the Starlark interpreter

  @CMP-02 @P1 @pending
  Scenario: namespace variables persist across executions until reset
    Given an isolated Cairn home
    And a project with a Claude Code transcript "session-a"
    When Claude calls the MCP tool "kernel_exec" with code "x = 41"
    And Claude calls the MCP tool "kernel_exec" with code "print(x + 1)"
    Then the printed output is "42"
    When Claude calls the MCP tool "kernel_reset" with no arguments
    And Claude calls the MCP tool "kernel_vars" with no arguments
    Then the namespace lists no variables

  @CMP-03 @P1 @pending
  Scenario: read-only recall built-ins return structured values
    Given an isolated Cairn home
    And a project with a Claude Code transcript "session-a" containing the word "migration"
    When Claude calls the MCP tool "kernel_exec" with code "hits = cairn.search(query='migration'); print(type(hits), hits[0]['seq'])"
    Then the printed output names a list and an address (writer, seq)
    And the built-ins "cairn.expand", "cairn.get", "cairn.landmarks", "json", "re", "math" and "time" are callable
    And every global name the kernel exposes, the interpreter's universal built-ins included, is on the kernel's allow-list
    And a built-in added to the interpreter's universe is unavailable to kernel code
    And the record holds the same number of events as before the execution

  @CMP-04 @P1 @I4 @pending
  Scenario Outline: the kernel has no access to host resources
    Given an isolated Cairn home
    And a project with a Claude Code transcript "session-a"
    When Claude calls the MCP tool "kernel_exec" with code "<code>"
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
    And a project with a Claude Code transcript "session-a"
    And Claude has set the kernel variable "kept" with "kernel_exec"
    When Claude calls the MCP tool "kernel_exec" with code "<code>"
    Then the result is an error stating the <limit> limit was exceeded and the namespace was lost
    And the "cairn kernel-worker" child process has been restarted
    And an audit entry records "kernel <limit> limit exceeded"

    Examples:
      | code                                | limit           |
      | while True: pass                    | step budget     |
      | sleep_forever()                     | 10 s wall clock |
      | big = ['x' * 1048576] * 1024 * 1024 | 512 MiB memory  |

  @CMP-06 @P1 @I2 @pending
  Scenario: only printed output returns to Claude, capped, enveloped and tainted
    Given an isolated Cairn home
    And a project with a Claude Code transcript "session-a" containing an untrusted web result
    When Claude calls the MCP tool "kernel_exec" with code "r = cairn.search(query='web'); print(r * 10000)"
    Then the result is wrapped in the recall envelope
    And the printed output is capped at 8,000 tokens with a truncation notice stating that variables persist
    And the envelope is tainted "untrusted" by the web event read during the execution
    And no value other than printed output is returned

  @CMP-07 @P1 @I5 @I8 @pending
  Scenario: kernel reads honour quarantine and recall scope
    Given an isolated Cairn home
    And a project with a Claude Code transcript "session-a"
    And a project with a Claude Code transcript "session-b"
    And the operator has quarantined w-1·7 with "cairn quarantine add --range w-1:7-7"
    When Claude calls the MCP tool "kernel_exec" with code "print(cairn.get(seq='w-1:7'), cairn.search(query='x', scope='project'))"
    Then the kernel returns exactly what the MCP tools "get" and "search" return for the same arguments
    And w-1·7, foreign lanes and other projects are absent

  @CMP-08 @P2 @I4 @pending
  Scenario: the opt-in Python kernel is sandboxed, network-less and read-only
    Given an isolated Cairn home
    And a project with a Claude Code transcript "session-a"
    And the external Python kernel is enabled in the tenant config
    When Claude calls the MCP tool "kernel_exec" with code "import socket; socket.create_connection(('example.com', 80))"
    Then the result reports that the network is unavailable
    And a write to the store from the Python worker is rejected
    And the store's record is unchanged

  @CMP-09 @P2 @I4 @pending
  Scenario: cairn never calls model APIs itself
    Given an isolated Cairn home
    And a project with a Claude Code transcript "session-a"
    When the shipped binary's imports and outbound network calls are inspected
    Then no model API client or model endpoint is referenced
    And parallel sub-calls over record data are only offered through harness subagents
