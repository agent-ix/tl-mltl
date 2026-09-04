---
id: PLAN-003
title: "Close shared-assurance review residuals"
type: Plan
status: complete
relationships:
  - target: ix://agent-ix/tl-mltl/NFR-003
    type: references
---
# PLAN-003: Close shared-assurance review residuals

## Objective

Close the concrete concurrency, probe-hermeticity, census-falsifiability and
record-accuracy findings left by the exact-head reviews of PRs #21 and #22,
without adding a repository-local evidence framework or Make parser.

## Approach

1. Make the shared-input mutex a required token at every helper that reads or
   mutates the protected state; serialize the two omitted file reader/writer
   tests and make poison a distinct failure.
2. Apply one clean-store-safe containment helper and the explicit symlink
   cleanup rationale to both scratch probes, and make stale producer-shim
   cleanup fail closed.
3. Strengthen TC-024's retained controls: discriminate the plain compatibility
   target name, reject renamed legacy proof IDs, state that `census_controls` is
   unsealed, and exercise Git template/excludes isolation.
4. Update NFR-003, TC-019 and the review record so the positive control,
   serialization boundary and actual execution order are declared.
5. Run the focused shared-assurance target, specification validation and the
   complete local `make ci` gate before review.

## Scope

In scope: `tests/shared_assurance.rs`, NFR-003, TC-019, the change-assurance
declaration, and the retained review/plan artifacts that describe those
controls.

Out of scope: the broader Make execution-control qualification class in #14,
any shared runner, certification, release or hosted-CI action, and temporal
evaluation semantics.

## Landing constraints

- Hosted CI remains manual-dispatch only and is not dispatched by this work.
- No negative probe counts unless the unmodified subject succeeds in the same
  fixture.
- A missing, unreadable or poisoned state is a named failure, never a skip or a
  recovery that reports an ordinary measurement.
- The exact PR head is independently reviewed before any administrative merge.
