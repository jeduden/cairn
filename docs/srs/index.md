---
summary: >-
  The Cairn Software Requirements Specification — the normative
  source for every requirement id a feature scenario is tagged with.
---
# Cairn — Software Requirements Specification

|                             |                                                                                                                   |
| --------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| **Product**                 | Cairn — lossless, security-first context layer for long-running Claude agents                                     |
| **Document**                | Software Requirements Specification (SRS)                                                                         |
| **Version**                 | 2.1-draft                                                                                                         |
| **Status**                  | Draft; the invariant changes are approved in [ADR-2610032155](../adr/ADR-2610032155-srs-2-invariants.md) (ENG-29) |
| **Date**                    | 2026-10-05                                                                                                        |
| **Implementation language** | Rust; TypeScript for the UI page ([ADR-2610050528](../adr/ADR-2610050528-language-rust-typescript.md))            |
| **Owners**                  | Product owner and security reviewer: @jeduden                                                                     |

## Change log

| Version   | Date       | Summary                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| --------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 0.1       | 2026-09-29 | Initial requirements                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| 0.2       | 2026-09-29 | Intent, invariants, concern traceability                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| 1.0-draft | 2026-09-29 | Full rewrite: verifiable requirements, trust model, event-sourced design, assumptions register, engineering and verification plan                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| 1.1-draft | 2026-09-29 | ENG-21: agents review each other's changes; the stakeholder approves requirement and gate changes                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| 1.2-draft | 2026-09-29 | ENG-26: design decisions as ADR files; ENG-18 justifies dependencies through them                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| 1.3-draft | 2026-09-30 | ENG-27: every record-keeping check is proven by an injected drift in CI                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| 1.4-draft | 2026-09-30 | ENG-28: agent approvals are posted by a reviewer app, through a gate the reviewing agent cannot reach                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| 1.5-draft | 2026-10-01 | ASM-11..16 and PRV-08 from a review of sottochat's transcript parser: harness context in attachments, structure-only provenance, missing timestamps, a closed kernel allow-list (CMP-03), OQ-12                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| 2.0-draft | 2026-10-03 | The lane: one signed log per writer, network boundaries B0–B3 (I1, I2, I4, I5, I8 and I10 reworded), the LANE, VIEW, OWN and PEER families, personas U1–U9 with Appendix C, the boundary register and the lane vocabulary, from plan 2610012322's proposal; blind peers another entity can host (PEER-12); agent-to-agent delegation (OWN-23 to OWN-26, VIEW-20); lane visibility and invite links (LANE-17, LANE-18); subagents sharing a worktree (LANE-19)                                                                                                                                                                                                                              |
| 2.1-draft | 2026-10-05 | OQ-32 answered for the language and the shell: Rust for every process around one core library, TypeScript for the one UI page, Tauri 2 as the app shell ([ADR-2610050528](../adr/ADR-2610050528-language-rust-typescript.md), [ADR-2610042341](../adr/ADR-2610042341-app-shell-tauri.md), both proposed); CON-01, CON-02, CON-03, ENG-01 to ENG-05, ENG-07, ENG-09, ENG-11, ENG-13, ENG-16, ENG-21, INJ-03, SEC-07, OPS-05, ADM-01, NFR-10, NFR-13, ADR-04, ADR-06, ADR-07, S2, S7, OQ-03, OQ-04 and OQ-32 restated for Rust and TypeScript; INJ-03 tightened (private fields, no public way to build or change a `TrustedText`) and OQ-04 reopened (the encrypting VFS left with ncruces) |

## How to read this document

- §1–§3 explain **why** Cairn exists and **what** it promises. Read these first.
- §4 is a **non-normative** reference architecture with key design decisions.
- §5–§7 are the **normative** requirements, and §8–§10 define normative
  **data**, **interfaces**, and **engineering quality** requirements.
- §11–§13 cover **verification**, **delivery**, and **open issues**.
- Appendix A traces every concern raised during research to the requirements
  that address it; Appendix B lists the requirements serving each invariant;
  Appendix C lists the personas each requirement serves.

## Conventions

- **MUST / MUST NOT / SHOULD / SHOULD NOT / MAY** are used as defined in RFC
  2119 and RFC 8174, and only when written in capitals.
- **Priority:** **P0** = required for v1.0; **P1** = expected for v1.0, may slip
  to v1.1 with explicit sign-off; **P2** = later release, specified now so v1
  does not preclude it.
- **Verification method:** **T** = automated test; **I** = inspection or review;
  **A** = analysis or measurement; **D** = demonstration.
- **Traces** link each requirement to the invariants (I1–I10) it serves.
- Every numeric target marked † is an **initial target**, to be calibrated with
  measurements during milestone M0 and then frozen.

## Sections

<?catalog
glob:
  - "*.md"
  - "!index.md"
sort: path
header: ""
row: "- [{title}]({filename}) — {summary}"
?>
- [1. Introduction](01-introduction.md) — Purpose, intent, the ten invariants (I1–I10) every requirement serves, scope by context layer, non-goals, and the glossary, including the lane, writer and owner-act terms of 2.0.
- [2. Context](02-context.md) — Stakeholders, deployment context, the assumptions register (ASM-01..20) about Claude Code behaviour that M0 must verify, the hard constraints (CON-01..06), and the personas U1–U9.
- [3. Rationale](03-rationale.md) — The research findings behind the design — lossless record, verbatim pins, pull-only recall, code access, cache stability, and lessons from lcm.
- [4. Reference architecture (non-normative)](04-reference-architecture.md) — Non-normative reference architecture: components, event-sourced state, the compaction and aggregation scenarios, and design decisions ADR-01..10.
- [5. Functional requirements](05-functional-requirements.md) — Normative functional requirements with priority, verification method and invariant traces: record (REC), provenance (PRV), pins (PIN), recall (RCL), landmarks (LMK), restore (INJ), kernel (CMP), administration (ADM), memory boundary (MEM), observability (OPS). The lane families continue in 05b and 05c.
- [5. Functional requirements: lane and lane view](05b-lane-requirements.md) — Normative functional requirements for the lane (LANE): identity, actors, results, evidence, landing and membership; and for the lane view (VIEW): what a person reads on every surface. Part of §5.
- [5. Functional requirements: owner acts and peers](05c-owner-and-peer-requirements.md) — Normative functional requirements for owner acts (OWN): answers, steering, endorsement, the owner-act classes and the sandbox state they rest on; and for the peer network (PEER). Part of §5.
- [6. Security](06-security.md) — Threat model (assets, actors, threats T1–T12 and T14–T25 with their controls, and the residual risks R1–R7 only a sandbox removes), the normative security requirements SEC-01..31, and the boundary register.
- [7. Non-functional requirements](07-non-functional-requirements.md) — Normative non-functional requirements NFR-01..15: latency, throughput, scale, availability, durability, concurrency, footprint, portability, compatibility, usability, maintainability, documentation.
- [8. Data and storage](08-data-and-storage.md) — Normative home layout, logical schema, canonical encoding (RFC 8785 + SHA-256) and the conservative token estimator.
- [9. Interfaces](09-interfaces.md) — Normative interfaces: the hook contract, the MCP tools, the recall envelope, the restore block, the CLI with its exit codes, and the configuration keys a project may only tighten.
- [9.7 Lane vocabulary](09b-lane-vocabulary.md) — The one vocabulary every surface uses (VIEW-14): harness status and freshness, lane status, evidence and proof classes, the Needs you order, integrity seals, trust marks and the keymap. Part of §9.
- [10. Engineering quality (ENG)](10-engineering-quality.md) — Engineering quality requirements ENG-01..29: code organisation, testing, static analysis, supply chain, and process.
- [11. Verification and acceptance](11-verification-and-acceptance.md) — The evaluation plan with baselines and acceptance targets, and the definition of done for v1.0.
- [12. Delivery plan](12-delivery-plan.md) — Milestone M0 spikes S1–S12 and milestones M1–M9 with their scope and exit criteria.
- [13. Open questions and risks](13-open-questions-and-risks.md) — Open questions OQ-01..32 with their resolution path, and the risk register with mitigations.
- [14. References](14-references.md) — The papers, issues, and standards the specification cites.
- [Appendix A — Concern traceability](appendix-a-concern-traceability.md) — Every concern raised during research and review, mapped to the requirements that answer it.
- [Appendix B — Invariant coverage](appendix-b-invariant-coverage.md) — Which requirements serve each invariant, generated from the Traces column of §5–§6, plus the requirement count by priority. A Go test keeps both in step with the tables.
- [Appendix C — Persona coverage](appendix-c-persona-coverage.md) — Which personas (§2.5) each requirement of §5–§7 and §10 serves. A gate test keeps every requirement covered and every persona served, and another keeps §2.5 in step with the persona agents.
- [1.3 Invariants](invariants.md) — The single source of Cairn's ten invariants, I1 to I10, which every requirement serves; other files include it through mdsmith.
<?/catalog?>
