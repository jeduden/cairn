---
title: "Appendix B — Invariant coverage"
summary: >-
  Which requirements serve each invariant, generated from the Traces
  column of §5–§6, plus the requirement count by priority. A Go test
  keeps both in step with the tables.
---
# Appendix B — Invariant coverage

Generated from the Traces column of §5–§6. Every invariant is served by at least
one verifiable requirement; NFR and ENG requirements provide additional coverage
not listed here.

| Invariant                                                                     | Requirements                                                                                                                                                                                                                                           |
| ----------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **I1** — Nothing is lost                                                      | REC-01, REC-02, REC-03, REC-04, REC-05, REC-06, REC-07, REC-08, REC-09, REC-14, REC-15, REC-16, RCL-01, RCL-03, ADM-05, ADM-06, ADM-07, SEC-08                                                                                                         |
| **I2** — No automatic path from untrusted content to the model                | PRV-01, PRV-02, PRV-03, PRV-04, PRV-05, PRV-06, PIN-01, PIN-02, PIN-05, PIN-07, RCL-04, LMK-02, LMK-03, LMK-04, LMK-06, INJ-03, INJ-04, INJ-08, INJ-09, CMP-06, ADM-12, MEM-01, MEM-02, CUE-01, CUE-07, CUE-10, CUE-12, SEC-06, SEC-07, SEC-11, SEC-13 |
| **I3** — Constraints are never summarized                                     | PIN-01, PIN-03, PIN-05, PIN-06, PIN-07, PIN-08, INJ-01, INJ-02                                                                                                                                                                                         |
| **I4** — Cairn never talks to the network                                     | CMP-04, CMP-08, CMP-09, OPS-05, SEC-01, SEC-10, SEC-15                                                                                                                                                                                                 |
| **I5** — Bad data can be removed from circulation without destroying evidence | REC-15, PRV-07, RCL-06, LMK-04, CMP-07, ADM-07, CUE-07, SEC-12, SEC-14                                                                                                                                                                                 |
| **I6** — No silent failures                                                   | REC-05, REC-07, REC-08, PRV-07, PIN-08, RCL-07, ADM-04, ADM-09, ADM-11, OPS-01, OPS-02, OPS-03, OPS-04, CUE-09, CUE-11, SEC-16                                                                                                                         |
| **I7** — Configuration changes only on explicit instruction                   | ADM-01, ADM-02, ADM-03, ADM-10, CUE-12, SEC-11                                                                                                                                                                                                         |
| **I8** — Isolation follows the tenant                                         | RCL-05, CMP-07, SEC-02, SEC-03, SEC-18                                                                                                                                                                                                                 |
| **I9** — Cairn never degrades the agent                                       | REC-13, INJ-05, INJ-07, CMP-05, CUE-08, CUE-11, SEC-04, SEC-05, SEC-16                                                                                                                                                                                 |
| **I10** — Everything derived is rebuildable                                   | REC-03, REC-06, REC-10, REC-12, PIN-04, LMK-01, LMK-05, INJ-06, ADM-08, ADM-09, CUE-02, CUE-09                                                                                                                                                         |

## Requirement count

| Priority | Functional and security (§5–§6) | Engineering (§10) |
| -------- | ------------------------------- | ----------------- |
| P0       | 85                              | 26                |
| P1       | 23                              | 2                 |
| P2       | 4                               | 0                 |
| —        | Non-functional (§7): 14         |                   |
