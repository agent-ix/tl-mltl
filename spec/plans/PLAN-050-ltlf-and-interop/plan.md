---
id: PLAN-050
title: LTLf evaluation and interchange plan
type: Plan
status: not_started
relationships:
  - target: ix://agent-ix/tl-mltl/FR-083
    type: references
  - target: ix://agent-ix/tl-mltl/FR-084
    type: references
  - target: ix://agent-ix/tl-mltl/FR-085
    type: references
  - target: ix://agent-ix/tl-mltl/FR-086
    type: references
  - target: ix://agent-ix/tl-mltl/FR-087
    type: references
  - target: ix://agent-ix/tl-mltl/FR-088
    type: references
  - target: ix://agent-ix/tl-mltl/FR-089
    type: references
  - target: ix://agent-ix/tl-mltl/FR-090
    type: references
  - target: ix://agent-ix/tl-mltl/FR-091
    type: references
  - target: ix://agent-ix/tl-mltl/FR-092
    type: references
  - target: ix://agent-ix/tl-mltl/FR-093
    type: references
  - target: ix://agent-ix/tl-mltl/FR-094
    type: references
  - target: ix://agent-ix/tl-mltl/NFR-047
    type: references
---

# PLAN-050: LTLf evaluation and interchange plan

## Objective

Land the tl-mltl side of TL-428 (LTLf evaluation and reference agreement), TL-431
(FRET MLTL and C2PO import, loss records, mission-bounding report), TL-432 (C2PO
import) and TL-433 (WEST import and export). No build task starts before the
ADRs 044 to 047 here and the tl-syntax and tl-parse ADRs are accepted by a human.

## Dependency DAG

Layer numbers match `ix-board project-chunk P-TL-151`: layer 0 TL-393 and TL-427;
layer 1 TL-394, TL-428, TL-431, TL-432, TL-433; layer 2 TL-395, TL-397, TL-429;
layer 3 TL-399, TL-434; layer 4 TL-404, TL-430; layer 5 TL-435. All tl-mltl tasks
sit in layer 1.

```text
layer 0  Task-000 (FR-093 gate: ADRs accepted, spec-review recorded)
layer 1  TL-428  Task-001 LTLf evaluator (FR-083, FR-085)
                   -> Task-002 reference corpus replay (FR-084)
         TL-432  Task-003 C2PO import (FR-086)
         TL-433  Task-004 WEST import and export (FR-092)
         TL-431  Task-005 loss records, FRET MLTL import, attribution (FR-091, FR-087, FR-089)
                   -> Task-006 FRET C2PO, variable map, header (FR-088)   needs Task-003
                   -> Task-007 mission report and closure refusal (FR-090, FR-094)   needs Task-001, Task-005
         all      Task-008 reader bounds and fuzz (NFR-047)   needs Task-003, Task-004, Task-005
```

Discrepancy for the planner: FR-088 (TL-431) reuses the C2PO reader of FR-086
(TL-432), but ix-board shows TL-431 and TL-432 in the same layer with no edge.
Either add TL-432 as a blocker of TL-431 or let Task-006 build the shared
reader first and TL-432 consume it; this plan assumes the former and flags it.

## Task File Mapping

| Task | Ticket | Scope | Exit evidence |
|---|---|---|---|
| Task-000 | TL-427 | FR-093 | TC-253 |
| Task-001 | TL-428 | FR-083, FR-085 | TC-227 to TC-230, TC-236 |
| Task-002 | TL-428 | FR-084 | TC-231 to TC-234 |
| Task-003 | TL-432 | FR-086 | TC-237 to TC-239 |
| Task-004 | TL-433 | FR-092 | TC-249 to TC-252 |
| Task-005 | TL-431 | FR-091, FR-087, FR-089 | TC-240, TC-241, TC-244, TC-247, TC-248 |
| Task-006 | TL-431 | FR-088 | TC-242, TC-243 |
| Task-007 | TL-431 | FR-090, FR-094 | TC-245, TC-246, TC-254 to TC-256 |
| Task-008 | TL-431/432/433 | NFR-047 | TC-257, TC-258 |

## Quality gates

- Gate A after Task-001: the closed-trace and prefix evaluators are untouched
  (existing corpora byte-identical) before any importer lands.
- Gate B before Task-007: the independent known-case enumerator exists and agrees
  with the LTLf evaluator on its own cases.

## Deferred verification

| Item | Why deferred | Needed to run |
|---|---|---|
| Fresh generation of the LTLf reference corpus (TC-235; blocks TC-231, TC-232) | no outside LTLf tool may be run during specification | install the ADR-044 reference at a recorded version, run the documented procedure, retain with provenance |
| Retrieval of published FRET examples (blocks TC-240, TC-242) | no FRET export was available offline; layouts ungrounded | fetch examples with licence and revision into the fixture manifest |
| Live R2U2/C2PO re-check of imported C2PO | no outside tool run | existing `make r2u2-nightly` pin |
| `make ci`, fuzz campaigns, mutation | no build capacity | repository gates |
| Human acceptance of ADR-044 to ADR-047 | only a human accepts | Peter |

## Ungrounded items

- FRET export file layouts and operator spellings (FR-087, FR-088).
- The C2PO unsupported-construct list against the R2U2 4.2 grammar (FR-086).
- Identity and versions of the LTLf reference tools (ADR-044).
- The literature's own named finite-versus-infinite counterexample (ADR-044 item 3).
