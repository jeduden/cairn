Feature: Engineering quality (ENG)

  Scenarios for SRS §10, one per requirement, tagged with its id and
  priority; the engineering tables carry no Traces column. A scenario
  still tagged @pending is declared but not yet written: its steps are
  bound when the plan that implements the requirement lands. The
  scenarios without @pending inspect the repository itself and run on
  every `go test ./...`.

  @ENG-01 @P0
  Scenario: the toolchain is pinned and release builds are static, trimmed and stamped
    Given the repository checkout
    Then "go.mod" pins the Go toolchain with a toolchain directive
    And the release workflow sets CGO_ENABLED to "0"
    And the release workflow builds with "-trimpath"
    And the release workflow builds with "-buildvcs=true"

  @ENG-02 @P0 @pending
  Scenario: dependency directions between the workspace crates are declared and enforced
    Given the declared dependency directions for the workspace crates
    When a crate depends on a crate its direction forbids
    Then the architecture check fails naming both crates
    And every library crate has exactly one declared responsibility
    And the browser room view client's TypeScript package imports nothing of the core but its generated types

  @ENG-03 @P0 @pending
  Scenario: no mutable global state, injectable I/O and deadlines everywhere
    Given the repository's code
    When the static analyzers run
    Then no crate declares mutable global state
    And every public blocking operation takes a deadline, directly or through a cancellation signal a deadline fires
    And no crate that computes derived artifacts reads the wall clock or randomness

  @ENG-04 @P0 @pending
  Scenario: errors keep their cause, are typed per counter, and panics stop at the entry points
    Given an isolated Cairn home
    And a hook handler that panics
    When the hook "SessionStart" runs with a valid input
    Then the command exits 0
    And the hook output is empty
    And an audit entry records "panic recovered"

  @ENG-05 @P0 @pending
  Scenario: structured logs pass through redaction
    Given an isolated Cairn home
    And a log field containing an AWS access key
    When Cairn writes the log line
    Then the log line is JSON emitted through tracing
    And the key appears only as a "[REDACTED:" marker

  @ENG-06 @P0 @pending
  Scenario: the crash-consistency test kills Cairn mid-operation and every invariant stays true
    Given an isolated Cairn home
    When the crash-consistency test kills Cairn at randomized points during ingestion, purge, migration and rebuild
    Then after every kill "cairn verify" exits 0
    And each nightly job completes at least 10000 iterations

  @ENG-07 @P0 @pending
  Scenario Outline: coverage-guided fuzzing covers every parser with a committed corpus
    Given the cargo-fuzz target for the <surface>
    Then its seed corpus is committed under fuzz/corpus
    And the nightly fuzz job fuzzes it

    Examples:
      | surface                   |
      | transcript parser         |
      | hook input decoder        |
      | event_search query parser |
      | envelope encoder          |
      | field sanitizer           |
      | redactor                  |
      | configuration parser      |

  @ENG-08 @P0 @pending
  Scenario Outline: property-based tests check the record's core laws
    Given randomly generated records
    Then the property "<property>" is true of every one

    Examples:
      | property                                                                                                           |
      | within each writer's log, seq is strictly increasing and gap-free, so no two events share an address (writer, seq) |
      | ingest twice equals ingest once                                                                                    |
      | rebuild reproduces derived artifacts exactly                                                                       |
      | envelope encoding round-trips any byte sequence                                                                    |
      | sanitized fields never contain restore delimiters                                                                  |

  @ENG-09 @P0 @pending
  Scenario: data races are excluded and the store survives a concurrency soak
    Given an isolated Cairn home
    When 50 concurrent writers append 1000000 events
    Then no two events share an address (writer, seq), and no seq is duplicated or skipped within any writer's log
    And "cairn verify" exits 0
    And every unsafe block that shares memory between threads passes under Miri or ThreadSanitizer

  @ENG-10 @P0 @pending
  Scenario: golden files fix the exact bytes of restore blocks and envelopes
    Given a fixture record
    When the restore block and an envelope are rendered
    Then both equal their committed golden files byte for byte

  @ENG-11 @P0 @pending
  Scenario: line coverage meets the floor per crate class
    Given a coverage profile of the whole suite
    Then every security-sensitive crate is at least 90% covered
    And the workspace as a whole is at least 80% covered

  @ENG-12 @P0 @pending
  Scenario Outline: the end-to-end suite runs each component confined to its boundary
    Given the end-to-end suite runs "<component>" confined to its boundary, which <confinement>
    When "<component>" opens a socket outside the boundary the register assigns it
    Then the suite fails naming the component and the caller

    Examples:
      | component           | confinement                   |
      | core                | denies all network            |
      | room-view component | denies all but loopback       |
      | launcher            | denies all but loopback       |
      | peer component      | allows only its register rows |
      | publish component   | allows only its register rows |
      | bridge component    | allows only its register rows |

  @ENG-13 @P1 @pending
  Scenario: mutation testing scores the security-sensitive crates
    Given the security-sensitive crates
    When mutation testing runs
    Then the mutation score is at least 70%

  @ENG-14 @P0 @pending
  Scenario: tests cannot touch the real home or Claude Code configuration
    Given a test that resolves a path under the real user's home
    When the suite runs
    Then the isolation guard aborts that test
    And no agent instruction file in the repository directs a destructive command at non-isolated state

  @ENG-15 @P0 @pending
  Scenario: the benchmark gate fails on a significant regression
    Given the benchmark suite covering NFR-01 to NFR-05
    When a change slows a benchmark by more than 10% with statistical significance
    Then the benchmark CI job fails naming the benchmark

  @ENG-16 @P0 @pending
  Scenario Outline: CI gates on the static checks and on build-time reach evidence per component
    Given the CI workflow
    When CI reads the build-time reach evidence for the "<component>", dependencies and start-up code included
    Then the workflow gates on cargo clippy, cargo fmt, cargo-deny, cargo-audit, the browser room view client's strict TypeScript compile and lint, and the custom checks
    And every crate that parses untrusted content or decides trust forbids unsafe code
    And a build-time check fails when code that computes derived artifacts reads a clock or randomness
    And a build-time check fails when restore content is built from anything but trusted text
    And CI fails when the evidence for the "<component>" is missing or shows that it <violation>

    Examples:
      | component           | violation                                                                                                      |
      | core                | opens any socket, or starts a program other than its own kernel worker                                         |
      | launcher            | makes an off-machine connection or listens beyond loopback or a local endpoint only the same OS user can reach |
      | room-view component | starts a program, makes an off-machine connection or listens beyond loopback                                   |
      | peer component      | starts a program or reaches beyond its register rows                                                           |
      | publish component   | starts a program or reaches beyond its register rows                                                           |
      | bridge component    | starts a program, listens, or reaches beyond its register rows                                                 |

  @ENG-17 @P0 @pending
  Scenario: contract tests replay every supported Claude Code version
    Given recorded hook inputs and transcripts for each supported Claude Code version
    When the contract suite replays them
    Then every one parses without an unparsed event
    And a nightly job drives real Claude Code through compaction, restore and recall

  @ENG-18 @P0
  Scenario: every direct dependency is justified by an accepted ADR with an allow-listed license
    Given the repository checkout
    When the direct dependencies are read from "go.mod"
    And the ADRs are read from "docs/adr"
    Then each direct dependency is named by exactly one accepted ADR
    And every module an accepted ADR names is a direct dependency
    And each named module has a purpose, a license and a maintenance status, and its ADR weighs alternatives
    And each license is on the allow-list the ENG-18 requirement states
    And the direct dependencies stay within the target the ENG-18 requirement states
    And "DEPENDENCIES.md" lists every ADR that names a module

  @ENG-19 @P0 @pending
  Scenario: two independent builders produce bit-identical release artifacts
    Given a release build of one commit on two independent builders
    When their artifacts are compared
    Then every artifact has the same SHA-256 on both builders

  @ENG-20 @P0 @pending
  Scenario: release artifacts are signed and carry provenance and an SBOM
    Given a published release
    Then the checksum file verifies with its keyless Sigstore bundle
    And every binary carries SLSA Build Level 3 provenance
    And an SPDX SBOM is attached and attested

  @ENG-21 @P0 @pending
  Scenario: main only moves through reviewed, signed pull requests
    Given the repository's branch protection for main
    Then direct pushes are rejected
    And commits must be signed
    And every pull request needs an approval from a reviewer other than its author
    And CODEOWNERS names the stakeholder on the requirement text and on every path that enforces it
    And CODEOWNERS names no code owner on any other path
    And changes to security-sensitive crates need two approvals, one from the designated security reviewer

  @ENG-22 @P0 @pending
  Scenario: every privacy or data-flow statement cites its proving test
    Given the documentation
    When every privacy or data-flow statement is listed
    Then each one references a test that exists and passes

  @ENG-23 @P0 @pending
  Scenario: releases are semver with a changelog, upgrade notes and migration tests
    Given a release candidate version
    Then its version is valid semantic versioning
    And the changelog and upgrade notes have a section for it
    And migrations from every previous minor version are tested

  @ENG-24 @P0
  Scenario: SECURITY.md names a private channel and a 90-day disclosure policy
    Given the repository checkout
    Then "SECURITY.md" links the private vulnerability reporting channel
    And "SECURITY.md" states a 90-day coordinated disclosure policy
    And "SECURITY.md" states that security fixes are backported to the latest minor release

  @ENG-25 @P1 @pending
  Scenario: two maintainers and an incident runbook before general availability
    Given the v1.0 release checklist
    Then at least two active maintainers are listed
    And an incident-response runbook exists in the repository

  @ENG-26 @P0
  Scenario: every design decision lives in one ADR file and a changed decision supersedes it
    Given the repository checkout
    When the ADRs are read from "docs/adr"
    Then every ADR has an id, a title, a status and a summary
    And every ADR's file is named for its id
    And every ADR's status is proposed, accepted or superseded
    And no two ADRs share an id
    And every superseded ADR names an ADR that exists as its successor

  @ENG-27 @P0
  Scenario: every check that keeps the records in step is proven by an injected drift
    Given the repository checkout
    When the drift cases are read
    Then the CI workflow runs the drift suite with "go test -tags drift ./internal/drift"
    And every drift case's edit applies to the checkout
    And every non-pending scenario that inspects the repository checkout has a drift case guarding its id
    And the requirement-scenario gate, the Appendix B check and the persona gates each have a drift case

  @ENG-28 @P0
  Scenario: an agent's approval passes through a gate the agent cannot reach
    Given the repository checkout
    When the review workflow is read from ".github/workflows/review.yml"
    Then it runs only when the "CI" workflow completes, as the default branch defines it
    And the job that runs the reviewing agent holds no write permission and not the "JEDUDEN_REVIEW_AGENT_KEY" key
    And only one job holds the "JEDUDEN_REVIEW_AGENT_KEY" key, and it runs no agent
    And that job runs only after the agent's job succeeded
    And that job decides the review with "go run ./cmd/review-gate"
    And the review gate decides:
      | verdict         | finding  | head    | CI     | other check | review          |
      | approve         | none     | current | passed | passed      | APPROVE         |
      | approve         | nit      | current | passed | passed      | APPROVE         |
      | approve         | blocking | current | passed | passed      | REQUEST_CHANGES |
      | approve         | none     | current | passed | failed      | REQUEST_CHANGES |
      | request_changes | none     | current | passed | passed      | REQUEST_CHANGES |
      | approve         | none     | moved   | passed | passed      | nothing         |
      | approve         | none     | current | failed | passed      | an error        |
      | malformed       | none     | current | passed | passed      | an error        |

  @ENG-29 @P0 @pending
  Scenario: an invariant or I2-review change lands only with an ADR recording a named security reviewer's approval
    Given the repository checkout
    When a change to an invariant or to a requirement marked for I2 review is proposed
    Then it lands only with an accepted ADR that records a named human security reviewer's approval
    And until that ADR is accepted, every component that crosses B0 is built only as a prototype
    And evidence from the release build proves no release artifact or tagged release contains prototype code
