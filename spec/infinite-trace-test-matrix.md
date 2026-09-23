---
id: TM-004
title: MLTL V1 infinite-trace provider test matrix
type: TestMatrix
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-001
    type: covers
---

# MLTL V1 infinite-trace provider test matrix

Implementation statuses below reflect only traced tests. Independent
`tl-oracle` comparison rows remain planned until a persistent test lane lands.

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|---|---|---|---|
| FR-027 | FR-027-AC-1 through FR-027-AC-3 | TC-086, TC-138 | 🚧 planned |
| FR-028 | FR-028-AC-1 through FR-028-AC-3 | TC-087, TC-138, TC-139 | 🚧 planned |
| FR-029 | FR-029-AC-1 through FR-029-AC-3 | TC-088, TC-089, TC-140 | 🚧 planned |
| FR-030 | FR-030-AC-1 through FR-030-AC-3 | TC-141 through TC-144 | 🚧 planned |
| FR-031 | FR-031-AC-1 through FR-031-AC-3 | TC-145 through TC-149 | 🚧 planned |
| FR-032 | FR-032-AC-1 through FR-032-AC-3 | TC-150 through TC-154 | 🚧 planned |
| FR-033 | FR-033-AC-1 through FR-033-AC-3 | TC-155 through TC-158 | 🚧 planned |
| FR-034 | FR-034-AC-1 through FR-034-AC-3 | TC-138, TC-159 | 🚧 planned |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-138 | Compare default and opt-in module visibility, bounded golden bytes, dependency and feature trees, and absence of bounded imports of `infinite` | Integration | P0 | FR-027-AC-1, FR-027-AC-2, FR-027-AC-3, FR-028-AC-3, FR-034-AC-3 | 🚧 planned |
| TC-139 | Register one liveness backend, refuse a duplicate, identity mismatch or model request without model capability, and preserve feature-off absence | Integration | P0 | FR-028-AC-1, FR-028-AC-2 | 🚧 planned |
| TC-140 | Retain provider revision, feature, profile, graph and clock attribution without reattributing bounded results | Integration | P0 | FR-029-AC-2 | 🚧 planned |
| TC-141 | Compare complete Boolean valuations with the independent oracle on each lasso position | Property | P0 | FR-030-AC-1 | 🚧 planned |
| TC-142 | Enumerate FR-160 possibilities for missing values using one common refinement per position | Property | P0 | FR-030-AC-1 | 🚧 planned |
| TC-143 | Preserve missing/conflicting reason and closure behavior over equal possibility sets, refuse an empty set, and never flip a conclusive result when information is added | Property | P0 | FR-030-AC-2 | 🚧 planned |
| TC-144 | Preserve repeated proposition and loop-position completion identity and refuse a foreign map or clock | Property | P0 | FR-030-AC-3 | 🚧 planned |
| TC-145 | Check F/G closed and unbounded witness and counterexample boundaries against an independent lasso oracle | Property | P0 | FR-031-AC-1 | 🚧 planned |
| TC-146 | Check U/R induction, duality and nonzero lower-bound obligations on repeated loops | Property | P0 | FR-031-AC-1 | 🚧 planned |
| TC-147 | Check O/H origin and lower-bound semantics without backward loop wrap | Property | P0 | FR-031-AC-2 | 🚧 planned |
| TC-148 | Check S/T/Y and mixed future/past nesting with independent origin-based oracle | Property | P0 | FR-031-AC-2 | 🚧 planned |
| TC-149 | Check Boolean and temporal duals, bounded embedding, W/M lowering and TL `[a,)` to QSL `[a,*]` denotation correspondence | Property | P0 | FR-031-AC-3 | 🚧 planned |
| TC-150 | Replay empty-prefix and nonempty-prefix lassos with exact loop-entry boundaries | Integration | P0 | FR-032-AC-1 | ✅ implemented |
| TC-151 | Compare repeated loop observations and event-position mapping against an independent infinite-word oracle | Property | P0 | FR-032-AC-1 | 🚧 planned |
| TC-152 | Admit every valid lasso under an empty fairness set and require infinite visits for each complete-value premise | Property | P0 | FR-032-AC-2 | 🚧 planned |
| TC-153 | Filter shared partial completions before claim evaluation and show weakening fairness cannot remove an admitted trace | Property | P0 | FR-032-AC-2 | 🚧 planned |
| TC-154 | Refuse foreign or malformed fairness/lasso identities and classify empty fair admission without vacuous proof | Integration | P0 | FR-032-AC-3 | 🚧 planned |
| TC-155 | Separate trace-scoped proved/refuted/inconclusive possibilities and refuse one-lasso model proof | Property | P0 | FR-033-AC-1 | ✅ implemented |
| TC-156 | Validate witness and counterexample details against the admitted completion and full loop | Integration | P0 | FR-033-AC-1 | 🚧 planned |
| TC-157 | Refute only continuation-invariant finite-prefix safety violations and never prove liveness from a prefix | Property | P0 | FR-033-AC-2 | ✅ implemented |
| TC-158 | Map unsupported and both failed execution dispositions to the exact FR-341 axes without a Boolean fallback | Integration | P0 | FR-033-AC-3 | 🚧 planned |
| TC-159 | Mutate each identity and limit axis at and one over bound; verify deterministic results and checked arithmetic | Property | P0 | FR-034-AC-1, FR-034-AC-2, FR-034-AC-3 | 🚧 planned |

## Evidence gate

The local campaign records source revisions, toolchain, feature tree, corpus
manifest digest, generated seed and accepted/discarded populations, oracle
revision, limit settings and actual result bytes. A provider result does not
qualify itself as its own oracle. TL-215 human acceptance is the predecessor
for deciding whether the remaining planned rows have qualifying evidence.
