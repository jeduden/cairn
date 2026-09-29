---
id: 2609292011
title: "The remaining engineering gates in CI"
status: "🔲"
summary: >-
  Custom analyzers, import-direction checks, the network-deny
  sandbox, the benchmark gate, branch protection and SLSA level 3 —
  the ENG P0 gates M1 exits on.
model: sonnet
depends-on: []
---
# The remaining engineering gates in CI

## Goal

Turn the engineering requirements that are CI gates rather than product
behavior into running checks, so M1 can exit on "CI gates green".

## Context

Bootstrap already runs vet, staticcheck, gosec, errcheck, govulncheck,
depguard, race tests, fuzz discovery and a reproducible signed release.
This plan covers the rest.

## Tasks

1. Declare the allowed import directions between internal packages and
   enforce them (ENG-02)
2. Add analyzers for TrustedText construction and for clock or
   randomness use in projection code (ENG-03, ENG-16, SEC-07)
3. Run the end-to-end suite in a network-denied sandbox that fails on
   any socket (ENG-12, SEC-01)
4. Add the isolation guard that aborts a test touching the real home
   (ENG-14)
5. Add the benchmark job with a significance-tested 10% regression gate
   (ENG-15)
6. Move the release build into a reusable workflow for SLSA Build Level
   3 (ENG-20)
7. Protect main: reviewed, signed pull requests only; agents approve
   code, and CODEOWNERS puts the stakeholder on the requirement text
   and the gates that enforce it (ENG-21)

## Acceptance Criteria

- [ ] Scenarios @ENG-02, @ENG-03, @ENG-12, @ENG-14, @ENG-15, @ENG-16,
  @ENG-19, @ENG-20 and @ENG-21 pass
- [ ] All tests pass: `go test -race ./...`
- [ ] `go tool -modfile=tools/go.mod golangci-lint run` is clean
