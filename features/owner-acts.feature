Feature: Owner acts (OWN)

  Scenarios for SRS §5.13, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @OWN-01 @P1 @I2 @pending
  Scenario: only the session's principal can instruct it
    Given an isolated Cairn home
    And an agent session started by tenant "tenant-a" on its node
    When tenant "tenant-b" writes an owner act addressed to that session
    Then the session's one principal is "tenant-a"
    And the act from "tenant-b" does not instruct the session

  @OWN-02 @P1 @I2 @I6 @pending
  Scenario Outline: an owner act is recorded only from an authenticated owner surface
    Given an isolated Cairn home
    And a running agent session
    When an owner act arrives from <surface>
    Then the outcome is "<outcome>"

    Examples:
      | surface                              | outcome                                                      |
      | the lane-view component under SEC-20 | recorded as an operator event covered by its writer's seal   |
      | the CLI under OWN-12                 | recorded as an operator event covered by its writer's seal   |
      | the harness's own prompt             | recorded as user or a harness_meta outcome, not an owner act |
      | the terminal the run component hosts | recorded as user or a harness_meta outcome, not an owner act |
      | any other surface                    | refused and audited                                          |

  @OWN-03 @P1 @I2 @pending
  Scenario: owner text reaches an agent only through the harness's input interface
    Given an isolated Cairn home
    And a running agent session whose harness offers an input interface
    And an untrusted event in the record
    When the owner sends a steer naming that event's id
    Then the steer reaches the agent through the harness's own input interface
    And no Cairn hook output carries the steer
    And the text sent is the owner-typed text or a fixed template that references ids
    And no field of the untrusted event is embedded in it

  @OWN-04 @P1 @I2 @pending
  Scenario: hook output carries only a computed permission decision with a templated reason
    Given an isolated Cairn home
    And owner-signed rules and grants covering an action
    When a hook reports a permission request for that action
    Then the hook output carries a decision of allow, ask, deny or defer from the rule engine
    And its reason comes from the fixed template set and references ids
    And the hook output carries no owner-typed text and no field of the record

  @OWN-05 @P1 @I6 @I10 @pending
  Scenario: the first valid answer to a held request wins and conflicts resolve to deny
    Given an isolated Cairn home
    And a permission request held with a stable id bound to the harness's request id
    When two owner surfaces answer it, one after the other and then concurrently with conflicting answers
    Then the first valid answer in causal order wins
    And the later answer is recorded as superseded
    And the concurrent conflicting answers resolve to deny
    And the conflict is recorded and shown

  @OWN-06 @P1 @I9 @I6 @pending
  Scenario Outline: Cairn holds a request only while an answer stays possible and never allows on failure
    Given an isolated Cairn home
    And a permission request from an agent
    And <situation>
    When the request is raised
    Then <outcome>

    Examples:
      | situation                                                            | outcome                                                                |
      | no away policy and the hold window passes                            | the request is not denied on the timeout                               |
      | no owner surface is connected and the harness prompt is unanswerable | the request is only mirrored, not held                                 |
      | the adapter cannot keep the harness's own prompt answerable          | the request is only mirrored and the answer is left to the harness     |
      | Cairn fails while holding the request                                | the harness falls back to its own prompt and the action is not allowed |

  @OWN-07 @P1 @I2 @I9 @pending
  Scenario Outline: an away policy only replies to the agent's own pending request
    Given an isolated Cairn home
    And the owner turned on the away policy "<policy>" for the lane as an owner act
    And an agent's permission request is held past the hold window
    When the hold window ends
    Then the agent's pending request gets the reply "<reply>"
    And no turn is started or resumed
    And a later approval reaches only the requesting session as the fixed template "Request <id> was approved by the owner" with a grant for the identical action
    And the held-request id resolves through get to the request's recorded event

    Examples:
      | policy                 | reply                                                                             |
      | keep going             | a deny with the fixed text "Held for the owner as <id>; continue with other work" |
      | pause at the first ask | the agent pauses                                                                  |
      | stop at the first ask  | the agent stops                                                                   |

  @OWN-08 @P2 @I2 @I6 @pending
  Scenario: an endorsement sends exactly the confirmed text inside a fixed template
    Given an isolated Cairn home
    And a post in the lane longer than its preview, holding hidden characters
    When a principal expands it, edits it and endorses it to one of their own agents
    Then Endorse was enabled only after the post was expanded
    And the agent receives exactly the confirmed text with hidden characters stripped and their count recorded
    And the endorsement is a signed owner act naming the post's commitment, writer key, target agent, original text and sent text
    And the post's writer is shown the edit as a diff
    And the text reaches the agent through the harness's input interface in a fixed template naming the writer key's short fingerprint and the post's recall address, with no petname

  @OWN-09 @P1 @I2 @pending
  Scenario Outline: nothing but a current owner act starts, resumes or sends text to an agent
    Given an isolated Cairn home
    And an idle agent session
    When <source> arrives for the session
    Then no turn is started or resumed
    And no text is sent to the agent

    Examples:
      | source                 |
      | a post                 |
      | a notice               |
      | an agent's message     |
      | a watchdog observation |

  @OWN-10 @P1 @I2 @I7 @pending
  Scenario: each action class has one rule level that only tightens
    Given an isolated Cairn home
    And the owner set the class "force pushes" to "ask first"
    And a repository file sets "force pushes" to "act without asking"
    When a recall-tainted child agent of a parent at that level attempts a force push
    Then the repository file does not loosen the level
    And the sensitive class is raised one level for the tainted session
    And the child holds no looser level than its parent
    And every rule change on record is an owner act

  @OWN-11 @P1 @I2 @I8 @pending
  Scenario Outline: an owner act is accepted according to its class
    Given an isolated Cairn home
    And an owner surface "<surface>"
    And OWN-22 "<risk state>" widening acts
    When the owner writes a "<class>" act there
    Then the act is "<outcome>"

    Examples:
      | surface                        | risk state      | class    | outcome                                                      |
      | authenticated                  | permits         | widen    | accepted                                                     |
      | authenticated                  | does not permit | widen    | refused                                                      |
      | authenticated                  | permits         | unlisted | treated as widening and accepted                             |
      | not backed by a presence check | permits         | cut      | recorded with a mark naming its surface, free text untrusted |
      | not backed by a presence check | permits         | neutral  | recorded with a mark naming its surface, free text untrusted |

  @OWN-12 @P1 @I2 @pending
  Scenario: a CLI verb writing an owner act refuses without a terminal
    Given an isolated Cairn home
    And a session on the node leaves a residual risk open with no recorded acceptance
    When the operator runs a widening owner-act verb at a terminal
    Then the verb refuses
    And the refusal names the open residual risks and the sessions that leave them open
    And the same verb with standard input or output not a terminal refuses before any other check

  @OWN-13 @P1 @I2 @pending
  Scenario: a steer arriving after its turn ended is not applied without confirmation
    Given an isolated Cairn home
    And the owner writes a steer naming the agent's current turn
    When the steer arrives after that turn ended
    Then the steer is not applied to another turn
    And the owner is asked to confirm before it is applied

  @OWN-14 @P1 @I6 @pending
  Scenario: a stop lists effects first and shows stopped only on acknowledgement
    Given an isolated Cairn home
    And a running agent session with completed, in-flight and waiting effects
    When the owner asks to stop it
    Then the view lists the completed effects, what is in flight and what is waiting, with whether and how each can be undone
    And no surface shows the harness as stopped before its acknowledgement is recorded

  @OWN-15 @P1 @I6 @I7 @pending
  Scenario: a control the adapter cannot honour is shown unavailable and never simulated
    Given an isolated Cairn home
    And Claude Code installed through plain hooks
    When the operator runs "cairn install"
    Then it offers, as a shown diff, an opt-in routing every harness launch through the run component
    And it states the controls the installed path lacks, including mid-turn steer, interrupt, stop and terminal takeover
    And each such control is shown unavailable with its reason and the launch path that offers it
    And no such control is simulated

  @OWN-16 @P1 @I2 @I8 @pending
  Scenario: a phone credential is minted by a widening act and limited to its scope
    Given an isolated Cairn home
    And a phone credential minted by a widening owner act
    When the phone tries to allow a held permission request for the whole session
    Then the lane-view component refuses it on the server
    And the phone can only read, allow once and deny held permission requests
    And where an authenticator is required each allow carries the phone's own presence check bound to that answer

  @OWN-17 @P2 @I2 @I8 @pending
  Scenario: an owner act from another of the owner's devices takes effect only within its scope
    Given an isolated Cairn home
    And a paired phone of the owner
    When the phone writes an owner act raising a rule level
    Then the act does not take effect here
    And the phone's acts are limited to allowing or denying held permission requests
    And acts from the owner's other nodes take effect only under PRV-10, within their device scope and maximum rule level

  @OWN-18 @P1 @I2 @pending
  Scenario: a command from an untrusted event runs only after the person confirms its exact text
    Given an isolated Cairn home
    And an untrusted event holding a command with hidden characters
    When the person confirms the command for a witness run
    Then the person was shown its exact text with hidden characters visible before confirming
    And the act is recorded as theirs
    And the witness run executes only through the run component, outside any agent context, on a fresh checkout of the exact commit, with network and the user's home denied
    And its command, exit status and tree hash are recorded

  @OWN-19 @P1 @I1 @I4 @pending
  Scenario: terminal takeover stays local and no-echo input is not stored
    Given an isolated Cairn home
    And a harness hosted by the run component on this machine
    When the owner takes over its terminal and types at a no-echo prompt
    Then the takeover runs only in the run component on the harness's machine
    And the no-echo input is not stored
    And no takeover from another machine is offered

  @OWN-20 @P1 @I2 @pending
  Scenario: hand-off raises an untrusted reason and hand-back sends only the note and checkpoint address
    Given an isolated Cairn home
    And an agent hands off with a stated reason
    When the owner hands back with a note
    Then a Needs you item shows the agent's reason marked untrusted
    And a worktree checkpoint is recorded
    And the agent receives only the owner's note and the checkpoint's address through the harness's input interface

  @OWN-21 @P1 @I6 @I10 @pending
  Scenario: Ready for review follows only an owner act and a later edit clears it
    Given an isolated Cairn home
    And a lane at head "h1"
    When the owner runs "cairn lane ready" and the agent then edits the worktree
    Then the lane showed Ready for review only after the act recorded with head "h1"
    And after the edit the lane returns to Running or Quiet until marked again

  @OWN-22 @P1 @I2 @I6 @I9 @pending
  Scenario: an open residual risk refuses widening acts unless the owner accepted it
    Given an isolated Cairn home
    And a session with no sandbox state record
    When the owner writes a widening act
    Then the session counts as unsandboxed and is still recorded
    And the act is refused unless a recorded acceptance names each open risk, including that the agent can forge it
    And an acceptance is shown on every surface beside each session relying on it
    And the acceptance is asked again when the set of open risks grows

  @OWN-23 @P1 @I2 @I6 @pending
  Scenario Outline: delegation beyond the session rests on a grant in force
    Given an isolated Cairn home
    And an agent of the principal in a running session
    And <grant>
    When the agent delegates a task to <target>
    Then the delegation is <outcome>

    Examples:
      | grant                                                | target                                    | outcome                                        |
      | no grant                                             | a subagent in its own session             | recorded under OWN-24, with no grant           |
      | no grant                                             | a new session in another worktree         | refused, audited and shown                     |
      | a grant naming that worktree, a budget and an expiry | a new session in that worktree            | started through the run component and recorded |
      | an expired grant                                     | an existing session of the same principal | refused, audited and shown                     |

  @OWN-24 @P1 @I2 @pending
  Scenario: a delegate inherits the delegating agent's ceiling and taint
    Given an isolated Cairn home
    And an agent whose session is recall-tainted, delegating under a grant whose rule ceiling is "ask first"
    When the delegate session starts
    Then the delegation is recorded on both sides with the delegating agent, the grant and the task's address
    And the delegate holds no rule level looser than the grant's or the delegating agent's
    And the delegate's session is recall-tainted
    And the task reached the delegate through the harness's input inside the fixed template, marked as written by the delegating agent
    And a further delegation beyond the grant's depth is refused

  @OWN-25 @P1 @I2 @I6 @pending
  Scenario: a delegate's result is pulled, never pushed
    Given an isolated Cairn home
    And a delegation outside the session that has returned a result
    When the delegating agent calls lane_result for that delegation
    Then the result arrives inside the untrusted envelope
    And no result text entered the delegating agent's context before that call
    And cancelling the grant stops every delegate it covers, as a cut act

  @OWN-26 @P2 @I2 @I8 @pending
  Scenario: another principal's agent takes work only under their acceptance grant
    Given an isolated Cairn home
    And a lane shared by principals "owner" and "co-author"
    And "co-author" has recorded an acceptance grant naming "owner", a target agent, a rule ceiling, a budget and an expiry
    When an agent of "owner" delegates a task to that target
    Then the task reaches the target in the fixed template, marked as from "owner"'s agent
    And the target keeps "co-author" as its one principal
    And the same delegation without the acceptance grant is refused and audited

  @OWN-27 @P1 @I2 @I10 @pending
  Scenario: only a person judges, and a verdict goes stale when what it judged changes
    Given an isolated Cairn home
    And a lane whose agent stated "C1 is done" and recorded a passing test run linked to C1
    When the owner records "met" on C1 and the agent then edits a file
    Then the verdict is recorded as the owner's act, bound to the intent version, the head and the evidence shown
    And Cairn pre-filled no verdict, and the agent's statement stays a claim
    And after the edit the verdict reads stale
    And the verdict does not approve the lane for landing

  @OWN-28 @P1 @I1 @I2 @pending
  Scenario: the owner course-corrects from the verdict
    Given an isolated Cairn home
    And a lane where the owner recorded "needs changes" on C2
    When the owner sends the correction "keep the header row in every file" and retries from an earlier checkpoint
    Then the correction reaches the agent through the harness's input in the fixed template naming the verdict, C2 and the results it concerns
    And the retry starts a new session in a new worktree at that checkpoint with the intent in force and the correction
    And the retry receives no content of the abandoned attempt except what it recalls
    And the abandoned attempt stays on record, shown beside the retry
    And C2 reads "unjudged" until the next verdict
