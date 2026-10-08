Feature: Peer network (PEER)

  Scenarios for SRS §5.14, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @PEER-01 @P2 @I4 @I9 @pending
  Scenario: sync runs only in the peer component, and only while the act turning it on stands
    Given an isolated Cairn home
    And an ephemeral node whose environment carries the person's write-once environment variable for the peer component, and whose access token, minted by the person's widening principal act, names a git-carrier remote
    When the node starts and the core runs its hook handlers and "cairn status --json"
    Then no core process starts the peer component, in-process or as a child
    And the ephemeral node's own entrypoint starts the peer component, and the publish component for the git-carrier remote its access token names, on the strength of the person's environment variable
    And the environment variable is recorded as a structural event, not a principal act, and audited
    And on a home where the CLI recorded no act turning it on, the peer component stays off, even when the person or a service manager starts its own entry point
    And with the peer component absent or stopped the core behaves exactly as with no peer component
    And all of the above is true whether the peer component ships in the core's executable or its own

  @PEER-02 @P2 @I4 @pending
  Scenario: a peer holds complete room copies and serves them only to nodes whose principal has a seat in the room or that hold it blind
    Given an isolated Cairn home
    And enrolled peers "a", "b" and "c", where the principals of "a" and "b" have seats in room "room-1" and the principal of "c" has none
    When "b" and "c" each ask "a" directly for the segments of "room-1"
    Then "a" serves its complete copy of "room-1" to "b"
    And "c" is refused every segment of "room-1"
    And a blind peer that holds "room-1" is served its segments, encrypted (PEER-12)
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
    When the hook handlers on "a" append an event to a run seat's writer
    Then the harness session's MCP server seals the run seat's writer within 2 s of that unsealed append, covering what the hook handler appended, and the event is carried as part of a sealed range
    And an event after the newest seal stays unsigned and is not carried
    And it appears in the room view of "b" and of "c" within 5 s of being written
    And it reaches "c" across at most one relay hop through an enrolled peer

  @PEER-05 @P2 @I1 @I6 @pending
  Scenario: an ephemeral node offers its sealed tail often and a retired writer's lost tail shows as a missing range
    Given an isolated Cairn home
    And an ephemeral node connected to a peer and running an agent
    When the run is active for 65 s, passes "Stop", "SubagentStop" and "SessionEnd", and the ephemeral node's access token expires
    Then the open segment was sealed and offered at each of those hooks and at least every 30 s, each sealed range as soon as it was sealed
    And once its clock passes the expiry, the node that minted the access token records the expire act ending it, naming the writer's last accepted seq
    And the writer is marked retired by that expire act, and no derivation reads a clock
    And before PRV-10 ships, the minting node instead refuses every later use of the expired access token
    And its segments past that seq that continue its chain without a fork are accepted and marked delivered after retirement, and only a revocation would refuse them
    And a tail lost after the last seq received is shown as a missing range, never as a quiet end or as "behind"

  @PEER-06 @P2 @I6 @I8 @pending
  Scenario: enrollment verifies keys on both nodes and an access token for an ephemeral node is scoped, carried and audited
    Given an isolated Cairn home
    And the person mints, as a widening principal act, an access token for an ephemeral node carrying repository "r"'s identity, continuing room "room-1" with an expiry, and a token key the person's device key certified, limited to the access token's rooms, the node's own personal room and the expiry, read from an environment secret by a configuration setting under ADM-04
    When an ephemeral node starts with the access token before it reaches any peer
    Then the node certifies its own seat keys with the token key, so each chains through the token key and the device key to the person's principal key, and knows every peer network address and git-carrier remote it may deliver to
    And a seat key it certifies for a room outside the access token's rooms other than the node's own personal room, or after the access token's expiry, chains to no principal key and is refused
    And its restore block carries the room's qualifying pins (PIN-10) from the access token as signed events, device-seat pins of provenance "operator" and run-seat pins of provenance "assistant", and says later pins may be missing
    And the node, with only the token key, has a device seat its token key certified, which signs no principal acts and no expire acts, and every pin the node writes, from its device seat or a run seat, restores only once a principal stamps it from one of its devices whose device scope allows it, and no trust grant covers its posts or pins
    And "cairn status" names where the access token is read from and no child process inherits the access token in its environment
    And the access token's issue, use, rotation and revocation are audited, and the issuing node shows an unused access token as "enrolled, never synced"
    And revoking the access token with "cairn access-token revoke" is a cut principal act, since it removes no pin from a restore block
    And discovery alone enrolls no peer, while enrolling one verifies the key on both nodes by matching words or a scanned code

  @PEER-07 @P2 @I2 @I8 @pending
  Scenario Outline: a relayed segment is accepted only when its seat key chains to a principal key in its key set
    Given an isolated Cairn home
    And a peer with the principal key of "alice" in its key set
    When a relayed segment arrives whose seat key <key>
    Then the segment is <expected>

    Examples:
      | key                                                   | expected                          |
      | chains to the principal key of "alice"                | accepted                          |
      | chains to a principal key not in the peer's key set   | refused                           |
      | was revoked, for events it sealed past its revocation | refused under the revocation rule |

  @PEER-08 @P2 @I4 @pending
  Scenario: the git carrier carries encrypted segments, one entry per writer, on the node's principal's remote
    Given an isolated Cairn home
    And a node of "alice" holding room "room-1", which she owns, where "alice", as the node's principal, enabled the git carrier for a remote of her own and, as the room's owner, for room "room-1", each a widening principal act
    And room "room-2" on that node, whose owner has not enabled the git carrier for it
    And no peer is reachable
    When the publish component carries the sealed segments of room "room-1"
    Then each writer's segments go to one entry in the namespaced location of "alice"'s remote that "alice" enabled
    And each segment is encrypted to the device keys of the room's principals, a token-key-only node's token key in place of one, and never to a seat key
    And a reader of the remote sees only entry names, sizes and times, and the git carrier says so
    And no segment of room "room-2" goes to the remote

  @PEER-09 @P2 @I2 @I8 @pending
  Scenario: presence hints and typing hints are ephemeral and drafts stay private to their author
    Given an isolated Cairn home
    And two connected seats in room "room-1", whose room settings, set by its owner's principal act, allow typing hints but not live drafts
    When one seat types a draft
    Then the other's room view shows the first seat's presence hint and typing hint, attributed only to the key that authenticated the connection, the peer's device key or a token-key-only node's token key
    And no presence hint or typing hint is stored in the record
    And the draft is not sent to the other seat

  @PEER-10 @P2 @I6 @I10 @pending
  Scenario: a writer that seals two different events at one seq is marked equivocated
    Given an isolated Cairn home
    And a writer that sealed two different events at seq 42
    When both events reach the peer
    Then both events are kept for forensics
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
    And the room view says "b" kept its copy
    And a quarantine request travels to the peers the same way, and a peer's principal applies it by a principal act of the class of the quarantine it applies

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
    And "carol"'s node, until it receives the bar, shows the missing range in the moderator's writer beside the room's membership
    And the room view says "bob" keeps what his node already holds
