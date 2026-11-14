---
id: SR-371
title: "Gap analysis: TL-371 mutation close-out, FR-022 and NFR-005 (tl-mltl #176, #177; tl-syntax #135, #136)"
type: SpecReview
analysis: gap-analysis
scope: "FR-022-AC-1..5, NFR-005-AC-1/2, TC-113..118; diffs of tl-mltl #176, #177 and tl-syntax #135, #136"
review_set: subset
relationships:
  - target: ix://agent-ix/tl-mltl/FR-022
    type: references
  - target: ix://agent-ix/tl-mltl/NFR-005
    type: references
---

## Summary

The computed matrix (`quire matrix --scope .`) binds every FR-022 and NFR-005
criterion in scope to a non-ignored test in `tests/mutation_gate.rs`; no
criterion is untagged. The tags do not all prove what the matrix rows claim.
tl-mltl builds its mutation gate from tl-syntax at rev `1146726`
(`tools/gate/Cargo.toml`), which predates tl-syntax #136. Run on a failing
baseline, that gate prints `FAIL baseline` and then
`ok   crate: kill 100.00% ...`, so TC-114 and FR-022-AC-2 ("emit no kill rate")
are not met in this repository although #177 marks them implemented. The
tl-mltl baseline test does not assert the absence of a kill rate, so the suite
stays green. No test covers an unstable baseline, and no test ties the
configured floors to the 95%/85% thresholds of FR-022-AC-4.

## Verdict

FAIL. FND-3711 is high: the matrix marks TC-114 and FR-022 implemented while the
gate this repository runs still emits a kill rate on a failing baseline.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-3711 | high | TC-114 and the FR-022 row are marked implemented, but `tools/gate` pins tl-syntax rev 1146726, before #136; on a failing baseline it still prints the group kill rates. `mutation_gate_refuses_a_failing_baseline` checks only the exit status and `FAIL baseline`, so it passes against the old gate. Fix: move the rev to a tl-syntax commit at or after e0ff8d3, and assert that stdout contains no `kill`. | tools/gate/Cargo.toml:11, tests/mutation_gate.rs:113, spec/verification-effectiveness-test-matrix.md:18, spec/verification-effectiveness-test-matrix.md:50 |
| FND-3712 | medium | FR-022-AC-2 and TC-114 require that an unstable baseline also refuse the campaign. tl-gate reads one Baseline outcome per outcomes directory. No test or mechanism covers an intermittently passing baseline, so the row claims more than the tests prove. | tests/mutation_gate.rs:113, spec/requirements/FR-022-measured-mutation-campaign.md |
| FND-3713 | low | TC-116 and FR-022-AC-4 name 95% and 85% floors. The test uses synthetic 75%/75.1% floors, and nothing checks that `mutation-thresholds.toml` keeps evaluator-core at 95 or above and crate at 85 or above. Lowering the floors to 0 would leave every test green. Today's floors (100.0 and 99.5) meet the criterion. | tests/mutation_gate.rs:162, mutation-thresholds.toml |

## Coverage

- Criteria in scope: FR-022-AC-1..5 and NFR-005-AC-1/2. All are tagged by a non-ignored test, and `cargo test --test mutation_gate` passes 6 of 6.
- Reverse gaps: none in the diffs. The tl-syntax #135 scanner tests and the #136 gate change are owned by TL-371 and SR-231. The tl-gate tests carry no trace tags, which matches the file's existing convention.
- Stubs and inflation: none found. FND-3711 is a test that is too weak, not a stub.
- Semantic review: limited to the in-scope rows, as the caller asked (each matrix status checked against what its tests prove).
- NFR-005 stays 🚧 planned in the matrix, which is consistent with the evidence.
- Plan completion: not assessed

## Resolution

The high finding (FND-3711) is fixed: `tools/gate/Cargo.toml` and `Cargo.lock` build tl-gate from tl-syntax `d44d00f`, which
includes #136 (a failing baseline prints no kill rate) and #138 (an unstable baseline is refused), and
`mutation_gate_refuses_a_failing_baseline` asserts that no kill rate is printed.

FND-3712 is closed. `tl-gate mutants` runs the unmutated suite `baseline_runs` times (default 2; `mutation-thresholds.toml`
sets 2) and merges one `Baseline` outcome per run; `tl-gate gate` refuses a campaign whose baseline outcomes are not all
successes with `FAIL baseline: unstable (passed N of M runs)` and no kill rate. `mutation_gate_refuses_an_unstable_baseline`
covers the gate's judgement over mixed baseline outcomes; the runner's N-run behavior is tested in tl-syntax.

FND-3713 is closed. `the_real_thresholds_keep_the_core_and_crate_floors` fails if `evaluator-core` is below 95 or `crate` is
below 85 in the real `mutation-thresholds.toml`, and `the_floor_check_fails_a_lowered_or_missing_floor` shows the check
fails on lowered, missing and unparsable floors. The floors are 100.0 and 99.5.

The matrix marks FR-022, TC-114, TC-120, TC-121, TC-122 and TC-127 implemented, and the NFR-005 summary row implemented. `kani_gate` runs each
harness under a wall-clock budget (`TL_MLTL_KANI_BUDGET_SECS`, default 1800 s), kills the run at the budget and fails the gate with
`<harness> TIMED OUT`; tests drive the gate binary with a fake `cargo` for the timeout and passing paths, and check the exact
`--harness NAME --exact` command. No test runs the real `cargo kani`. This pass did not run a fresh full mutation campaign; the floors
rest on the measured runs recorded in SR-231.
