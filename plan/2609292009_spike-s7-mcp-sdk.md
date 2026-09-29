---
id: 2609292009
title: "Spike S7: the official MCP Go SDK or an in-house server"
status: "🔲"
summary: >-
  Evaluate the official MCP Go SDK for stdio, structured content and
  cancellation against a minimal in-house server.
model: sonnet
depends-on: []
---
# Spike S7: the official MCP Go SDK or an in-house server

## Goal

Choose how `cairn mcp` speaks MCP, weighing the SDK's fit against
ENG-18's dependency budget and SEC-01's import rules.

## Context

Informs §9.2 and RCL-01. The SDK must not pull `net/http` into the
shipped binary.

## Tasks

1. Build `search` as a stub tool with the SDK and with a minimal stdio
   server
2. Check structured content, cancellation, and the import closure of
   each
3. Record the decision and, if the SDK wins, its DEPENDENCIES.md row

## Acceptance Criteria

- [ ] A decision is recorded in
  [docs/srs/09-interfaces.md](../docs/srs/09-interfaces.md) or an ADR
- [ ] The import-closure test passes with the chosen approach
