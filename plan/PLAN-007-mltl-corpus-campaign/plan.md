---
id: PLAN-007
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

- `tl-mltl#38 acceptance -> Task-010..Task-016`
  Reason: specification existence is not implementation authorization.
- `Task-010 -> Task-011, Task-012, Task-013, Task-014, Task-015`
  Reason: every producer and consumer needs the same closed campaign schema,
  cell preimage, limit profile, and typed refusal vocabulary.
- `Task-011 -> Task-012, Task-013, Task-014, Task-015`
  Reason: parser, rewrite, evaluator, and mapping overlays consume the
  authoritative formula/profile family rather than restamping it.
- `Task-011 + Task-012 + Task-013 + Task-014 + Task-015 -> Task-016`
  Reason: the integrated manifest/report gate consumes every current owner
  family and disposition producer.

NFR-004 constrains every task. This plan routes only TL-owned families: the
native-predicate and native-temporal correspondence dimension is specified and
routed in `quire-mltl`, the dedicated bridge crate, and is not a lane here (see
MRS-002 and [ADR-002](../../spec/decisions/ADR-002-native-correspondence-lives-in-quire-mltl.md)).

### The seams

Task-010 adds the campaign types and validator beside the existing strict wire
types. Task-014 attaches only through the public `tl-syntax` canonical graph
and the current evaluation/horizon entry points. Task-015 extends the existing
mapping/differential record boundary without executing a target. Task-016 feeds
the resulting Rust producer record into the already-owned Quire/Quoin intake;
it creates no evidence store or approval mechanism.

## Test Plan

### Property tests

- [ ] **TC-092:** Enumerate the reviewed cell registry, require every dimension
  class, pin the ordered set digest, and discriminate census mutations.
- [ ] **TC-093:** Replay applicable semantic cells against isolated stored
  expectations.
- [ ] **TC-094:** Exercise trace, interval, witness, counterexample, and prefix
  progress boundaries.

### Integration tests

- [ ] **TC-091:** Round-trip strict campaign/cell schemas and exact digest
  preimages.
- [ ] **TC-095:** Refuse malformed, incompatible, stale, incomplete, and
  resource-exceeding cells at the declared stage.
- [ ] **TC-096:** Prove W/M source-to-canonical parity without a second
  evaluator semantic population.
- [ ] **TC-097:** Gate past/history rows on the exact accepted tl-syntax profile
  and routed evaluator revision, and bind all anchor/history/capture/clock/result
  identities when admitted.
- [ ] **TC-098:** Round-trip each owner family and exercise every safe-path and
  resource-limit boundary.
- [ ] **TC-099:** Detect mutation, removal, restamping, unsafe artifacts, and
  invalid generated-input promotion.
- [ ] **TC-100:** Exercise the closed interoperability state table and preserve
  admission-refusal versus non-conclusive causes.
- [ ] **TC-101:** Turn every profile/operator/status/oracle/denominator mutation
  red.
- [ ] **TC-102:** Validate task/ticket ownership, consumers, methods,
  predecessors, resume conditions, language, and framework boundary.
- [ ] **TC-103:** Reproduce byte-identical ordered raw populations and reports.

## Remaining Work

### Track A: Critical Path (serial)

- **A1 = Task-010** campaign schema and census — Hard; exit: every accepted
  registry has one deterministic identity and every structural mutation
  refuses.
- **A2 = Task-011** syntax-owned canonical family — Medium; exit: current
  future/W/M bytes have one authoritative, replayable manifest.
- **A3 = Task-014** evaluator overlays and oracle — Hard; exit: every current
  applicable semantic cell agrees with an isolated expected result.
- **A4 = Task-016** integrated lifecycle/report gate — Hard; exit: the complete
  current campaign reproduces byte-identically and every denominator mutation
  turns red.

### Track B: Parallel owner families

- **B1 = Task-012** parser family — Medium; exit: parse/format/span cases replay
  from the syntax-owned identities.
- **B2 = Task-013** rewrite family — Medium; exit: equivalence pairs and
  counterexamples replay without a semantic fork.
- **B3 = Task-015** interoperability states — Medium; exit: every valid state
  combination round-trips and every invalid one refuses.

## Parallel Execution Summary

```text
accept #38 -> Task-010 -> Task-011 -> Task-014 -------------> Task-016
                                 |-> Task-012 -----------------^
                                 |-> Task-013 -----------------^
                                 `-> Task-015 -----------------^
```

## Task File Mapping

| Task | Track | Owns (references) | Verified by (verifies) | Status |
|---|---|---|---|---|
| Task-010 | A | FR-008 | TC-091, TC-092, TC-095 | blocked |
| Task-011 | A | FR-008, FR-009 | TC-096, TC-098 | blocked |
| Task-012 | B | FR-009 | TC-098, TC-099 | blocked |
| Task-013 | B | FR-009 | TC-098, TC-099 | blocked |
| Task-014 | A | FR-008, FR-009 | TC-093, TC-094, TC-097, TC-099 | blocked |
| Task-015 | B | FR-010 | TC-100, TC-102 | blocked |
| Task-016 | A | FR-009, NFR-004 | TC-098, TC-099, TC-101, TC-102, TC-103 | blocked |

## Coordination Rules

- All tasks remain blocked until issue #38 records human acceptance of the
  exact reviewed revision; opening a GitHub ticket is not authorization.
- Each owner publishes one manifest. Consumers pin and replay it; they do not
  copy, restamp, or reinterpret it.
- Task-010 is the single writer for shared campaign wire types. Task-016 is the
  single writer for the integrated census/report path.
- Past/history successor rows stay blocked until the accepted tl-syntax profile
  and its routed evaluator implementation land. No fallback or foreign runtime
  is introduced.
- Every Rust PR uses `/rust-review`, then gap analysis and the repository local
  gate at the unchanged reviewed head. Hosted CI remains manual-dispatch only.
