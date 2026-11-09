---
id: FR-023
title: "State and guard bounded Kani claims"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-003
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-020
    type: depends_on
---

# FR-023: State and guard bounded Kani claims

## Description

When a Kani harness is added, tl-mltl shall state in the harness the bounds it covers
and what it does not claim, shall fail a harness that is vacuous or whose
unwinding is incomplete, and shall add no Kani dependency to production code.

## Inputs

- The harness list and the proposition each harness proves.
- The Kani version and the unwind setting for each harness.
- The property gaps, fuzz counterexamples, and mutation survivors that motivated
  a harness.

## Outputs

- For each harness, an outcome of `proved_within_bounds`, `counterexample`,
  `unwind_incomplete`, `vacuous`, `timed_out`, `tool_error`, or `not_run`, with the
  Kani version.
- A reproducible counterexample when one exists.

## Behavior

The harness list is:

| Harness | Proves, within its stated bounds |
|---|---|
| bound addition | the horizon bound addition equals checked addition over the full `u32` bound and `u64` child range (the existing `horizon_bound_addition_matches_checked_add`) |
| interval arithmetic | no overflow or wrap in bound, shift, and horizon computations over the full integer range |
| evaluator no-panic | evaluation of every formula up to depth 2 over every trace up to length 4 terminates without panicking |
| wire decode no-panic | any input up to 64 bytes decodes or returns a typed error |
| closing prefix | once the prefix reaches the horizon, the verdict does not change |

Each harness carries a doc comment that names its bounds and its non-claims. For
example, the evaluator harness states that it covers depth at most 2 and says
nothing about semantic correctness. A proof of a private arithmetic primitive does
not prove arbitrary horizon traversal, a harness over atomic formulas does not
prove nested formulas, and a finite bound does not prove unbounded MLTL.

A harness counts as proved only when every assertion holds, unwinding is complete,
and the cover checks show the interesting paths are reachable. An assumption
that admits no case leaves the proof vacuous, and a vacuous proof is a failure.
A harness over code with no loop or recursion may omit an unwind bound only when
the reachable code is shown to have none; an unused unwind option does not count
as a check. A timeout, unwinding failure, missing cover result, or unsupported
construct keeps its own status and is never proof.

The local gate runs every listed harness by its exact name, so a renamed or
removed harness fails the gate rather than silently dropping out. CBMC's internal
property counts are diagnostics and not a proof count.

Only the proof module and the visibility needed to call an existing subject may
be gated under `cfg(kani)`. The subject function body compiled for verification
is identical to the production body, and no Kani, Loom, Verus, or concolic-execution
library appears in the production dependency tree. Ordinary stable and MSRV builds
reject a stray verifier-only configuration.

The campaign report records the Kani version, each harness's bounds, and its
outcome. A survivor routed here from FR-022 names the harness that closes it.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-023-AC-1 | Every harness in the list carries a doc comment stating its bounds and non-claims, and the local gate runs each by its exact name, so a harness missing from the run or from the list fails the gate. | Test (TC-120) |
| FR-023-AC-2 | A harness is proved only when every assertion holds, unwinding is complete, and its cover checks are reachable; a vacuous proof, unwinding failure, timeout, or unsupported construct is a failure with its own status. | Test (TC-120) |
| FR-023-AC-3 | The campaign report records the Kani version and each harness's bounds and outcome, and a counterexample is reproducible from the recorded harness. | Test (TC-120, TC-121) |
| FR-023-AC-4 | Verifier-only code changes only proof-module and visibility reachability over an identical subject body, ordinary and MSRV builds detect any divergence, and no Kani, Loom, Verus, concolic, Java, Node, or Electron dependency enters production. | Test (TC-123) |
| FR-023-AC-5 | Withdrawn: rejection of historical closed-unmerged proof artifacts as current evidence. | Withdrawn |

## Dependencies

Depends on the FR-020 property laws and on a stated reason for each harness.
Fuzz- and mutation-driven harnesses depend on the FR-021 and FR-022 results.
Kani is the selected bounded-proof tool, not a source-language, runtime, or
universal verification dependency.
