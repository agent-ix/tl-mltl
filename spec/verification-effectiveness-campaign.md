---
id: MRS-003
title: "MLTL verification-effectiveness and bounded-proof campaign"
type: MasterRequirements
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-001
    type: depends_on
  - target: ix://agent-ix/tl-mltl/MRS-002
    type: depends_on
  - target: ix://agent-ix/tl-mltl/issues/39
    type: references
---

# MLTL verification-effectiveness and bounded-proof campaign

## Purpose

This specification defines how the TL ecosystem measures property-domain
coverage, retained fuzz campaigns, mutation effectiveness, and bounded Kani
proofs without converting test counts, elapsed fuzz time, mutation ratios, or
one bounded proof into claims about unbounded semantic correctness.

Native Quire remains the sole editable formal-clause language. The TL crates
remain internal Rust representations, parsers, rewriters, reference evaluators,
and mapping infrastructure. This campaign measures those components; it does
not create a TL authoring language or qualify an external monitor.

## Scope

### In scope

- A closed criterion-to-property census grounded from exact Quire exports.
- Reproducible generated and exhaustive property runs with independent oracles,
  explicit domains, seeds, discard counts, shrinking, and limitations.
- Budgeted cargo-fuzz campaigns with retained starting/final corpora, crashes,
  coverage observations, stopping reasons, and a bounded plateau diagnostic.
- A deterministic Rust mutation-testing pilot with a pre-run population,
  conclusive and non-conclusive states, and reviewed survivor dispositions.
- Narrow Kani proof claims that bind exact harnesses, assumptions, bounds,
  unwind checks, solver/toolchain identities, outcomes, and counterexamples.
- Domain-produced structured records mapped through the existing Engineering
  Assurance, Quire, and Quoin boundary.

### Out of scope

- New temporal operators, semantics, native Quire grammar, parsers, rewriters,
  evaluator branches, mapping behavior, or production monitoring.
- An exhaustive claim over all formulas, intervals, traces, histories, clocks,
  captures, native predicates, or target behaviors.
- Treating a property count, fuzz duration, plateau, no-crash result, mutation
  score, or bounded proof as source release, qualification, certification, or
  human approval.
- Loom for the non-concurrent semantic core, or adopting Verus, concolic
  execution, a foreign runtime, or a repository-local assurance framework.
- Executing R2U2, C2PO, FRET/Electron, Java, Node, or another foreign runtime in
  a production or qualification path.

## Requirements architecture

[FR-011](./requirements/FR-011-property-obligation-ledger.md) owns the complete
criterion/property ledger and grounded run contract.
[FR-012](./requirements/FR-012-budgeted-fuzz-campaigns.md) owns fuzz campaign
budgets, stopping, plateau, and crash handling.
[FR-013](./requirements/FR-013-measured-mutation-campaign.md) owns mutation
population identity, execution states, scores, and survivor disposition.
[FR-014](./requirements/FR-014-bounded-kani-claims.md) owns bounded Kani claim
semantics. [FR-015](./requirements/FR-015-shared-effectiveness-intake.md) owns
the shared producer/intake boundary. [NFR-005](./requirements/NFR-005-truthful-effectiveness-evidence.md)
constrains reproducibility, retention, and claim language. MP-003 through MP-006
define the four measurements, and TM-003 assigns planned evidence.

## Admission and dependency order

Specification and review may proceed while parent work is pending. M0 is
already landed at tl-mltl v0.1.0 (`4bff387`), and the W/M specification and
routed implementations are landed at the exact revisions named by MRS-002.
Current future/W/M obligations therefore cannot remain blocked merely because
this campaign predates those merges.

No campaign implementation begins until M4 PR #44 and MRS-002 are independently
accepted and landed and MRS-003 itself is human-accepted at an exact reviewed
revision. Past/history obligations remain blocked on tl-syntax PR #38 plus its
routed evaluator support; native-predicate and temporal-bridge obligations
remain blocked on the accepted and implemented quire-contract-ir #63/#64 path.
A blocked row earns no coverage or effectiveness credit.

Implementation follows this order:

1. Each owning repository completes the FR-011 criterion census and grounded
   property baseline using Quoin `spec-correctness`; the already merged
   tl-mltl property test is historical input, not a complete ledger.
2. Owners run FR-012 only at reviewed byte-oriented or structured-input fuzz
   boundaries selected from the evidence catalog. Generated inputs remain
   outside the canonical MRS-002 corpus unless promoted under FR-009.
3. Owners may run FR-013 against an exact stable-green baseline and immutable
   selected mutant population, in parallel with applicable fuzz campaigns, then
   route every missed or timed-out mutant by reviewed priority.
4. Owners select FR-014 harnesses from property gaps, retained fuzz
   plateaus/counterexamples, mutation dispositions, or an exact reviewed
   bounded-arithmetic proposition. Matrix priority orders admitted candidates
   but is not by itself a Kani-candidate trigger. A primitive proof cannot be
   widened to its caller or to the evaluator as a whole.
5. Rust domain producers emit the records; Quire supplies static facts and
   Quoin validates, binds, retains, and presents them for human review.

Sibling implementation remains tracked by tl-syntax #26, tl-parse #25, and
tl-rewrite #27. Historical tl-mltl #31 supplies the landed property baseline and
bounded-Kani feasibility result but explicitly excluded immediate fuzzing and
mutation; it is not repurposed as the M5 implementation ticket. PLAN-004 is
mirrored by tl-mltl #58, #56, #60, #61, #59, #57, and #62 in task order. Every
new ticket remains blocked until the exact specification and predecessor gates
recorded by the plan are satisfied. Any changed semantics discovered by those
tickets returns to `/specify` and `/spec-review` before implementation.

Each repository owns and versions its own native result schemas. Identities in
this MRS use `tl-mltl.*` only for tl-mltl examples; sibling repositories use
their own namespaces while preserving the common required fields. This campaign
does not make tl-mltl the owner of a cross-repository runtime, schema package,
or evidence framework.
