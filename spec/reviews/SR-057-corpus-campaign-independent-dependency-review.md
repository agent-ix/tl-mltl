---
id: SR-057
title: "Independent dependency review of the current corpus campaign"
type: SpecReview
analysis: dependency
scope: "FR-008 through FR-010, NFR-004, PLAN-006, linked implementation tickets"
review_set: all
---

## Summary

**PASS after remediation.** The current future/W/M critical path is acyclic and
separates shared enablement from owner-family and reporting work. Native bridge
consumption remains a conditional successor and cannot block or contaminate the
current baseline.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-5701 | high | W/M prerequisites were recorded as unresolved even though all five repository lanes had landed. Fixed by pinning the exact merges and removing W/M from the blocked family. | MRS-002, FR-008 Dependencies |
| FND-5702 | high | Prose implementation stages did not supply task-level predecessor edges or an inspectable TC-048 input. Fixed by the PLAN-006 DAG and GitHub issues #51–#55/#44/#33/#37. | PLAN-006, TC-048 |
| FND-5703 | medium | Native correspondence could become an accidental critical-path dependency for the current corpus. Fixed as Task-008, a blocked successor lane depending on #63/#64 and Task-001 but not gating Task-007. | Task-008, MRS-002 |
| FND-5704 | low | No dependency cycle remains: schema precedes owner families; the syntax owner family precedes consumers; current families and dispositions precede the integrated report. | PLAN-006 |

## Classification

| Requirement | Class | Rationale |
|---|---|---|
| FR-008 | enablement | Shared wire, identity, registry, and census contract. |
| FR-009 | enablement | Owner-family manifests and lifecycle consumed by replay/reporting. |
| FR-010 | feature | Reviewer-visible loss, availability, and comparison output. |
| NFR-004 | cross-cutting constraint | Reproducibility and claim integrity across every task. |

## Topological order

`#38 acceptance -> Task-001 -> Task-002 -> (Task-003 || Task-004 || Task-005 || Task-006) -> Task-007`.
Task-008 proceeds separately after `Task-001 + quire-contract-ir#63 + #64`.
