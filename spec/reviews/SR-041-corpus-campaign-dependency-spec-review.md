---
id: SR-041
title: "Dependency review of the corpus campaign"
type: SpecReview
analysis: dependency
scope: "FR-008 through FR-010, NFR-004 and routed ecosystem work"
review_set: all
---

## Summary

**PASS after remediation.** Enablement precedes feature evidence, every family
has one owner, and conditional profile/native rows stay blocked until exact
accepted contracts and implementations exist. The dependency graph is acyclic.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-4101 | high | Draft corpus rows could be mistaken for authorization to implement pending W/M, past, or native-bridge features. Fixed: blocked cells have no verdict or denominator credit and require landed exact revisions plus M0/campaign acceptance. | MRS-002, FR-008-AC-4 |
| FND-4102 | medium | Corpus family ownership was implicit and could create competing canonical copies. Fixed with singular owner and consumer allocations plus exact revision/digest replay. | FR-009-AC-2 |
| FND-4103 | medium | The implementation order did not distinguish schema/census enablement from fixture, adapter, and measurement work. Fixed with a four-stage owner-routed topological order. | MRS-002 Admission and implementation gate |

## Classification

| Requirement | Class | Rationale |
|---|---|---|
| FR-008 | enablement | Defines the cell schema and census all fixture families consume. |
| FR-009 | enablement | Defines immutable family manifests, ownership, and lifecycle. |
| FR-010 | feature | Produces user/reviewer-visible mapping loss and observation dispositions. |
| NFR-004 | cross-cutting constraint | Constrains every population, retention, reproducibility, and claim surface. |

## Dependency graph and order

The explicit DAG is `M0 + accepted MRS-002 -> FR-008 -> FR-009 -> FR-010 ->
MP-002 collection`, with NFR-004 constraining FR-008 through FR-010. Existing
FR-001 through FR-003 precede current semantic cells; FR-004/FR-005/FR-007
precede FR-010. Accepted #37 precedes W/M admission, accepted #38 plus its
evaluator precedes past admission, and implemented #63/#64 precede native rows.
No reverse edge exists.
