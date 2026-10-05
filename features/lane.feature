Feature: Room (LANE)

  Scenarios for SRS §5.11, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @LANE-01 @P0 @I1 @I10 @pending
  Scenario: every event belongs to exactly one room derived from the record
    Given an isolated Cairn home
    And a session on branch "main" that switches to branch "feature/x", which no room of the project holds
    And a second node that created a room for "feature/x" concurrently
    When the session's hooks run and the two nodes' records are merged
    Then every event carries the project identity and exactly one room id, minted as 128 random bits with no Cairn command
    And the events after the branch observation belong to the room of "feature/x" with no owner act
    And the two rooms for "feature/x" merge in derived state into the lower id, keeping both creation events
    And every reference to the merged id resolves to the surviving room

  @LANE-02 @P0 @I1 @I6 @I8 @pending
  Scenario Outline: a project's identity is independent of the local path
    Given an isolated Cairn home
    And a repository clone that <clone>
    When the first hook event for the clone runs
    Then the project identity is <identity>
    And no identity is minted from the local path
    And no name Cairn derives from the identity for its local state reveals anything about the identity off this node
    And a later change in the identity the directory resolves to is audited, starts no new store, and rebinds only on "cairn project bind"

    Examples:
      | clone                                                           | identity                              |
      | contains the bound parentless commit of the default branch      | the bound identity                    |
      | is shallow, lacks the bound commit and holds an enrolment token | the identity from the enrolment token |
      | has no commit                                                   | a provisional, node-local identity    |

  @LANE-03 @P1 @I2 @pending
  Scenario: every event names an actor derived from its writer and source
    Given an isolated Cairn home
    And a subagent event whose text claims to come from the human principal "alice"
    And a pin added by "alice" through "cairn pin add"
    When both events are recorded
    Then the subagent event names the agent session and its subagent identity as actor
    And the pin event names the human principal "alice" as actor
    And no actor is taken from event content

  @LANE-04 @P1 @I2 @I10 @pending
  Scenario: files changed and commands run are derived from structural fields
    Given an isolated Cairn home
    And a room whose session edits two files through tool calls and runs a command that exits 1
    And a third file changed only between two worktree checkpoints
    And an assistant message claiming an edit to a fourth file
    When the room's changes and runs are derived
    Then the files changed and the command with exit status 1 are listed per session and per room, each bound to its address range
    And each diff hunk names the event, actor and preceding message that produced it, and the third file's hunk is marked "from checkpoint"
    And the fourth file is not listed, and "cairn rebuild" derives the same result

  @LANE-05 @P1 @I2 @I10 @pending
  Scenario Outline: each result carries exactly one evidence class from structural events
    Given an isolated Cairn home
    And a room whose result rests on <source>
    When the room's results are derived
    Then the result carries the evidence class "<class>" and no other
    And the result's binding is "<binding>"
    And the class is derived from structural events only, never from event text

    Examples:
      | source                                                                                                  | class       | binding |
      | an assistant message stating the tests pass                                                             | claim       | —       |
      | tool output alone                                                                                       | claim       | —       |
      | a command, its exit status and its tree recorded by a hook on the room's own node                       | own run     | bound   |
      | a command recorded by a hook after edits made through a shell                                           | claim       | unbound |
      | the command run through the run component on a fresh checkout of the exact commit by an uninvolved node | witness run | —       |
      | a check result for the exact commit signed by an enrolled CI key and brought by the CI carrier          | CI attested | —       |

  @LANE-06 @P1 @I6 @I10 @pending
  Scenario Outline: the rooms behind a landed commit carry one proof class
    Given an isolated Cairn home
    And a room whose recorded change <relation> a landed commit
    When the rooms behind the landed commit are derived from the record and the local clone
    Then the room is listed with proof class "<proof>"
    And the link carries the reason "<reason>"
    And the link counts as proven: <proven>
    And no process is started to read git objects

    Examples:
      | relation                            | proof       | reason            | proven |
      | lands as the same commit in         | same commit | —                 | yes    |
      | lands as the same patch in          | same patch  | —                 | yes    |
      | lands as the same tree in           | same tree   | —                 | yes    |
      | is named only by a trailer in       | asserted    | —                 | no     |
      | was rewritten out of history before | not proven  | history-rewritten | no     |
      | was made outside the record before  | not proven  | outside-record    | no     |

  @LANE-07 @P2 @I2 @I6 @I10 @pending
  Scenario: the landing gate counts only signed human verdicts and required checks under the target branch's policy
    Given an isolated Cairn home
    And a room whose target branch policy requires one approver key and a "witness run" check, while the room's head carries a looser policy
    And an agent's approval, a human approval signed under the owner act, a later head move and an "own run" result
    When the landing gate is evaluated
    Then the policy is read from the target branch, never from the room's head
    And the agent's verdict is shown as a comment and does not count
    And the human approval is stale after the head moved
    And the required check stays unsatisfied by the "own run" result
    And Cairn flags the landing and neither pushes nor merges

  @LANE-08 @P2 @I2 @I4 @I6 @pending
  Scenario: a configured forge's branch protection is authoritative and its verdicts are shown beside Cairn's
    Given an isolated Cairn home
    And a room with a forge configured whose branch protection requires one review
    And a Cairn approval, a forge approval under a policy that hands approvals to the forge, and a failed forge fetch
    When the room view is shown
    Then the forge's branch protection governs landing
    And the Cairn approval is labelled as not counting toward the forge's required reviews and shown beside the forge's verdict
    And the forge approval, imported as an untrusted event through the forge bridge, is shown as "asserted" with its forge and time
    And the landing it covers is not flagged "landed-not-approved"
    And the failed fetch is counted and shown with its time

  @LANE-09 @P2 @I10 @pending
  Scenario: concurrent room metadata edits merge to the same value on every node
    Given an isolated Cairn home
    And two nodes that concurrently set one room's title to "Alpha" and "Beta" as events
    When each node merges the other's events
    Then both nodes derive the same title by the documented conflict-free rule
    And the losing title stays visible in the room's history

  @LANE-10 @P2 @I2 @I6 @pending
  Scenario: an invite names key and role, is reviewed before it takes effect, and roles are enforced
    Given an isolated Cairn home
    And a room owned by the operator
    When the owner invites the key "bob" as "watcher"
    Then the invite names the key "bob" and the role "watcher"
    And before the invite takes effect a review step shows which classes and ranges will replicate to the invitee's node, with SEC-08 applied
    And the joiner's first view shows the room's goal, pins, state, pending requests and latest results within 3 s of connecting, before the full sync completes
    And the node refuses and audits a post from "bob", which the watcher role does not permit
    And a request from "bob" for a wider role enters the owner's Needs you as Q3

  @LANE-11 @P2 @I2 @I6 @I10 @pending
  Scenario: an accepted handover moves ownership and keeps the former owner's agents and pins working
    Given an isolated Cairn home
    And a room owned by "alice" with a pin that restores to her agents
    And "alice" offers the room to "bob" with a signed event after a presence check
    When "bob" accepts with a signed event after a presence check
    Then the handover shows as "accepted" to both parties
    And "alice" holds the co-author role and her agents' events stay accepted
    And her pin keeps restoring to her agents and reaches "bob"'s agents only after "bob" re-signs it
    And pending requests stay with each agent's principal

  @LANE-12 @P2 @I6 @pending
  Scenario: only requests addressed to an agent enter its principal's endorse queue
    Given an isolated Cairn home
    And a room with a discussion post addressed to people, a request addressed to an agent, and a reviewer's request for changes
    When the posts are delivered
    Then only the request enters the endorse queue of the target agent's principal
    And the request for changes creates a request addressed to the room owner's agents
    And each request shows its writer exactly one state: delivered, endorsed or dismissed
    And an endorsed request names the endorsing principal and shows any edit as a diff against the post

  @LANE-13 @P1 @I6 @pending
  Scenario: an edit to a file another open room has edited raises a Needs you item on both rooms
    Given an isolated Cairn home
    And two open rooms "a" and "b" on one node, where room "a" has edited "notes.txt"
    When room "b" edits "notes.txt"
    Then a Needs you item is raised on both rooms at that edit, naming the other room and "notes.txt"
    And the item clears on the owner's acknowledgement
    And a later edit of "notes.txt" raises no new item, while an edit of a file not yet acknowledged does

  @LANE-14 @P1 @I6 @pending
  Scenario: every agent turn records its trigger and token use
    Given an isolated Cairn home
    And a room agent session
    And a prompt from its principal, an endorsed request and a room post nobody endorsed
    When the agent's turns run
    Then each turn records its trigger as harness_meta: the principal's prompt, or the owner act with the endorsement's source
    And each turn records its token use, so spend is attributable per agent and per trigger
    And the post nobody endorsed triggers no turn

  @LANE-15 @P2 @I2 @I6 @pending
  Scenario: a foreign room view states what is asserted and verifies the contributor's binding
    Given an isolated Cairn home
    And a foreign room bundle whose pull request commits are signed by the contributor's existing commit-signing identity, which also signed a binding statement naming the bundle's owner key
    And a writer key in the bundle that does not chain to the owner key, and flags carried by the bundle
    When the operator imports the bundle and opens the foreign room view
    Then the view states that every event, evidence class and proof mark in it is asserted by the publisher's key
    And the commits show as a match for the owner key, verified offline within the core's boundary, with no program started and no connection opened
    And the writer key that does not chain is shown unbound, by its fingerprint
    And PRV-07 flags are computed locally, the bundle's flags are ignored, and hidden characters are shown in place

  @LANE-16 @P2 @I2 @I6 @pending
  Scenario Outline: each room role permits exactly its rights
    Given an isolated Cairn home
    And a room member with role "<role>"
    When the member opens the room view
    Then the view shows the role "<role>" and the rights "<rights>"
    And the node accepts exactly those acts from the member and refuses and audits any other

    Examples:
      | role      | rights                                                                   |
      | owner     | every owner act on the room                                              |
      | co-author | posts, their own agents' events, comments, endorsing to their own agents |
      | reviewer  | posts, comments, approvals, requests for changes                         |
      | watcher   | reading, a request for a wider role                                      |

  @LANE-17 @P1 @I6 @I8 @pending
  Scenario: every room shows its visibility
    Given an isolated Cairn home
    And a private room, a room shared with two members, a published room and a room stored on a blind peer
    When the person opens the room view and runs "cairn rooms"
    Then each room shows its visibility on both surfaces
    And a shared room lists each member's petname and role
    And changing a room's visibility is recorded as an owner act

  @LANE-18 @P2 @I2 @I8 @pending
  Scenario: an invite link binds once and reveals nothing early
    Given an isolated Cairn home
    And the owner issued an invite link with the role "reviewer" and an expiry
    When a person opens the link for the first time
    Then the token binds to that person's key
    And the invite takes effect only after the owner's review step
    And no room content is revealed before it does
    And a second use of the link, or a use after its expiry, is refused and audited

  @LANE-19 @P1 @I1 @I6 @pending
  Scenario: subagents sharing a worktree are told apart
    Given an isolated Cairn home
    And an orchestrating agent whose two subagents edit the same worktree at once
    When both edit "internal/store/fts.go" and one changes another file through a shell
    Then each tool-call edit is attributed to its subagent
    And the shell change's checkpoint hunk is marked ambiguous, naming both subagents
    And a Needs you item names both subagents and the file
    And a run marked unbound names the edits that unbound it

  @LANE-20 @P1 @I2 @I3 @pending
  Scenario: the owner's intent is the room's first pin, versioned and restored word for word
    Given an isolated Cairn home
    And a room whose owner set the intent "add CSV export" with criteria "C1 exports every column" and "C2 keeps the header row"
    And an agent of the room proposed a criterion "C3 streams large files"
    When the owner revises C2 and the agent's context compacts
    Then the intent is stored as the room's first pin, of type "intent", at the highest priority
    And the revision is recorded as the removal of the first version's pin followed by the addition of the second, with its version and its diff against the first
    And the restore block carries the second version word for word with its version, among the active pins and nowhere else
    And C3 stays an inactive, untrusted candidate until the owner adopts it, exactly as shown

  @LANE-21 @P1 @I2 @I10 @pending
  Scenario: every result traces to the intent it was produced under
    Given an isolated Cairn home
    And a room whose intent names criteria C1 and C2 and the path "internal/export/"
    When an agent links its test run to C1 through "room_link" and edits "go.mod"
    Then the run names the intent version in force when its turn began
    And the link to C1 reads as a claim
    And no result is linked to C2 from event text
    And the edit to "go.mod" is marked "outside intent"

  @LANE-22 @P2 @I2 @I10 @pending
  Scenario: several people judge one outcome and only the owner changes the intent
    Given an isolated Cairn home
    And a room shared by its owner, a reviewer and a co-author
    When the reviewer records "not met" on C1, the owner records "met" on C1 and the co-author posts a revised criterion
    Then every member sees both verdicts on C1 side by side, each with its judge's petname and role
    And neither verdict replaces the other
    And the co-author's revision reaches no agent until the owner adopts it
