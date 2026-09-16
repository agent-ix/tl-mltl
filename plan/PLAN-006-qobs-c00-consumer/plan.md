---
id: PLAN-006
title: "QObs C00 consumer compatibility"
type: Plan
status: complete
relationships:
  - target: ix://agent-ix/tl-mltl/FR-019
    type: references
---

# Implementation Plan: QObs C00 consumer compatibility

## Retrospective ordering

This narrow bundle was authored after the FR-019 implementation because the
formal post-implementation gap-analysis workflow requires a typed plan target.
It records the actual order: the requirement and subset spec review preceded
code; risk/evidence readiness and this task decomposition were completed
retrospectively before the formal gap gate. It does not claim test-first task
artifacts that did not exist.

## Requirements Summary

### Functional Requirements

- [x] **FR-019**: Bind the exact QObs C00 revision, delegate the existing bounded
  temporal consumer path, and return typed unsupported dispositions for QObs
  repair and query contracts TL does not consume.

## Dependency Graph

- `FR-018 -> FR-019`
  Reason: the C00 successor reuses the accepted constructor-private temporal
  views and bounded `wire::request::derive` adapter; it adds no evaluator.
- `QObs FR-009 -> FR-019 temporal compatibility`
  Reason: the exact C00 authority types and revision provenance are upstream
  owner facts.
- `QObs FR-010, FR-011 -> FR-019 unsupported dispositions`
  Reason: TL must name those owner contracts without accepting their artifacts
  or recreating their semantics.

### Cross-cutting constraints

- NFR-001's deterministic/resource boundary remains owned by FR-018 and is
  observed through exact byte/usage and one-over comparisons.
- NFR-002's provenance boundary applies to Cargo resolution, the exported QObs
  revision, typed fixture construction, and dependency-source policy.

### The seams

`src/wire/observation.rs` dispatches the compatibility selector and delegates
temporal input to `wire::request::derive`. `tests/tc_084_temporal_owner_wire.rs`
exercises that public seam with constructor-private QObs views.

## Test Plan

### Integration Tests

- [x] **TC-085** (FR-019-AC-1 through FR-019-AC-3): compare direct and
  compatibility-dispatched temporal bytes/usage, exercise exact and one-over
  limits, inspect supported/unsupported metadata, and bind the exact revision
  across Cargo/public provenance.

## Remaining Work

### Track A: Critical Path (serial)

- **A1 = Task-008** Compatibility implementation — complete; exit: exact C00
  pin, admitted typed fixture, real bounded temporal delegation, and closed
  repair/query refusals are present.
- **A2 = Task-009** Pre-gap verification — complete; exit: Rust review has
  no unresolved finding and repository gates pass for the candidate. **Gate:** exact focused,
  all-target, dependency, documentation, spec, and assurance checks pass.

## Parallel Execution Summary

```text
Task-008 specification + compatibility implementation
  -> Task-009 Rust review + candidate gates
    -> formal gap analysis -> commit/push/PR
```

No independent track is claimed: both tasks touch the same dependency,
provenance, fixture, matrix, and owner-boundary files.

## Task File Mapping

| Task | Track | Owns (references) | Verified by (verifies) | Status |
|---|---|---|---|---|
| Task-008 | A | FR-019 | TC-085 | done |
| Task-009 | A | FR-019 | TC-085 | done |

## Coordination Rules

- Keep the QObs revision one exact 40-hex commit across Cargo, lockfile, public
  provenance, compatibility results, documentation, and tests.
- Do not add repair/query artifact parameters or reproduce QObs-owned semantics.
- Use native Quoin for normal work; the npm fallback is confined to the affected
  assurance gate documented in agent-ix/quoin#543.
- Keep TL PR #70 open and unmerged until draft QObs PR #27 lands at the exact
  pinned commit; a different upstream landing requires repinning and renewed
  compatibility review.
- Run formal gap analysis only after both plan tasks are done, then publish but
  do not merge the final PR.
