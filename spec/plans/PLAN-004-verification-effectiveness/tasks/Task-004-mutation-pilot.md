---
id: Task-004
title: "Run the deterministic mutation pilot"
type: Task
status: blocked
track: C
priority: P0
relationships:
  - target: ix://agent-ix/tl-mltl/Task-002
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-013
    type: references
  - target: ix://agent-ix/tl-mltl/NFR-005
    type: references
  - target: ix://agent-ix/tl-mltl/TC-059
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-060
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-061
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-062
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-063
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-064
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-067
    type: verifies
---

# Task-004: Run the deterministic mutation pilot

## Scope

Implement complete mutant discovery, deterministic selection, isolated runs,
interleaved controls, conservative scoring, and reviewed missed/timeout routing.

## Subtasks

- [ ] Consume complete exact Quire `implements` bindings for the population.
- [ ] Freeze all discovered, excluded, selected, and unselected identities.
- [ ] Implement baseline/batch controls, restoration checks, and dispositions.

## Deliverables

- `tl-mltl.mutation-campaign/v1` records with complete raw JSON.
- Executing requirement-tagged tests for the assigned matrix rows.

## Notes

- Quire 0.31 provides the relation; complete and review tl-mltl's local bindings
  before selection and never infer a private relation in the producer.
