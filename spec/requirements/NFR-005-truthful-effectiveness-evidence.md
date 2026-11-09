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

The verification-effectiveness campaign shall reproduce each result from a recorded
source and tool versions while reporting every non-passing state and limitation,
and shall leave release and qualification decisions to humans.

## Scope

Applies to every property run, fuzz run, mutation run, Kani proof, and campaign
report in MRS-003. It constrains evidence claims and does not alter temporal
semantics.

## Rationale

Generated testing is easy to overstate: a count can hide an empty domain, a fuzz
plateau can hide a changed build, a kill rate can omit survivors and timeouts, and
a bounded proof can hide its assumptions or an unfinished unwind. Recorded seeds,
raw counts, and stated bounds make those failures reviewable.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Property-law tests with a recorded seed, case count, and discriminating oracle | every FR-020 law | 100% | Test |
| Fuzz runs missing a seed, budget, stop reason, or coverage record | 0 | 0 | Test |
| Surviving mutants without a listed reason | 0 | 0 | Test |
| Kani harnesses missing stated bounds or non-claims | 0 | 0 | Test |
| Campaign members reported as pass when not run, unavailable, or planned | 0 | 0 | Test |
| Automated source-release, qualification, certification, or monitor decisions | 0 | 0 | Test |

## Verification

Rust tests check the recorded seeds and counts, the campaign report, and the
inventory of evidence-bearing make targets and fuzz targets. Seeded faults and
weakened test selections confirm the controls can fail. Independent review
evaluates limitations and claim scope, and only the named human authority decides
release.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-005-AC-1 | Repeating a deterministic run at the same source, seed, toolchain, and configuration reproduces its results, a replay of a retained campaign report refuses a moved or dirty source, and stochastic runs keep every repetition separately. | Test (TC-107, TC-114, TC-121, TC-127) |
| NFR-005-AC-2 | A dropped campaign member, an unregistered evidence target, a collapsed non-pass state, a weakened test selection, or a seeded fault makes its owning check fail and names what changed. | Test (TC-118, TC-120, TC-127) |
| NFR-005-AC-3 | Every report states its populations, seeds, tool versions, result states, and limitations before any ratio, and makes no automated release, qualification, certification, or monitor claim. | Test (TC-122) |

## Dependencies

Constrains FR-020 through FR-023 and relies on the MRS-002 corpus boundary between
generated and canonical inputs.
