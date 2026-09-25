---
id: FR-050
title: Measure real code coverage with llvm-cov
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-045
    type: depends_on
---

# FR-050: Measure real code coverage with llvm-cov

## Description

When a V1 coverage report is published, each owning crate shall record
llvm-cov line and branch coverage for its production source and identify
uncovered semantic or refusal branches.

## Behavior

The report states toolchain, build profile, feature set, test selection and
source revision. It separates production source from generated/test code.
Critical operator and typed-refusal branches have a 100% executed-branch
target; any uncovered branch is listed by file and line with a test or
reviewed infeasibility reason. Aggregate percentage cannot hide an uncovered
critical branch, and coverage is not itself semantic correctness.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-050-AC-1 | llvm-cov reports each crate and feature set with exact source identity, critical branch census and uncovered locations. | Test (TC-189) |

## Dependencies

FR-045 owns the tests measured by coverage.
