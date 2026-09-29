---
summary: >-
  The Cairn Software Requirements Specification — the normative
  source for every requirement id a feature scenario is tagged with.
---
# Cairn — Software Requirements Specification

|                             |                                                                               |
| --------------------------- | ----------------------------------------------------------------------------- |
| **Product**                 | Cairn — lossless, security-first context layer for long-running Claude agents |
| **Document**                | Software Requirements Specification (SRS)                                     |
| **Version**                 | 1.2-draft                                                                     |
| **Status**                  | Draft for kickoff review                                                      |
| **Date**                    | 2026-09-29                                                                    |
| **Implementation language** | Go                                                                            |
| **Owners**                  | TBD (product owner, tech lead, security reviewer)                             |

## Change log

| Version   | Date       | Summary                                                                                                                           |
| --------- | ---------- | --------------------------------------------------------------------------------------------------------------------------------- |
| 0.1       | 2026-09-29 | Initial requirements                                                                                                              |
| 0.2       | 2026-09-29 | Intent, invariants, concern traceability                                                                                          |
| 1.0-draft | 2026-09-29 | Full rewrite: verifiable requirements, trust model, event-sourced design, assumptions register, engineering and verification plan |
| 1.1-draft | 2026-09-29 | ENG-21: agents review each other's changes; the stakeholder approves requirement and gate changes                                 |
| 1.2-draft | 2026-09-29 | ENG-26: design decisions as ADR files; ENG-18 justifies dependencies through them                                                 |

## How to read this document

- §1–§3 explain **why** Cairn exists and **what** it promises. Read these first.
- §4 is a **non-normative** reference architecture with key design decisions.
- §5–§7 are the **normative** requirements, and §8–§10 define normative
  **data**, **interfaces**, and **engineering quality** requirements.
- §11–§13 cover **verification**, **delivery**, and **open issues**.
- Appendix A traces every concern raised during research to the requirements
  that address it.

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
- [1. Introduction](01-introduction.md) — Purpose, intent, the ten invariants (I1–I10) every requirement serves, scope by context layer, non-goals, and the glossary.
- [2. Context](02-context.md) — Stakeholders, deployment context, the assumptions register (ASM-01..10) about Claude Code behaviour that M0 must verify, and the hard constraints (CON-01..05).
- [3. Rationale](03-rationale.md) — The research findings behind the design — lossless record, verbatim pins, pull-only recall, code access, cache stability, and lessons from lcm.
- [4. Reference architecture (non-normative)](04-reference-architecture.md) — Non-normative reference architecture: components, event-sourced state, the compaction and aggregation scenarios, and design decisions ADR-01..10.
- [5. Functional requirements](05-functional-requirements.md) — Normative functional requirements with priority, verification method and invariant traces: record (REC), provenance (PRV), pins (PIN), recall (RCL), landmarks (LMK), restore (INJ), kernel (CMP), administration (ADM), memory boundary (MEM), observability (OPS).
- [6. Security](06-security.md) — Threat model (assets, actors, threats T1–T12 and their controls) and the normative security requirements SEC-01..18.
- [7. Non-functional requirements](07-non-functional-requirements.md) — Normative non-functional requirements NFR-01..14: latency, throughput, scale, availability, durability, concurrency, footprint, portability, compatibility, usability, maintainability, documentation.
- [8. Data and storage](08-data-and-storage.md) — Normative home layout, logical schema, canonical encoding (RFC 8785 + SHA-256) and the conservative token estimator.
- [9. Interfaces](09-interfaces.md) — Normative interfaces: the hook contract, the MCP tools, the recall envelope, the restore block, the CLI with its exit codes, and the configuration keys a project may only tighten.
- [10. Engineering quality (ENG)](10-engineering-quality.md) — Engineering quality requirements ENG-01..25: code organisation, testing, static analysis, supply chain, and process.
- [11. Verification and acceptance](11-verification-and-acceptance.md) — The evaluation plan with baselines and acceptance targets, and the definition of done for v1.0.
- [12. Delivery plan](12-delivery-plan.md) — Milestone M0 spikes S1–S8 and milestones M1–M6 with their scope and exit criteria.
- [13. Open questions and risks](13-open-questions-and-risks.md) — Open questions OQ-01..11 with their resolution path, and the risk register with mitigations.
- [14. References](14-references.md) — The papers, issues, and standards the specification cites.
- [Appendix A — Concern traceability](appendix-a-concern-traceability.md) — Every concern raised during research and review, mapped to the requirements that answer it.
- [Appendix B — Invariant coverage](appendix-b-invariant-coverage.md) — Which requirements serve each invariant, generated from the Traces column of §5–§6, plus the requirement count by priority. A Go test keeps both in step with the tables.
<?/catalog?>
