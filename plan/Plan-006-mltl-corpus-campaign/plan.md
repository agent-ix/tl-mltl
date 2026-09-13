---
id: Plan-006
title: "MLTL corpus and interoperability campaign"
type: Plan
status: active
relationships:
  - target: ix://agent-ix/tl-mltl/FR-008
    type: references
  - target: ix://agent-ix/tl-mltl/FR-009
    type: references
  - target: ix://agent-ix/tl-mltl/FR-010
    type: references
  - target: ix://agent-ix/tl-mltl/NFR-004
    type: references
---

# Implementation Plan: MLTL corpus and interoperability campaign

## Requirements Summary

### Functional Requirements

- [ ] **FR-008:** Implement the closed cell-obligation registry, exact
  identities, and fail-closed census.
- [ ] **FR-009:** Publish and retain singularly owned, versioned fixture
  families under bounded manifest and lifecycle rules.
- [ ] **FR-010:** Preserve closed loss, availability, and comparison states
  without external-runtime or qualification overclaim.

### Non-Functional Requirements

- [ ] **NFR-004:** Reproduce the exact population and preserve every gap,
  exclusion, limitation, and human-authority boundary.

## Dependency Graph

- `tl-mltl#38 acceptance -> Task-001..Task-008`
  Reason: specification existence is not implementation authorization.
- `Task-001 -> Task-002, Task-003, Task-004, Task-005, Task-006, Task-008`
  Reason: every producer and consumer needs the same closed campaign schema,
  cell preimage, limit profile, and typed refusal vocabulary.
- `Task-002 -> Task-003, Task-004, Task-005, Task-006`
  Reason: parser, rewrite, evaluator, and mapping overlays consume the
  authoritative formula/profile family rather than restamping it.
- `Task-002 + Task-003 + Task-004 + Task-005 + Task-006 -> Task-007`
  Reason: the integrated manifest/report gate consumes every current owner
  family and disposition producer.
- `quire-contract-ir#63 + #64 + Task-001 -> Task-008`
  Reason: native rows cannot become applicable before their owner publishes an
  accepted executable correspondence family.

NFR-004 constrains every task. Task-008 is a conditional successor lane and is
not on the current future/W/M critical path while its rows remain visibly
blocked.

### The seams

Task-001 adds the campaign types and validator beside the existing strict wire
types. Task-005 attaches only through the public `tl-syntax` canonical graph
and the current evaluation/horizon entry points. Task-006 extends the existing
mapping/differential record boundary without executing a target. Task-007 feeds
the resulting Rust producer record into the already-owned Quire/Quoin intake;
it creates no evidence store or approval mechanism.

## Test Plan

### Property tests

- [ ] **TC-038:** Enumerate the reviewed cell registry, require every dimension
  class, pin the ordered set digest, and discriminate census mutations.
- [ ] **TC-039:** Replay applicable semantic cells against isolated stored
  expectations.
- [ ] **TC-040:** Exercise trace, interval, witness, counterexample, and prefix
  progress boundaries.

### Integration tests

- [ ] **TC-037:** Round-trip strict campaign/cell schemas and exact digest
  preimages.
- [ ] **TC-041:** Refuse malformed, incompatible, stale, incomplete, and
  resource-exceeding cells at the declared stage.
- [ ] **TC-042:** Prove W/M source-to-canonical parity without a second
  evaluator semantic population.
- [ ] **TC-043:** Gate past/native rows on exact accepted producers and bind all
  anchor/history/capture/clock/result identities when admitted.
- [ ] **TC-044:** Round-trip each owner family and exercise every safe-path and
  resource-limit boundary.
- [ ] **TC-045:** Detect mutation, removal, restamping, unsafe artifacts, and
  invalid generated-input promotion.
- [ ] **TC-046:** Exercise the closed interoperability state table and preserve
  admission-refusal versus non-conclusive causes.
- [ ] **TC-047:** Turn every profile/operator/status/oracle/denominator mutation
  red.
- [ ] **TC-048:** Validate task/ticket ownership, consumers, methods,
  predecessors, resume conditions, language, and framework boundary.
- [ ] **TC-049:** Reproduce byte-identical ordered raw populations and reports.

## Remaining Work

### Track A: Critical Path (serial)

- **A1 = Task-001** campaign schema and census — Hard; exit: every accepted
  registry has one deterministic identity and every structural mutation
  refuses.
- **A2 = Task-002** syntax-owned canonical family — Medium; exit: current
  future/W/M bytes have one authoritative, replayable manifest.
- **A3 = Task-005** evaluator overlays and oracle — Hard; exit: every current
  applicable semantic cell agrees with an isolated expected result.
- **A4 = Task-007** integrated lifecycle/report gate — Hard; exit: the complete
  current campaign reproduces byte-identically and every denominator mutation
  turns red.

### Track B: Parallel owner families

- **B1 = Task-003** parser family — Medium; exit: parse/format/span cases replay
  from the syntax-owned identities.
- **B2 = Task-004** rewrite family — Medium; exit: equivalence pairs and
  counterexamples replay without a semantic fork.
- **B3 = Task-006** interoperability states — Medium; exit: every valid state
  combination round-trips and every invalid one refuses.

### Track C: Conditional successor

- **C1 = Task-008** native bridge consumption — Hard; exit: accepted #63/#64
  owner families move exact rows from blocked to applicable without erasure.

## Parallel Execution Summary

```text
accept #38 -> A1 -> A2 -> A3 -------------> A4
                    |-> B1 -----------------^
                    |-> B2 -----------------^
                    `-> B3 -----------------^
#63/#64 + A1 -----------------------------> C1 (conditional successor)
```

## Task File Mapping

| Task | Track | Owns (references) | Verified by (verifies) | Status |
|---|---|---|---|---|
| Task-001 | A | FR-008 | TC-037, TC-038, TC-041 | blocked |
| Task-002 | A | FR-008, FR-009 | TC-042, TC-044 | blocked |
| Task-003 | B | FR-009 | TC-044, TC-045 | blocked |
| Task-004 | B | FR-009 | TC-044, TC-045 | blocked |
| Task-005 | A | FR-008, FR-009 | TC-039, TC-040, TC-045 | blocked |
| Task-006 | B | FR-010 | TC-046, TC-048 | blocked |
| Task-007 | A | FR-009, NFR-004 | TC-044, TC-045, TC-047, TC-048, TC-049 | blocked |
| Task-008 | C | FR-008, FR-009, FR-010 | TC-043, TC-046 | blocked |

## Coordination Rules

- All tasks remain blocked until issue #38 records human acceptance of the
  exact reviewed revision; opening a GitHub ticket is not authorization.
- Each owner publishes one manifest. Consumers pin and replay it; they do not
  copy, restamp, or reinterpret it.
- Task-001 is the single writer for shared campaign wire types. Task-007 is the
  single writer for the integrated census/report path.
- Native, past/history, and FRETish successor rows stay blocked until their
  named owner contracts land. No fallback or foreign runtime is introduced.
- Every Rust PR uses `/rust-review`, then gap analysis and the repository local
  gate at the unchanged reviewed head. Hosted CI remains manual-dispatch only.
