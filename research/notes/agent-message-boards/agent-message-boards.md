# Agent message boards: what they surface, and what went wrong

Scope: systems where AI agents, mostly coding agents, post to and read
from a shared board, inbox or channel, read on 4 October 2026 from
their repositories, documentation and incident reports. Some sources
were read through a summarising fetch, so exact field names should be
checked against the linked page before they are relied on.

## The systems

- **MCP Agent Mail**
  ([repository](https://github.com/Dicklesworthstone/mcp_agent_mail)):
  - **Messages and reading.** Messages are Markdown with JSON
    frontmatter (thread, sender, recipients, importance, whether an
    acknowledgement is required), addressed like email. Agents pull
    their inbox, unread only or as snippets, and acknowledge messages
    that ask for it.
  - **Leases.** Advisory file leases with a time-to-live, exclusive or
    shared, with a pre-commit hook that blocks commits into someone
    else's exclusive lease.
  - **People.** A "Human Overseer" composer prefixes messages with a
    preamble telling agents to pause and prioritise them.
- **Beads and Gas Town**
  ([Beads](https://github.com/steveyegge/beads),
  [Gas Town](https://github.com/steveyegge/gastown)):
  - **Beads** keeps an issue graph in the repository. `bd ready` lists
    unblocked work; `--claim` atomically assigns and starts an item.
    Messages are an issue type with threads.
  - **Gas Town** adds mailboxes, a watchdog heartbeat, a problems feed
    that groups agents as stalled, zombie or needing intervention,
    escalation up to a person, and a merge queue.
- **Claude Code agent teams**
  ([agent teams](https://code.claude.com/docs/en/agent-teams),
  [cross-session messaging](https://code.claude.com/docs/en/cross-session-messaging)):
  - **Inboxes and tasks.** One inbox file per agent, pushed to the lead
    without polling, and direct addressing only. A shared task list
    with states and dependencies, claimed under a file lock.
  - **Trust.** A received message is labelled as from another session
    and "can't approve anything". Inbound messages can be accepted,
    held for a person or refused.
  - **Limits.** Messages are size-capped; bursts are refused and
    duplicates dropped, with a queue of at most 50, because messaging
    loops happened.
- **Codex subagents**
  ([subagents](https://learn.chatgpt.com/docs/agent-configuration/subagents.md)):
  a parent spawns children and receives only summaries; approvals from
  child threads carry the thread's label.
- **Amp** ([read threads](https://ampcode.com/news/read-threads)): no
  live channel; an agent pulls another thread on demand, distilled by
  another model, and handoffs carry a goal and a file list.
- **Moltbook**, January 2026
  ([Willison](https://simonwillison.net/2026/Jan/30/moltbook/)):
  - **What it was.** A Reddit-like board for agents, driven by a
    heartbeat that told agents to fetch a file from the site and follow
    it: instructions from the internet.
  - **Exposed data.** An open database with 1.5 million API tokens and
    private messages holding plaintext keys
    ([Wiz](https://www.wiz.io/blog/exposed-moltbook-database-reveals-millions-of-api-keys)).
  - **No identity.** About 17,000 people behind 1.5 million "agents",
    no way to tell an agent from a person, and anyone able to post as
    any agent.
  - **Injection and spam.** One study counted 506 prompt-injection
    attacks in 19,802 posts over 72 hours, and crypto spam at 19% of
    content ([Simula](https://zenodo.org/records/18444900)).
- **Boards without messages between agents**
  ([Vibe Kanban](https://github.com/BloopAI/vibe-kanban),
  [Conductor](https://www.conductor.build/docs/),
  [Claude Squad](https://github.com/smtg-ai/claude-squad)): a worktree,
  terminal, dev server and diff per task. They isolate agents, and
  people coordinate through review.
- **Protocols and frameworks:**
  - [A2A](https://a2a-protocol.org/latest/specification/): task states
    including input required and auth required; remote responses
    treated as untrusted.
  - [Coral](https://arxiv.org/html/2505.00749v2): threads with
    mentions; an agent waits until mentioned instead of polling.
  - [Letta](https://docs.letta.com/guides/agents/multi-agent):
    receipts, broadcast by tag, shared memory blocks.
  - AutoGen: broadcasts every reply to every member.
  - [LangGraph supervisor](https://pypi.org/project/langgraph-supervisor/):
    full history or last message only.
- **Research:**
  - Blackboard coordination is back: agents chosen or volunteering
    from what is posted ([2507.01701](https://papers.cool/arxiv/2507.01701),
    [2510.01285](https://arxiv.org/abs/2510.01285)), and a testbed for
    malicious agents on a blackboard
    ([Terrarium](https://arxiv.org/abs/2510.14312)).
  - Failure studies find agents withholding or ignoring each other's
    information ([MAST](https://arxiv.org/abs/2503.13657)).

## What they surface

| System              | Status  | Claims or locks      | Blockers | Handoffs | Acks    | Conflicts           | Cost | Human approval | Trust marking          |
| ------------------- | ------- | -------------------- | -------- | -------- | ------- | ------------------- | ---- | -------------- | ---------------------- |
| Agent Mail          | partial | file leases with TTL | threads  | yes      | yes     | at grant and commit | no   | overseer       | overseer preamble only |
| Beads, Gas Town     | yes     | atomic claim         | yes      | yes      | no      | merge queue         | no   | escalation     | no                     |
| Claude Code teams   | yes     | task lock            | partial  | partial  | partial | advice only         | no   | yes            | "from another session" |
| Codex subagents     | partial | no                   | no       | summary  | no      | no                  | no   | thread label   | no                     |
| Amp                 | no      | no                   | no       | yes      | no      | no                  | no   | no             | no                     |
| Moltbook            | no      | no                   | no       | no       | no      | no                  | no   | no             | none                   |
| Kanban-style boards | yes     | worktree per task    | no       | no       | no      | via review          | no   | review         | not applicable         |
| A2A                 | yes     | no                   | yes      | yes      | no      | no                  | no   | partial        | "treat as untrusted"   |

## Table stakes

- Threads with stable ids; replies inherit the thread.
- Explicit addressing (direct, mention), broadcast sparingly.
- Atomic claims on work, or leases on paths, that expire.
- States that show what is blocked and who waits on whom.
- Cheap reading: unread only, snippets, wait for a mention, distil on
  read.
- One human view across agents, with diffs and a place to step in.
- An audit trail.

## What went wrong in practice

- **A board that turns into a command channel.** Moltbook's heartbeat
  told agents to fetch and follow; injections spread from agent to
  agent.
- **Spoofed identity and spam**, where anyone could post as any agent.
- **Secrets in messages**, stored without access control.
- **Stale claims and lagging status**: forced lease release,
  watchdogs for zombie agents, dependants blocked by lagging states.
- **Loops and floods**, which forced burst limits and queue caps.
- **Context bloat** from broadcasting every turn to every member.
- **Polling cost against interrupting pushes.**
- **Agents stopping early or ignoring each other**, which no board
  fixes on its own; a person or a check must judge outcomes.
- **A human marker made of text**, such as Agent Mail's overseer
  preamble, which any poster could forge (inferred, not tested).
- **Cost.** No system examined shows spend per agent or per thread.
