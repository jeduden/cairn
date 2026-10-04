# Persona review, round 10: blind review of pitch v12

Round 10 gave all nine personas the text of pitch v12 and nothing
else, in up to three loops. Each said whether it would try Cairn, what
lands, what is unclear or overclaimed, three rewrites and one cut.

## Loop 1: the first v12 draft

| Seat                    | Try it? | What lands best                                               |
| ----------------------- | ------- | ------------------------------------------------------------- |
| Fleet developer         | Partly  | What needs you, ranked; pins back after compaction            |
| Returning owner         | Partly  | Catch-up lines linked to the record; the ranked queue         |
| Reviewer                | Partly  | Every commit knows its room; links to diff and log lines      |
| Live collaborator       | Partly  | Every message under its agent's name; follow any player       |
| OSS maintainer          | Partly  | Everything else in a room is untrusted                        |
| Multi-machine developer | Partly  | No central service; commit links outlive the room's machine   |
| Platform operator       | Partly  | Shows every change, removes cleanly; budgets                  |
| Security officer        | Partly  | Cross-room posts pull-only; the residual risk stated honestly |
| Agent                   | Partly  | Cross-room posts pull-only; pins back after compaction        |

| #      | Finding                                                                                                   | Seats                         | Change in loop 2                                                                                 |
| ------ | --------------------------------------------------------------------------------------------------------- | ----------------------------- | ------------------------------------------------------------------------------------------------ |
| P10-1  | Agents' pins seem to come back to every agent like the person's rules: agent text arriving as instruction | security, agent               | Only your pins come back word for word; agents' pins are notes, marked by author, read on demand |
| P10-2  | The browser reads as the only surface; the terminal sits in the last clause                               | fleet                         | The terminal is named beside the browser at the start                                            |
| P10-3  | "Speak once and every agent hears you" is a trust hole once a guest is in the room                        | collaborator                  | "All your agents hear you"                                                                       |
| P10-4  | Where review happens is unsaid; it could happen twice                                                     | reviewer                      | Review and land on the forge as always                                                           |
| P10-5  | The commit link: to what host, and does a squash keep it?                                                 | reviewer, OSS, owner, machine | The link is in the message, which a squash keeps                                                 |
| P10-6  | No search, no exact recall                                                                                | owner, agent                  | "Find it again" added                                                                            |
| P10-7  | Catch-up is per room                                                                                      | owner                         | One page across all rooms                                                                        |
| P10-8  | Integrity and redaction are not mentioned                                                                 | owner, security, OSS          | Append-only record; secrets redacted before writing                                              |
| P10-9  | "Connects to nothing" beside a localhost screen                                                           | security, operator            | "Opens no connection off your machine; listens on localhost only"                                |
| P10-10 | "Never slows your harness" is an unbacked absolute                                                        | fleet, operator, owner, agent | Every hook has a deadline and fails open                                                         |
| P10-11 | Trust by display name or by key?                                                                          | security, OSS                 | "By key"                                                                                         |
| P10-12 | Own machines peering is missing                                                                           | machine                       | Laptop, server and sandboxes share rooms, next                                                   |
| P10-13 | Overlap warning: before or after?                                                                         | fleet                         | "The moment a second agent edits a file another has edited"                                      |

Not taken, with the reason:

- **Evidence classes on runs** (reviewer, owner): the stakeholder
  dropped the recorded-versus-claim line in v11 (history item 24).
- **Team rollout, tenancy, purge** (operator): the pitch is for the
  developer; the operator's path is in the SRS.
- **Cutting "follow any player"** (reviewer, agent): the stakeholder
  named it the heart of multiplayer.
- **A lane file for strangers** (OSS): export is P2 and not in the
  room model's pitch yet.
