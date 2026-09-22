---
id: PLAN-008
title: "Verification-effectiveness and bounded-proof implementation"
type: Plan
status: blocked
relationships:
  - target: ix://agent-ix/tl-mltl/FR-020
    type: references
  - target: ix://agent-ix/tl-mltl/FR-021
    type: references
  - target: ix://agent-ix/tl-mltl/FR-022
    type: references
  - target: ix://agent-ix/tl-mltl/FR-023
    type: references
  - target: ix://agent-ix/tl-mltl/FR-024
    type: references
  - target: ix://agent-ix/tl-mltl/NFR-005
    type: references
---

# PLAN-008: Verification-effectiveness and bounded-proof implementation

## Objective

Implement the tl-mltl portion of MRS-003 as finite, identity-bound property,
fuzz, mutation, and Kani evidence with lossless shared intake and no automated
release claim. This plan changes internal Rust verification infrastructure
only, and routes no cross-repository producer dependency.

## Requirements Summary

### Functional Requirements

- [ ] **FR-020:** Census and ground every exact Quire criterion.
- [ ] **FR-021:** Run bounded, retained fuzz campaigns at reviewed boundaries.
- [ ] **FR-022:** Run a deterministic Rust mutation pilot with complete outcomes.
- [ ] **FR-023:** Constrain and retain exact bounded Kani claims.
- [ ] **FR-024:** Route domain records through shared assurance without a local substitute.

### Non-Functional Requirements

- [ ] **NFR-005:** Keep every population, identity, limitation, and authority claim reproducible and bounded.

## Dependency Graph

- `landed MRS-002/PLAN-007 + accepted MRS-003 -> Task-017`
  Reason: M0 and current W/M are landed, while implementation must still use an
  accepted M4 corpus/profile baseline.
- `Task-017 -> Task-018`
  Reason: the property ledger must freeze only after every applicable semantic
  profile and shared intake premise has an explicit admitted or blocked state.
- `Task-018 -> Task-019, Task-020, Task-021`
  Reason: fuzz, mutation, and Kani selections all consume stable criterion,
  requirement, domain, and priority identities from the grounded ledger.
- `complete exact Quire implements bindings -> Task-020`
  Reason: Quire provides the relation, but the export is not self-evidently
  populated. Measured at this head with the installed `quire 0.32.2`, the
  `symbols` export binds zero `implements` edges while `src/` carries six
  `Implements:` annotations for FR-001 through FR-005. Selection requires a
  complete reviewed applicable population proved by a control, and a
  tl-mltl-private mapping is forbidden.
- `raw Task-020 missed/timeout outcomes -> conditional Task-021 candidates`
  Reason: proof selection consumes an immutable execution outcome, not a final
  equivalent-mutant disposition that would make the dependency circular.
- `accepted Quoin #363 attachment + #364 build-profile contracts -> Task-019, Task-021, Task-022`
  Reason: binary artifacts and instrumented/model-check builds cannot be
  misrepresented in MeasurementCollection v2.
- `Task-018 + Task-019 + Task-020 + Task-021 -> Task-022 -> Task-023`
  Reason: shared intake is exercised only after native records exist; closure
  follows complete local evidence and mutation controls.

NFR-005 constrains every task. Current future/W/M rows are admitted at the
exact MRS-002 revisions, and past/history rows remain blocked on tl-syntax #38
plus evaluator support — the plan's only external resume condition, and it is
TL-internal. The native-correspondence bridge is not measured here: it is
`quire-mltl`'s under
[ADR-002](../../spec/decisions/ADR-002-native-correspondence-lives-in-quire-mltl.md).
Sibling repositories own separate plans and owner-native producers.

## The seams

The property ledger begins from Quire `properties --json` and the owning
TestMatrix. Native Rust producers attach to the existing evaluator, parser-like
input boundaries, cargo-mutants execution, and Kani harness seams. Quoin accepts
only the resulting wrapper and retained raw evidence; it does not run or infer
domain behavior.

## Test Plan

### Property ledger and grounding

- [ ] **TC-104:** Reject omitted, duplicate, stale, or unknown criterion rows.
- [ ] **TC-105:** Round-trip complete applicable, excluded, and blocked groundings.
- [ ] **TC-106:** Reject self-oracles, shared derivations, vacuity, and missed controls.
- [ ] **TC-107:** Reproduce generated/exhaustive runs, failures, and shrinking.
- [ ] **TC-108:** Refuse incomplete promotion of generated inputs to canonical corpus.

### Fuzz campaigns

- [ ] **TC-109:** Enforce seeds, dual caps, first-stop semantics, and observed budgets.
- [ ] **TC-110:** Admit plateau only from exact stable feature identities and full windows.
- [ ] **TC-111:** Preserve every fuzz stop and result state distinctly.
- [ ] **TC-112:** Retain and independently replay corpora and crashes.

### Mutation campaigns

- [ ] **TC-113:** Freeze and identity-bind the discovered population before selection.
- [ ] **TC-114:** Reproduce populations and reject unstable or partial controls.
- [ ] **TC-115:** Isolate mutants, interleave controls, and verify restoration.
- [ ] **TC-116:** Compute only the complete conservative killed fraction.
- [ ] **TC-117:** Route every missed/timeout mutant to one reviewed disposition.
- [ ] **TC-118:** Seed faults in population, state, denominator, isolation, and restoration.

### Bounded Kani claims

- [ ] **TC-119:** Census candidates and require every applicable proof check.
- [ ] **TC-120:** Mutate assumptions, bounds, checks, outcome, and claim scope.
- [ ] **TC-123:** Detect verifier-only production or MSRV divergence.
- [ ] **TC-124:** Reject closed-unmerged #35 evidence as current proof.

### Shared intake and claim boundaries

- [ ] **TC-121:** Bind source, tool, environment, artifact, and retention identities.
- [ ] **TC-122:** Refuse widened automated authority claims.
- [ ] **TC-125:** Enforce Rust/Quire/Quoin/Engineering-Assurance boundaries.
- [ ] **TC-126:** Prove shared tools do not execute or synthesize producer bytes.
- [ ] **TC-127:** Preserve shared/domain states and reject corrupt substitutions.
- [ ] **TC-128:** Refuse unsafe paths and resource-cap violations before decode.
- [ ] **TC-129:** Refuse invalid plan and verification-stack bindings.

## Remaining Work

### Track Gate: Admission (serial)

- **Gate = Task-017** M5 admission snapshot — Medium; exit: every prerequisite is bound to an exact accepted revision or a typed blocked state, with no local substitute.

### Track A: Critical Path (serial)

- **A1 = Task-018** property ledger and grounded runner — Hard; exit: the complete criterion population reproduces with discriminating oracles and explicit limitations.
- **A2 = Task-022** shared intake and receipts — Hard; exit: each admitted native record round-trips losslessly through compatible shared contracts.
- **A3 = Task-023** full campaign closure — Medium; exit: all applicable rows execute, all blocked rows stay visible, and every mutation control fails red.

### Track B: Post-ledger fuzz

- **B1 = Task-019** retained fuzz baseline — Hard; exit: each applicable target completes the declared seeded budget or retains its exact non-conclusive state.

### Track C: Post-ledger mutation

- **C1 = Task-020** deterministic mutation pilot — Hard; exit: frozen populations, controls, conservative score, and every missed/timeout disposition reproduce.

### Track D: Post-ledger bounded proof

- **D1 = Task-021** Kani candidate ledger and harnesses — Hard; exit: each admitted finite proposition has an exact result and no claim widening.

## Parallel Execution Summary

```text
Gate: Task-017
          |
          v
       Task-018
       /   |   \
      v    v    v
 Task-019 Task-020 Task-021
      \    |    /
       v   v   v
       Task-022
          |
          v
       Task-023
```

## Task File Mapping

| Task | Track | Owns (references) | Verified by (verifies) | Status |
|---|---|---|---|---|
| Task-017 | Gate | FR-020, FR-024, NFR-005 | TC-104, TC-129 | blocked |
| Task-018 | A | FR-020, NFR-005 | TC-104 through TC-108 | blocked |
| Task-019 | B | FR-021, NFR-005 | TC-109 through TC-112, TC-121, TC-122 | blocked |
| Task-020 | C | FR-022, NFR-005 | TC-113 through TC-118, TC-121 | blocked |
| Task-021 | D | FR-023, NFR-005 | TC-119 through TC-124 | blocked |
| Task-022 | A | FR-024, NFR-005 | TC-121, TC-122, TC-125 through TC-129 | blocked |
| Task-023 | A | FR-020 through FR-024, NFR-005 | TC-104 through TC-129 | blocked |

## Coordination Rules

- Do not implement until MRS-002/PLAN-007 is landed and MRS-003 is
  independently accepted; preserve exact parent-before-child merge order.
- Current future/W/M work uses the exact landed revisions. Do not implement
  past/history rows before their named semantic prerequisites. A blocked row
  receives no coverage credit.
- Quire owns the `implements` relation contract and tl-mltl owns complete local
  annotations. Reusable Quoin #363 attachment, #364 build-profile, and
  local-plan contracts remain single-writer shared work. Do not create local
  stand-ins.
- Freeze the ledger/population before parallel producer work. Schema changes
  land before producers; producer records land before shared-intake fixtures.
- Hosted CI remains manual-only and is not dispatched by this plan.
