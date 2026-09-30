---
id: NFR-005
title: "Keep effectiveness evidence reproducible and bounded"
type: NFR
quality_attribute: reliability
relationships:
  - target: ix://agent-ix/tl-mltl/FR-020
    type: constrains
  - target: ix://agent-ix/tl-mltl/FR-021
    type: constrains
  - target: ix://agent-ix/tl-mltl/FR-022
    type: constrains
  - target: ix://agent-ix/tl-mltl/FR-023
    type: constrains
---

# NFR-005: Keep effectiveness evidence reproducible and bounded

## Statement

The verification-effectiveness campaign shall reproduce each declared finite
population and result from exact identities while preserving exclusions,
non-conclusive states, limitations, and human authority.

## Scope

Applies to every MRS-003 ledger, run, campaign, proof, summary, and retained
artifact. It constrains evidence
claims and does not alter temporal semantics.

## Rationale

Generated testing is easy to overstate: a count can hide an empty valid domain,
a fuzz plateau can hide changed instrumentation, a mutation score can omit
survivors or timeouts, and a bounded proof can hide assumptions or unwind
failures. Complete populations and immutable identities make those failures
reviewable.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Quire criteria classified exactly once | complete exported population | 100% | Test |
| Applicable property rows with complete grounding and discriminating oracle | complete applicable population | 100% | Test |
| Fuzz repetitions missing finite budgets, seeds, snapshots, stop reason, or retained corpus/crash identity | 0 | 0 | Test |
| Selected mutants missing identity, execution state, or missed/timeout disposition | 0 | 0 | Test |
| Bounded-proof claims missing assumptions, bounds, unwind/cover results, toolchain, or non-claims | 0 | 0 | Test |
| Domain/shared states collapsed or excluded from reported populations | 0 | 0 | Test |
| Current claims backed only by closed-unmerged or stale evidence | 0 | 0 | Test |
| New first-party non-Rust producer, validator, adapter, or audit path | 0 | 0 | Test |
| Automated source-release, qualification, certification, or monitor decisions | 0 | 0 | Test |

## Verification

Rust validators enumerate each finite population and verify schema, identity,
path, resource, state, and claim invariants. Mutation probes remove or
change one denominator, oracle, budget, stop, survivor, bound, assumption,
unwind, status, or artifact identity at a time and require the owning control to
name the defect. Quire reports static coverage. Independent review evaluates limitations and claim scope; only the
named human authority decides release.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-005-AC-1 | Repeating a deterministic census or replay with identical source, requirement, domain/population, seed, toolchain, configuration, environment, and artifact identities produces byte-identical ordered records; stochastic observations retain every repetition separately. | Test (TC-107, TC-114, TC-121, TC-127) |
| NFR-005-AC-2 | Deleting, duplicating, reclassifying, restamping, collapsing, or substituting any criterion, run, mutant, proof, outcome, bound, assumption, artifact, plan, build profile, limitation, or dependency makes its owning gate red and names the affected identity. | Test (TC-118, TC-120, TC-127, TC-128) |
| NFR-005-AC-3 | Every report states its exact finite population, exclusions, environment, result states, retention identity, and limitations before ratios and makes no automated release, qualification, certification, source-language, or monitor claim. | Test (TC-122) |

## Dependencies

Constrains FR-020 through FR-023 and relies on the MRS-002 corpus lifecycle
and applicable human authority.
