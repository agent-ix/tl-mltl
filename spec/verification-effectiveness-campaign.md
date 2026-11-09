---
id: MRS-003
title: "MLTL verification-effectiveness and bounded-proof campaign"
type: MasterRequirements
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-001
    type: depends_on
  - target: ix://agent-ix/tl-mltl/MRS-002
    type: depends_on
---

# MLTL verification-effectiveness and bounded-proof campaign

## Purpose

This specification defines how the TL ecosystem measures how well its tests catch
defects: property laws checked against an independent oracle, fuzz runs, a
mutation kill rate, and bounded Kani proofs. It does not convert test counts, fuzz
time, a kill rate, or one bounded proof into a claim of unbounded semantic
correctness.

The TL crates are Rust representations, parsers, rewriters, reference
evaluators, and mapping infrastructure. This campaign measures those components;
it creates no authoring language and qualifies no external monitor. Every
obligation here is TL-internal, consistent with the TL-175 ruling that each TL-*
crate stays independent of the agent-ix/Quire ecosystem.

## Scope

### In scope

- Property laws and exhaustive small-scope sweeps over the evaluators, each
  with a recorded seed or enumerated domain, compared with the independent
  oracle, and with seeded-fault controls.
- Fuzz targets at byte-oriented and structured-input boundaries, each run under
  several seeds with a fixed budget and a plateau stop, with every crash turned
  into a regression test.
- A cargo-mutants kill rate with thresholds, a rule that the unmutated baseline
  is green, and a listed reason for every surviving mutant.
- Kani harnesses that state their bounds and non-claims, guard against vacuous
  proofs, and add no runtime dependency.
- One campaign report that records tool versions, seeds, and outcomes per
  member, and replays against the recorded source.

### Out of scope

- New temporal operators, semantics, parsers, rewriters, evaluator branches,
  mapping behavior, or production monitoring.
- An exhaustive claim over all formulas, intervals, traces, histories, clocks,
  captures, or target behaviors.
- Effectiveness measurement of the native-correspondence bridge, which
  `quire-mltl` owns under
  [ADR-002](./decisions/ADR-002-native-correspondence-lives-in-quire-mltl.md).
- Treating a property count, fuzz duration, plateau, no-crash result, kill rate,
  or bounded proof as source release, qualification, certification, or human
  approval.
- Loom for the non-concurrent semantic core, Verus, concolic execution, a foreign
  runtime, or a repository-local assurance framework.
- Executing R2U2, C2PO, FRET/Electron, Java, Node, or another foreign runtime in
  a production or qualification path.

## Requirements architecture

[FR-020](./requirements/FR-020-property-obligation-ledger.md) owns the evaluator
property laws, their seeds, and the independent-oracle comparison.
[FR-021](./requirements/FR-021-budgeted-fuzz-campaigns.md) owns fuzz targets,
seeds, budgets, the plateau stop, and crash handling.
[FR-022](./requirements/FR-022-measured-mutation-campaign.md) owns the mutation
kill rate, thresholds, and survivor reasons.
[FR-023](./requirements/FR-023-bounded-kani-claims.md) owns the Kani harness
list and what a bounded proof may claim.
[NFR-005](./requirements/NFR-005-truthful-effectiveness-evidence.md) constrains
reproducibility and claim language, and TM-003 assigns evidence.

## How the techniques relate

Property laws and sweeps are the baseline suite. Fuzzing covers inputs the
laws do not generate, and each crash it finds becomes a committed regression
test. Mutation testing measures whether that suite would notice a defect, and a
survivor is either killed by a new test, recorded as equivalent with a reason, or
routed to a Kani proof. Kani checks all inputs within stated bounds for the
arithmetic and panic-freedom properties a runtime monitor depends on. Generated
inputs stay outside the canonical corpus unless promoted under FR-009. A bounded
proof of a primitive is never widened to its caller or to the evaluator as a
whole.

Any changed semantics discovered while running these techniques returns to
`/specify` and `/spec-review` before implementation. Each repository versions
its own result files.
