Feature: Peer network (PEER)

  Scenarios for SRS §5.14, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @PEER-01 @P2 @I4 @I9 @pending
  Scenario: peering runs only in its own peer component, started by the person
    Given an isolated Cairn home
    And an ephemeral node in a sandbox whose environment carries the person's write-once peering setting
    When the node starts and the core runs its hook handlers and "cairn status --json"
    Then no core process starts the peer component, in-process or as a child
    And the sandbox's own entrypoint starts the peer component on the strength of the person's setting
    And on a home with no such action of the person the peer component stays off
    And with the peer component absent or stopped the core behaves exactly as in standalone
    And the outcome is the same whether the peer component ships in the core's executable or its own

  @PEER-02 @P2 @I4 @pending
  Scenario: a peer holds complete room copies and serves them only to nodes whose principal has a seat in the room
    Given an isolated Cairn home
    And enrolled peers "a", "b" and "c", where the principals of "a" and "b" have seats in room "room-1" and the principal of "c" has none
    When "b" and "c" each ask "a" directly for the segments of "room-1"
    Then "a" serves its complete copy of "room-1" to "b"
    And "c" is refused every segment of "room-1"
    And no other peer and no third-party service took part in enrollment, discovery or relay

  @PEER-03 @P2 @I9 @I10 @pending
  Scenario: peers exchange sealed ranges and derive identical state whatever the arrival order
    Given an isolated Cairn home
    And two peers receiving the same closed segments and sealed prefix of an open one, in different orders and partitions
    And a third node cut off from every peer
    When both peers rebuild their room state and index
    Then the room state and index are byte-identical on both peers
    And only sealed ranges of writer logs were exchanged
    And the cut-off node kept appending to its own writers without slowing its agents

  @PEER-04 @P2 @I9 @pending
  Scenario: an event written on one connected peer reaches every connected peer within 5 s
    Given an isolated Cairn home
    And connected peers "a", "b" and "c", where "c" reaches "a" only through "b"
    When the hook handlers on "a" append an event to a run seat's writer and the run's MCP server then serves a call
    Then that call seals the run seat's writer, covering what the hook handler appended, and the event is carried as part of a sealed range
    And an event after the newest seal stays unsigned and is not carried
    And it appears in the room view of "b" and of "c" within 5 s of being written
    And it reaches "c" across at most one relay hop through an enrolled peer

  @PEER-05 @P2 @I1 @I6 @pending
  Scenario: an ephemeral node offers its sealed tail often and a retired writer's lost tail shows as a gap
    Given an isolated Cairn home
    And an ephemeral node connected to a peer and running an agent
    When the run is active for 65 s, passes "Stop", "SubagentStop" and "SessionEnd", and the node's access token expires
    Then the open segment was sealed and offered at each of those hooks and at least every 30 s, each sealed range as soon as it was sealed
    And the writer is marked retired
    And its later segments that continue its chain without a fork are accepted and marked delivered after retirement, and only a revocation would refuse them
    And a tail lost after the last seq received is shown as a gap, never as a quiet end or as "behind"

  @PEER-06 @P2 @I6 @I8 @pending
  Scenario: enrollment verifies keys on both nodes and an access token for a sandbox is scoped, carried and audited
    Given an isolated Cairn home
    And the person mints, as a widening principal act, an access token for a sandbox carrying repository "r"'s identity, continuing room "room-1" with an expiry, and a token key the person's device key certified, limited to the access token's rooms and expiry, read from an environment secret by recorded opt-in
    When a node in a sandbox starts with the access token before it reaches any peer
    Then the node certifies its own seat keys with the token key, so each chains through the token key and the device key to the person's principal key, and knows every peer address and git-carrier remote it may deliver to
    And a seat key it certifies for a room outside the access token's rooms, or after the access token's expiry, chains to no principal key and is refused
    And its restore block carries the room's qualifying pins (PIN-10) from the access token as signed events of provenance "operator" and says later pins may be missing
    And the node, holding only the token key, has a device seat its token key certified, which signs no principal acts and no expire acts, and a pin its runs write restores only once a principal stamps it from one of its devices
    And "cairn status" names where the access token is read from and no child process inherits the access token in its environment
    And the access token's issue, use, rotation and revocation are audited, and the issuing node shows an unused access token as "enrolled, never synced"
    And discovery alone enrolls no peer, while enrolling one verifies the key on both nodes by matching words or a scanned code

  @PEER-07 @P2 @I2 @I8 @pending
  Scenario Outline: a relayed segment is accepted only when its seat key chains to a trusted principal key
    Given an isolated Cairn home
    And a peer that trusts the principal key of "alice"
    When a relayed segment arrives whose seat key <key>
    Then the segment is <outcome>

    Examples:
      | key                                                   | outcome                           |
      | chains to the principal key of "alice"                | accepted                          |
      | chains to a principal key the peer does not trust     | refused                           |
      | was revoked, for events it sealed past its revocation | refused under the revocation rule |

  @PEER-08 @P2 @I4 @pending
  Scenario: the git carrier carries encrypted segments, one entry per writer, on the node's principal's remote
    Given an isolated Cairn home
    And a node of "alice" holding room "room-1", which she owns, where "alice" enabled the git carrier with a remote of her own, a widening principal act
    And no peer is reachable
    When the publish component carries the sealed segments of room "room-1"
    Then each writer's segments go to one entry in the namespaced location of "alice"'s remote that "alice" enabled
    And each segment is encrypted to the device keys of the room's principals, a token-key-only node's token key in place of one, and never to a seat key
    And a reader of the remote sees only entry names, sizes and times, and the carrier says so

  @PEER-09 @P2 @I2 @I8 @pending
  Scenario: presence hints and typing hints are ephemeral and drafts stay private to their author
    Given an isolated Cairn home
    And two connected seats in room "room-1", whose room settings, set by its owner's principal act, allow typing hints but not live drafts
    When one seat types a draft
    Then the other's room view shows the typist's presence hint and typing hint, attributed only to the enrolled key that authenticated the connection
    And no presence hint or typing hint is stored in the record
    And the draft is not sent to the other seat

  @PEER-10 @P2 @I6 @I10 @pending
  Scenario: a writer that seals two different events at one seq is marked equivocated
    Given an isolated Cairn home
    And a writer that sealed two different events at seq 42
    When both events reach the peer
    Then both events are kept as evidence
    And the writer is marked "equivocated"
    And the room shows its integrity status as "equivocated"
    And the derived artifacts for that writer stop at the fork until the node's principal chooses which fork to keep as a widening principal act

  @PEER-11 @P2 @I5 @I6 @pending
  Scenario: erasure and quarantine requests reach every peer signed, and each peer's state is shown
    Given an isolated Cairn home
    And room "room-1" held by enrolled peers "a", "b" and "c"
    When the node's principal purges a range of "room-1", so the node sends its purge to its peers as an erasure request, and the principal of "a" applies it by a widening principal act, "b" refuses it and "c" is unreachable
    Then the erasure request was sent to every enrolled peer holding the room as a signed event of provenance "operator"
    And "a" erased or tombstoned every copy of the range it holds in any writer's log
    And each peer's state, applied, refused or unreachable, is audited, counted and shown
    And the view says "b" kept its copy
    And a quarantine request travels to the peers the same way

  @PEER-12 @P2 @I4 @I8 @pending
  Scenario: a blind peer stores and serves a room it cannot read
    Given an isolated Cairn home
    And a room where "alice" and "bob" have seats
    And a node of another principal, enrolled as a blind peer
    When the nodes of "alice" and "bob" sync the room's sealed ranges through the blind peer
    Then the blind peer stores only ranges encrypted to the device keys of the room's principals, never to a seat key
    And it verifies the seat key's signature over each range before storing it
    And it holds no event content, header field, commitment key or room metadata
    And it derives no room state and has no seat in the room
    And every surface marks it as a blind peer

  @PEER-13 @P2 @I6 @I8 @pending
  Scenario: a bar takes effect at once on its node and stops future segments
    Given an isolated Cairn home
    And a room owned by "alice", shared between her node and the nodes of "bob" and "carol"
    When a moderator's seat on "alice"'s node bars "bob"'s principal key while "carol"'s node is unreachable
    Then the bar takes effect on "alice"'s node at once
    And "alice"'s peer component sends no further segments of the room to any key the bar covers
    And "carol"'s node, until it receives the bar, shows the gap in the moderator's writer beside the room's membership
    And the view says "bob" keeps what his node already holds
