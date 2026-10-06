---
title: "1. Introduction"
summary: >-
  Purpose and intent of Cairn, and where the rest of section 1 lives:
  the invariants, the scope and the non-goals each in their own file,
  and every term in the domain model.
---
# 1. Introduction

Section 1 spans four files: this one, [§1.3 Invariants](invariants.md),
[§1.4 Scope](01b-scope.md) and [§1.5 Non-goals for v1](01c-non-goals.md).
This specification defines no terms of its own: every term it uses is
defined in the [domain model](../domain-model.md).

## 1.1 Purpose

This document specifies Cairn, a context layer that lets Claude agents work over
unbounded histories — long research runs, long-running conversations, and
remote agents running for hours or days — without losing information to
compaction and without turning stored history into a prompt-injection channel.

## 1.2 Intent

Cairn keeps the **complete, tamper-evident record of what long-running Claude
agents did**, one room at a time, on their principals' own machines. It gives
the agents exact recall of it and the people working with them a live view of
it, without any new path from untrusted content to an agent. The record shows
tampering by anyone but a process of the same OS user until a head receipt
leaves the machine. Every component beyond the core is opt-in, bounded and
reviewed.

Existing tools treat memory as a convenience feature and add security
afterwards. Cairn treats the memory store as security-critical infrastructure
from the first line of code. Where usefulness and safety conflict, Cairn chooses
the design that keeps the agent safe and makes the convenience opt-in, never the
reverse.
