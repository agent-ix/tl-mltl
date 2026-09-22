---
id: Task-017
title: "Establish the M5 admission snapshot"
type: Task
status: blocked
track: Gate
priority: P0
owner_repository: agent-ix/tl-mltl
consumer_repositories: [agent-ix/tl-mltl]
evidence_method: integration-and-inspection-test
github_issue: ix://agent-ix/tl-mltl/issues/58
resume_conditions: [ix://agent-ix/tl-mltl/issues/38, ix://agent-ix/tl-mltl/issues/39]
relationships:
  - target: ix://agent-ix/tl-mltl/issues/38
    type: depends_on
  - target: ix://agent-ix/tl-mltl/issues/39
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-020
    type: references
  - target: ix://agent-ix/tl-mltl/FR-024
    type: references
  - target: ix://agent-ix/tl-mltl/NFR-005
    type: references
  - target: ix://agent-ix/tl-mltl/TC-104
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-129
    type: verifies
---

# Task-017: Establish the M5 admission snapshot

## Scope

Bind the campaign to accepted M0/M4/MRS-003 revisions and classify every
semantic and shared-capability prerequisite as admitted or blocked.

## Subtasks

- [ ] Record exact landed M0 and MRS-002/PLAN-007 identities plus the accepted
  MRS-003 identity.
- [ ] Classify W/M and past/history rows against their named gates.
- [ ] Demonstrate complete exact Quire `implements` bindings plus attachment,
  build-profile, and local active-plan capabilities without a local substitute.

## Deliverables

- Reproducible admission manifest and refusal fixtures.

## Notes

- Resume only after the M4 corpus specification is accepted and landed and #39
  records human acceptance of the exact reviewed MRS-003 revision.
