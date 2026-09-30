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
  Scenario: import directions between internal packages are declared and enforced
    Given the declared import directions for the internal packages
    When a package imports a package its direction forbids
    Then the architecture check fails naming both packages
    And every internal package has exactly one declared responsibility

  @ENG-03 @P0 @pending
  Scenario: no mutable package state, injectable I/O and deadlines everywhere
    Given the source tree
    When the static analyzers run
    Then no package declares a mutable package-level variable
    And every exported blocking operation takes a context.Context
    And no projection package reads the wall clock or a random source

  @ENG-04 @P0 @pending
  Scenario: errors are wrapped, typed per counter, and panics stop at the entry points
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
    When Cairn writes the log record
    Then the record is JSON emitted through log/slog
    And the key appears only as a "[REDACTED:" marker

  @ENG-06 @P0 @pending
  Scenario: the crash harness kills Cairn mid-operation and every invariant holds
    Given an isolated Cairn home
    When the crash harness kills Cairn at randomized points during ingestion, purge, migration and rebuild
    Then after every kill "cairn verify" exits 0
    And a nightly run completes at least 10000 iterations

  @ENG-07 @P0 @pending
  Scenario Outline: native fuzzing covers every parser with a committed corpus
    Given the fuzz target for the <surface>
    Then its seed corpus is committed under testdata/fuzz
    And the nightly workflow fuzzes it

    Examples:
      | surface              |
      | transcript parser    |
      | hook input decoder   |
      | query compiler       |
      | envelope encoder     |
      | field sanitizer      |
      | redactor             |
      | configuration parser |

  @ENG-08 @P0 @pending
  Scenario Outline: property-based tests pin the record's core laws
    Given randomly generated records
    Then the property "<property>" holds for every one

    Examples:
      | property                                          |
      | seq is strictly increasing and gap-free           |
      | ingest twice equals ingest once                   |
      | rebuild reproduces projections exactly            |
      | envelope encoding round-trips any byte sequence   |
      | sanitized fields never contain restore delimiters |

  @ENG-09 @P0 @pending
  Scenario: the suite passes under the race detector and survives a concurrency soak
    Given an isolated Cairn home
    When 50 concurrent writers append 1000000 events
    Then no seq value is duplicated or skipped
    And "cairn verify" exits 0
    And the whole suite passes with -race

  @ENG-10 @P0 @pending
  Scenario: golden files pin the exact bytes of restore blocks and envelopes
    Given a fixture record
    When the restore block and a recall envelope are rendered
    Then both equal their committed golden files byte for byte

  @ENG-11 @P0 @pending
  Scenario: statement coverage meets the floor per package class
    Given a coverage profile of the whole suite
    Then every security-sensitive package is at least 90% covered
    And the module as a whole is at least 80% covered

  @ENG-12 @P0 @pending
  Scenario: the end-to-end suite runs with the network denied
    Given the end-to-end suite runs inside a network-denied sandbox
    When any code under test creates a socket
    Then the suite fails naming the caller

  @ENG-13 @P1 @pending
  Scenario: mutation testing scores the security-sensitive packages
    Given the security-sensitive packages
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
  Scenario: CI gates on the static analyzers and the custom checks
    Given the CI workflow
    Then it gates on go vet, staticcheck, gosec, errcheck and govulncheck
    And it gates on the TrustedText construction check
    And it gates on the forbidden-import check for net, net/http and os/exec
    And it gates on the wall-clock and randomness check for projection code

  @ENG-17 @P0 @pending
  Scenario: contract tests replay every supported Claude Code version
    Given recorded hook payloads and transcripts for each supported Claude Code version
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
    And CODEOWNERS names no owner on any other path
    And changes to security-sensitive packages need two approvals, one from the designated security reviewer

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

  @ENG-27 @P0 @pending
  Scenario: every check that keeps the records in step is proven by an injected drift
    Given the repository checkout
    When the drift cases are read
    Then every drift case's edit applies to the checkout
    And every non-pending scenario that inspects the repository checkout has a drift case guarding its id
    And the requirement-scenario gate and the Appendix B check each have a drift case
    And the CI workflow runs the drift suite with "go test -tags drift ./internal/drift"
