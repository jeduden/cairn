---
id: 2609292006
title: "Spike S4: prove plugin distribution"
status: "🔲"
summary: >-
  Install a stub Cairn plugin on a workstation, a runner image and
  the Agent SDK, and check how runners seed ~/.claude.
model: sonnet
depends-on: []
---
# Spike S4: prove plugin distribution

## Goal

Show that Cairn can ship as a Claude Code plugin on every target ADR-06
names, so ADM-01 and NFR-12 rest on a working install.

## Context

Resolves ASM-07 and ASM-08, and informs OQ-06.

## Tasks

1. Package a plugin that registers a no-op hook and a stub MCP server
2. Install it on a workstation, a self-hosted runner image and an Agent
   SDK worker
3. Check how the runner seeds `~/.claude` and which user it runs as
4. Check whether interactive mode can be detected (OQ-06)

## Acceptance Criteria

- [ ] ASM-07 and ASM-08 marked verified or re-planned in the SRS
- [ ] Scenarios @ASM-07 and @ASM-08 pass against recorded install
  evidence
- [ ] OQ-06 has an answer or a stated follow-up
