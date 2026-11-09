---
id: TM-006
title: Past-profile evaluation test matrix
type: TestMatrix
relationships:
  - target: ix://agent-ix/tl-mltl/FR-042
    type: covers
---

# Past-profile evaluation test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-042 | FR-042-AC-1 through FR-042-AC-9 | TC-178 through TC-186 | ✅ implemented |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-178 | Evaluate O/H against an independent reverse-offset oracle across Boolean, endpoint, singleton, and pre-origin cases under both clocks | Property | P0 | FR-042-AC-1 | ✅ implemented |
| TC-179 | Evaluate S against the oracle, with the exact reverse `[a,j)` left range and inclusive witness endpoints | Property | P0 | FR-042-AC-2 | ✅ implemented |
| TC-180 | Evaluate T against the Boolean dual of S and the oracle across all bounded and origin cases | Property | P0 | FR-042-AC-3 | ✅ implemented |
| TC-181 | Evaluate Y against O[1,1] for propositions, constants and Boolean nests at every anchor | Property | P0 | FR-042-AC-4 | ✅ implemented |
| TC-182 | Refuse each history completeness, ordering, digest, revision, identity and anchor dimension with a distinct error | Unit | P0 | FR-042-AC-5 | ✅ implemented |
| TC-183 | Compute required history by the recursive equations, span-independent, and refuse a non-past formula | Unit | P0 | FR-042-AC-6 | ✅ implemented |
| TC-184 | Refuse required-history `u64` overflow at the checked-addition boundary | Unit | P0 | FR-042-AC-7 | ✅ implemented |
| TC-185 | Admit both clocks exactly; refuse overflow, malformed, non-positive, unsupported, and mismatched bindings; preserve owner non-values | Unit | P0 | FR-042-AC-8 | ✅ implemented |
| TC-186 | Refuse each evaluation resource limit with its typed error and no verdict | Unit | P0 | FR-042-AC-9 | ✅ implemented |
