---
id: Task-001
title: "Establish the M5 admission snapshot"
type: Task
status: blocked
track: Gate
priority: P0
relationships:
  - target: ix://agent-ix/tl-mltl/FR-011
    type: references
  - target: ix://agent-ix/tl-mltl/FR-015
    type: references
  - target: ix://agent-ix/tl-mltl/NFR-005
    type: references
  - target: ix://agent-ix/tl-mltl/TC-050
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-075
    type: verifies
---

# Task-001: Establish the M5 admission snapshot

## Scope

Bind the campaign to accepted M0/M4/MRS-003 revisions and classify every
semantic and shared-capability prerequisite as admitted or blocked.

## Subtasks

- [ ] Record exact landed #43, #44/MRS-002, and accepted MRS-003 identities.
- [ ] Classify W/M, past/history, and native rows against their named gates.
- [ ] Demonstrate complete exact Quire `implements` bindings plus attachment,
  build-profile, and local active-plan capabilities without a local substitute.

## Deliverables

- Reproducible admission manifest and refusal fixtures.

## Notes

- Resume only after each mandatory predecessor is accepted and landed.
