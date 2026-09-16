---
id: Task-008
title: "Implement QObs C00 consumer compatibility"
type: Task
status: done
track: A
priority: P0
relationships:
  - target: ix://agent-ix/tl-mltl/FR-019
    type: references
  - target: ix://agent-ix/tl-mltl/TC-085
    type: verifies
---

# Task-008: Implement QObs C00 consumer compatibility

## Scope

Advance the exact QObs dependency to C00, retain the real bounded temporal
consumer seam, and represent repair/query non-consumption without foreign
artifact input or semantic substitution.

## Subtasks

- [x] Pin Cargo resolution and public provenance to QObs `581d98f1`.
- [x] Adapt temporal fixtures to the exact typed FCD/QObs C00 constructor API.
- [x] Delegate temporal compatibility to `wire::request::derive`.
- [x] Expose typed supported and unsupported dispositions with exact contract
  and revision metadata.

## Deliverables

- `src/wire/observation.rs` and its public module export.
- Exact Cargo/lockfile/source/license policy updates.
- Exact admitted FCD static-bundle fixture with provenance.
- TC-085 and its FR-019 matrix trace.

## Notes

- Completed before this retrospective task file was authored; status reflects
  observed repository state, not a backdated workflow claim.
- QObs retains repair, query, authority-bundle, and observation semantics.
