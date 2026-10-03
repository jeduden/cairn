Feature: Peer network (PEER)

  Scenarios for SRS §5.14, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @PEER-01 @P2 @I4 @I9 @pending
  Scenario: peering runs only in its own cairn peer component, started by the tenant
    Given an isolated Cairn home
    And an ephemeral sandbox whose environment carries the tenant's write-once peering setting
    When the sandbox starts and the core runs its hooks and "cairn status --json"
    Then no core process starts "cairn peer", in-process or as a child
    And the sandbox's own entrypoint starts "cairn peer" on the strength of the tenant's setting
    And on a home with no such tenant action "cairn peer" stays off
    And with "cairn peer" absent or stopped the core behaves exactly as in standalone

  @PEER-02 @P2 @I4 @pending
  Scenario: a peer holds complete lane copies and serves them only to the lane's members
    Given an isolated Cairn home
    And enrolled peers "a", "b" and "c", where "a" and "b" are members of lane "l-1" and "c" is not
    When "b" and "c" each request the segments of "l-1" from "a" directly
    Then "a" serves its complete copy of "l-1" to "b"
    And "c" is refused every segment of "l-1"
    And no other peer and no third-party service took part in enrollment, discovery or relay

  @PEER-03 @P2 @I9 @I10 @pending
  Scenario: peers exchange sealed ranges and derive identical state whatever the arrival order
    Given an isolated Cairn home
    And two peers receiving the same closed segments and sealed prefix of an open one, in different orders and partitions
    And a third writer cut off from every peer
    When both peers rebuild their lane state and index
    Then the lane state and index are byte-identical on both peers
    And only sealed ranges of writer logs were exchanged
    And the cut-off writer kept appending to its own log without slowing its agents

  @PEER-04 @P2 @I9 @pending
  Scenario: an event written on one connected peer reaches every connected peer within 5 s
    Given an isolated Cairn home
    And connected peers "a", "b" and "c", where "c" reaches "a" only through "b"
    When a hook on "a" writes an event and ends
    Then the event is sealed at the end of the hook and carried as part of a sealed range
    And it appears in the lane view of "b" and of "c" within 5 s of being written
    And it reaches "c" across at most one relay hop through an enrolled peer

  @PEER-05 @P2 @I1 @I6 @pending
  Scenario: an ephemeral node offers its sealed tail often and a retired writer's lost tail shows as a gap
    Given an isolated Cairn home
    And an ephemeral node connected to a peer and running a session
    When the session runs for 65 s, passes "Stop", "SubagentStop" and "SessionEnd", and its writer token expires
    Then the open segment was sealed and offered at each of those hooks and at least every 30 s, each sealed range as soon as it was sealed
    And the writer is marked retired
    And its later segments that continue its chain without a fork are accepted and marked delivered after retirement, and only a revocation would refuse them
    And a tail lost after the last seq received is shown as a gap, never as a quiet end or as "behind"

  @PEER-06 @P2 @I6 @I8 @pending
  Scenario: enrollment verifies keys on both nodes and a sandbox token is scoped, carried and audited
    Given an isolated Cairn home
    And the tenant mints, as a widening act, a sandbox token for repository "r" continuing lane "l-1" with an expiry, read from an environment secret by recorded opt-in
    When a sandbox starts with the token before it reaches any peer
    Then the sandbox certifies its own writer key from the token's bound identity, project delegation and expiry, and knows every peer address and git-carrier remote it may deliver to
    And its restore block holds the lane's active pins from the token as signed operator events and says later pins may be missing
    And "cairn status" names the token's source and no child process inherits the token in its environment
    And the token's issue, use, rotation and revocation are audited, and the issuing node shows an unused token as "enrolled, never synced"
    And discovery alone enrolls no peer, while enrolling one verifies the key on both nodes by matching words or a scanned code

  @PEER-07 @P2 @I2 @I8 @pending
  Scenario Outline: a relayed segment is accepted only when its writer key chains to a trusted owner key
    Given an isolated Cairn home
    And a peer that trusts the owner key "o-1"
    When a relayed segment arrives whose writer key <key>
    Then the segment is <outcome>

    Examples:
      | key                                                   | outcome                           |
      | chains to "o-1"                                       | accepted                          |
      | chains to an owner key the peer does not trust        | refused                           |
      | was revoked, for events it sealed past its revocation | refused under the revocation rule |

  @PEER-08 @P2 @I4 @pending
  Scenario: the git carrier carries encrypted segments as one ref per writer on the owner's remote
    Given an isolated Cairn home
    And the owner opted in to the git carrier with their own remote
    And no peer is reachable
    When "cairn publish" carries the sealed segments of lane "l-1"
    Then each writer's segments go to one ref under "refs/cairn/" on the owner's remote
    And each segment is encrypted to the enrolled keys of the lane's members
    And a reader of the remote sees only ref names, sizes and times, and the carrier says so

  @PEER-09 @P2 @I2 @I8 @pending
  Scenario: presence and typing hints are ephemeral and drafts stay private to their writer
    Given an isolated Cairn home
    And two connected participants in lane "l-1", which allows typing hints but not live drafts
    When one participant types a draft
    Then the other's lane view shows the typist's presence and typing, attributed only to the peer key that authenticated the connection
    And no presence or typing hint is stored in the record
    And the draft is not sent to the other participant

  @PEER-10 @P2 @I6 @I10 @pending
  Scenario: a writer that seals two different events at one seq is marked equivocated
    Given an isolated Cairn home
    And a writer that sealed two different events at seq 42
    When both events reach the peer
    Then both events are kept as evidence
    And the writer is marked "equivocated"
    And derived state for that writer stops at the fork until the owner chooses a branch as an owner act

  @PEER-11 @P2 @I5 @I6 @pending
  Scenario: purges and quarantines reach every peer as signed requests and each peer's state is shown
    Given an isolated Cairn home
    And lane "l-1" held by enrolled peers "a", "b" and "c"
    When the operator purges a range of "l-1", and "a" applies it, "b" refuses it and "c" is unreachable
    Then the purge was sent to every enrolled peer holding the lane as a signed operator event
    And "a" erased or tombstoned every copy of the range it holds in any writer's log
    And each peer's state, applied, refused or unreachable, is audited, counted and shown
    And the view says "b" kept its copy
    And a quarantine request travels to the peers the same way

  @PEER-12 @P2 @I4 @I8 @pending
  Scenario: a blind peer stores and serves a lane it cannot read
    Given an isolated Cairn home
    And a lane whose members are "owner" and "co-author"
    And a peer run by another entity, enrolled as a blind peer
    When the members' nodes sync the lane's sealed ranges through the blind peer
    Then the blind peer stores only ranges encrypted to the members' keys
    And it verifies the writer's signature over each range before storing it
    And it holds no event content, header field, commitment key or lane metadata
    And it derives no lane state and counts as no member
    And every surface marks it as a blind peer
