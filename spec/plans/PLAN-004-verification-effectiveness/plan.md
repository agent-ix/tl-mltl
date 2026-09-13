---
id: PLAN-004
title: "Verification-effectiveness and bounded-proof implementation"
type: Plan
status: blocked
relationships:
  - target: ix://agent-ix/tl-mltl/FR-011
    type: references
  - target: ix://agent-ix/tl-mltl/FR-012
    type: references
  - target: ix://agent-ix/tl-mltl/FR-013
    type: references
  - target: ix://agent-ix/tl-mltl/FR-014
    type: references
  - target: ix://agent-ix/tl-mltl/FR-015
    type: references
  - target: ix://agent-ix/tl-mltl/NFR-005
    type: references
---

# PLAN-004: Verification-effectiveness and bounded-proof implementation

## Objective

Implement the tl-mltl portion of MRS-003 as finite, identity-bound property,
fuzz, mutation, and Kani evidence with lossless shared intake and no automated
release claim. Native Quire remains the sole authored formal-clause language;
this plan changes internal Rust verification infrastructure only.

## Requirements Summary

### Functional Requirements

- [ ] **FR-011:** Census and ground every exact Quire criterion.
- [ ] **FR-012:** Run bounded, retained fuzz campaigns at reviewed boundaries.
- [ ] **FR-013:** Run a deterministic Rust mutation pilot with complete outcomes.
- [ ] **FR-014:** Constrain and retain exact bounded Kani claims.
- [ ] **FR-015:** Route domain records through shared assurance without a local substitute.

### Non-Functional Requirements

- [ ] **NFR-005:** Keep every population, identity, limitation, and authority claim reproducible and bounded.

## Dependency Graph

- `landed M4 #44 + accepted MRS-003 -> Task-009`
  Reason: M0 and current W/M are landed, while implementation must still use an
  accepted M4 corpus/profile baseline.
- `Task-009 -> Task-010`
  Reason: the property ledger must freeze only after every applicable semantic
  profile and shared intake premise has an explicit admitted or blocked state.
- `Task-010 -> Task-011, Task-012, Task-013`
  Reason: fuzz, mutation, and Kani selections all consume stable criterion,
  requirement, domain, and priority identities from the grounded ledger.
- `complete exact Quire implements bindings -> Task-012`
  Reason: Quire 0.31 provides the relation, but the current tl-mltl export binds
  only six production symbols for FR-001 through FR-005; selection requires a
  complete reviewed applicable population and a tl-mltl-private mapping is
  forbidden.
- `raw Task-012 missed/timeout outcomes -> conditional Task-013 candidates`
  Reason: proof selection consumes an immutable execution outcome, not a final
  equivalent-mutant disposition that would make the dependency circular.
- `accepted Quoin #363 attachment + #364 build-profile contracts -> Task-011, Task-013, Task-014`
  Reason: binary artifacts and instrumented/model-check builds cannot be
  misrepresented in MeasurementCollection v2.
- `Task-010 + Task-011 + Task-012 + Task-013 -> Task-014 -> Task-015`
  Reason: shared intake is exercised only after native records exist; closure
  follows complete local evidence and mutation controls.

NFR-005 constrains every task. Current future/W/M rows are admitted at the exact
MRS-002 revisions; past/history rows remain blocked on #38 plus evaluator
support, and native rows on implemented quire-contract-ir #63/#64. Sibling
repositories own separate plans and native producers.

## The seams

The property ledger begins from Quire `properties --json` and the owning
TestMatrix. Native Rust producers attach to the existing evaluator, parser-like
input boundaries, cargo-mutants execution, and Kani harness seams. Quoin accepts
only the resulting wrapper and retained raw evidence; it does not run or infer
domain behavior.

## Test Plan

### Property ledger and grounding

- [ ] **TC-050:** Reject omitted, duplicate, stale, or unknown criterion rows.
- [ ] **TC-051:** Round-trip complete applicable, excluded, and blocked groundings.
- [ ] **TC-052:** Reject self-oracles, shared derivations, vacuity, and missed controls.
- [ ] **TC-053:** Reproduce generated/exhaustive runs, failures, and shrinking.
- [ ] **TC-054:** Refuse incomplete promotion of generated inputs to canonical corpus.

### Fuzz campaigns

- [ ] **TC-055:** Enforce seeds, dual caps, first-stop semantics, and observed budgets.
- [ ] **TC-056:** Admit plateau only from exact stable feature identities and full windows.
- [ ] **TC-057:** Preserve every fuzz stop and result state distinctly.
- [ ] **TC-058:** Retain and independently replay corpora and crashes.

### Mutation campaigns

- [ ] **TC-059:** Freeze and identity-bind the discovered population before selection.
- [ ] **TC-060:** Reproduce populations and reject unstable or partial controls.
- [ ] **TC-061:** Isolate mutants, interleave controls, and verify restoration.
- [ ] **TC-062:** Compute only the complete conservative killed fraction.
- [ ] **TC-063:** Route every missed/timeout mutant to one reviewed disposition.
- [ ] **TC-064:** Seed faults in population, state, denominator, isolation, and restoration.

### Bounded Kani claims

- [ ] **TC-065:** Census candidates and require every applicable proof check.
- [ ] **TC-066:** Mutate assumptions, bounds, checks, outcome, and claim scope.
- [ ] **TC-069:** Detect verifier-only production or MSRV divergence.
- [ ] **TC-070:** Reject closed-unmerged #35 evidence as current proof.

### Shared intake and claim boundaries

- [ ] **TC-067:** Bind source, tool, environment, artifact, and retention identities.
- [ ] **TC-068:** Refuse widened automated authority claims.
- [ ] **TC-071:** Enforce Rust/Quire/Quoin/Engineering-Assurance boundaries.
- [ ] **TC-072:** Prove shared tools do not execute or synthesize producer bytes.
- [ ] **TC-073:** Preserve shared/domain states and reject corrupt substitutions.
- [ ] **TC-074:** Refuse unsafe paths and resource-cap violations before decode.
- [ ] **TC-075:** Refuse invalid plan and verification-stack bindings.

## Remaining Work

### Track Gate: Admission (serial)

- **Gate = Task-009** M5 admission snapshot — Medium; exit: every prerequisite is bound to an exact accepted revision or a typed blocked state, with no local substitute.

### Track A: Critical Path (serial)

- **A1 = Task-010** property ledger and grounded runner — Hard; exit: the complete criterion population reproduces with discriminating oracles and explicit limitations.
- **A2 = Task-014** shared intake and receipts — Hard; exit: each admitted native record round-trips losslessly through compatible shared contracts.
- **A3 = Task-015** full campaign closure — Medium; exit: all applicable rows execute, all blocked rows stay visible, and every mutation control fails red.

### Track B: Post-ledger fuzz

- **B1 = Task-011** retained fuzz baseline — Hard; exit: each applicable target completes the declared seeded budget or retains its exact non-conclusive state.

### Track C: Post-ledger mutation

- **C1 = Task-012** deterministic mutation pilot — Hard; exit: frozen populations, controls, conservative score, and every missed/timeout disposition reproduce.

### Track D: Post-ledger bounded proof

- **D1 = Task-013** Kani candidate ledger and harnesses — Hard; exit: each admitted finite proposition has an exact result and no claim widening.

## Parallel Execution Summary

```text
Gate: Task-009
          |
          v
       Task-010
       /   |   \
      v    v    v
 Task-011 Task-012 Task-013
      \    |    /
       v   v   v
       Task-014
          |
          v
       Task-015
```

## Task File Mapping

| Task | Track | Owns (references) | Verified by (verifies) | Status |
|---|---|---|---|---|
| Task-009 | Gate | FR-011, FR-015, NFR-005 | TC-050, TC-075 | blocked |
| Task-010 | A | FR-011, NFR-005 | TC-050 through TC-054 | blocked |
| Task-011 | B | FR-012, NFR-005 | TC-055 through TC-058, TC-067, TC-068 | blocked |
| Task-012 | C | FR-013, NFR-005 | TC-059 through TC-064, TC-067 | blocked |
| Task-013 | D | FR-014, NFR-005 | TC-065 through TC-070 | blocked |
| Task-014 | A | FR-015, NFR-005 | TC-067, TC-068, TC-071 through TC-075 | blocked |
| Task-015 | A | FR-011 through FR-015, NFR-005 | TC-050 through TC-075 | blocked |

## Coordination Rules

- Do not implement until #44/MRS-002 is landed and MRS-003 is independently
  accepted; preserve exact parent-before-child merge order.
- Current future/W/M work uses the exact landed revisions. Do not implement
  past/history or native-predicate rows before their named semantic
  prerequisites. A blocked row receives no coverage credit.
- Quire owns the `implements` relation contract and tl-mltl owns complete local
  annotations. Reusable Quoin #363 attachment, #364 build-profile, and
  local-plan contracts remain single-writer shared work. Do not create local
  stand-ins.
- Freeze the ledger/population before parallel producer work. Schema changes
  land before producers; producer records land before shared-intake fixtures.
- Hosted CI remains manual-only and is not dispatched by this plan.
