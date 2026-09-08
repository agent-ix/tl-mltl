---
id: SR-018
title: "Dependency retrospective review of the tl-mltl specification corpus"
type: SpecReview
analysis: dependency
scope: "spec/requirements, spec/plans, spec/assurance, and cross-repository relationships at origin/main 5c4ce2a"
review_set: all
---

## Summary

The logical dependency graph is acyclic: StR-001 and StR-002 lead to FR-001 through
FR-006; StR-003 leads to FR-007; NFR-001 through NFR-003 constrain those outcomes.
FR-007 correctly consumes the shared tl-syntax contract and names its tl-rewrite
dependency without adding a runtime cycle. The remaining dependency risk is release
coordination rather than a missing prerequisite edge.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No dependency cycle or unowned prerequisite was found. Preserve the existing separation between shared contract dependencies and local runtime dependencies when pins advance; a direct tl-rewrite runtime dependency would recreate the cycle the contextual design explicitly excludes. | FR-007 Dependencies; FR-007-AC-7; StR-003; SR-012 FND-1203 |
