# Research

Evidence behind Cairn's design decisions, kept verbatim. Each report
answers one question and cites its sources. The notes under
[notes](notes/) are the raw, sourced findings the reports were written
from.

## Reports

- [Agent session storage beyond git](reports/agent-session-storage-beyond-git.md):
  the merged answer. Cairn owns an append-only record and lets git
  carry copies. Code stays on git.
- [Git as agent session storage](reports/git-as-agent-session-storage.md):
  20 systems compared on storing sessions in git, with twelve design
  rules.
- [Custom storage, git and chat interfaces](reports/custom-storage-git-and-chat-interfaces.md):
  Cairn owns the record and its keys. Git and chat are opt-in views
  outside the shipped binary, and shredding works by destroying keys.
- [Interesting ideas digest](reports/interesting-ideas-digest.md):
  the top ideas from a sweep of about 175 systems.

## Notes

- [Git as agent session storage](notes/git-as-agent-session-storage/):
  Beads and Gas Town, session-in-repo tools, git-native collaboration
  data, git's limits and real time.
- [Git alternatives for agent sessions](notes/git-alternatives-for-agent-sessions/):
  Cursor Origin, agent-native version control, Entire read from
  source, Hacker News, and per-system lessons.
- [Version control beyond git](notes/version-control-beyond-git/):
  non-git systems, organisations that left git at scale, and
  migration paths and the debate.
- [Custom storage, git and chat](notes/custom-storage-git-and-chat/):
  platforms that combine git, chat and encryption, git interfaces
  over custom storage, chat protocols and their deletion semantics,
  and crypto-shredding in law and practice.
- [Agent session storage sweep](notes/agent-session-storage-sweep/):
  the catalog of 308 systems, with storage-pattern counts, cross-cutting
  findings and lessons, plus every fact sheet in `sheets.json`.
- [Live-PR pitch review](notes/live-pr-pitch-review/): a blind
  adversarial review of the "live pull request" pitch in three lenses:
  missing process, existing products, and inconsistencies with the
  contract and the evidence.
