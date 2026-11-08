---
id: SR-611
title: "Evidence record - tl-mltl fuzz targets and runs (TL-229, TL-230, TL-29)"
type: SpecReview
analysis: evidence
scope: "fuzz/, evaluator_differential, wire_cli_decode, history_intake and c2po_map targets, FR-001-AC-1, FR-018-AC-1, FR-018-AC-2, TC-001, TC-090"
review_set: subset
---

## Summary

Recorded libFuzzer runs of the targets listed below, three RNG seeds of 60 seconds
each per target (24,509,635 executions in total). No run produced a crash, timeout, out-of-memory or sanitizer finding.
These are bounded observations of finite runs: a run without a crash and a coverage
curve that had not flattened are not evidence of correctness (FR-021-AC-5).

## Verdict

**PASS** for the recorded evidence: every run completed, each new target is shown
able to find a seeded fault, and the fuzz gate (`make fuzz-check`) builds every target.
Targets are discharged only by these recorded runs, not by existing.

## Run record

Source revision 0c68fc5ef6d2 (campaign tool state clean for `src/`, `Cargo.toml`, `Cargo.lock`);
tools: rustc 1.100.0-nightly (1303417c4 2026-09-21), cargo-fuzz 0.13.1, host aarch64-apple-darwin; sanitizer: address (cargo-fuzz default); libFuzzer flags
`-max_total_time=60 -max_len=1024 -rss_limit_mb=8192`; seeds 101, 102, 103 (distinct, run in that order on one
growing corpus per target, or on a throwaway copy of the checked-in corpus for
targets whose checked-in seeds are tested one-by-one). Time is libFuzzer's
reported run time and excludes build time. Machine-readable record:
`fuzz/evidence/2026-11-08/campaign.json`.

| Target | Seed | Seconds | Executions | Coverage edges | Features | Corpus entries | Findings |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| evaluator_differential | 101 | 61 | 188,379 | 2,239 | 12,786 | 1,562 | none |
| evaluator_differential | 102 | 61 | 132,678 | 2,244 | 12,959 | 1,418 | none |
| evaluator_differential | 103 | 61 | 115,276 | 2,244 | 12,977 | 1,394 | none |
| wire_cli_decode | 101 | 61 | 1,575,886 | 5,045 | 15,283 | 2,667 | none |
| wire_cli_decode | 102 | 61 | 2,056,850 | 5,333 | 16,849 | 2,862 | none |
| wire_cli_decode | 103 | 61 | 1,653,400 | 5,489 | 17,683 | 2,990 | none |
| history_intake | 101 | 61 | 2,186,807 | 3,321 | 8,712 | 1,695 | none |
| history_intake | 102 | 61 | 3,283,569 | 3,406 | 9,306 | 1,702 | none |
| history_intake | 103 | 61 | 3,218,909 | 3,516 | 9,663 | 1,841 | none |
| c2po_map | 101 | 61 | 2,129,758 | 2,815 | 7,408 | 1,873 | none |
| c2po_map | 102 | 61 | 4,411,065 | 2,931 | 8,030 | 1,819 | none |
| c2po_map | 103 | 61 | 3,557,058 | 3,012 | 8,282 | 1,953 | none |

## Seeded-fault check

A target is credited only if it fails on a deliberately introduced fault. Each
fault was applied temporarily to the library source (and reverted), and the
target was run from an empty or seed-only corpus.

| Target | Fault | Result |
| --- | --- | --- |
| evaluator_differential | temporal interval upper endpoint shortened by one in the closed evaluator (`temporal_endpoints`, src/future/evaluate.rs) | found within the 90 s cap from an empty corpus on the final target (seed 230; about 37,000 executions): production verdict differs from the oracle |
| wire_cli_decode | `ValidatedTrace::from_json_bytes` keeps empty canonical bytes for traces longer than two instants (src/wire/trace.rs) | found within the 90 s cap on the checked-in seeds (seed 229; 29 s including build): canonical bytes differ from the input |
| history_intake | off-by-one slice in the strictly-ordered proposition check (`true_propositions[i..=i+1]`, src/past/mod.rs) | found within the 90 s cap on the checked-in seed (seed 229; 28 s including build): slice index panic |
| history_intake | unchecked `previous_position + 1` in the gap check (src/past/mod.rs) | NOT found in 91 s and 3,031,066 executions; earlier validation rejects every input that could reach that line, so the fault is unreachable through public input and is not credited |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-6401 | low | No crash, timeout, out-of-memory or sanitizer finding and no production-versus-oracle disagreement in any recorded run of `evaluator_differential`, `wire_cli_decode` or `history_intake`. Coverage was still growing at the end of every run, so no plateau is claimed. | campaign.json |
| FND-6402 | low | `c2po_map` (existing target) ran for the first time on a scratch copy of its eight seeds: no crash. | campaign.json |
| FND-6403 | medium | The repository had no fuzz gate. `make fuzz-check` (builds every target, audits the fuzz crate with cargo-deny, now in `make ci`) and `make fuzz-smoke` (corpus replay) were added; NCSA, the license of libFuzzer, is allow-listed for the fuzz crate only. | Makefile, deny.toml |
| FND-6404 | medium | A first draft of `evaluator_differential` stalled for more than 30 seconds on a 128-byte input: it converted every generated node, including nodes the root does not reference, into the oracle's owned trees. The target now builds the oracle tree from the root only and skips formulas whose naive evaluation cost (nested interval widths multiply) exceeds 100,000 steps, because tl-oracle has no step bound of its own. | fuzz/fuzz_targets/evaluator_differential.rs |
| FND-6405 | low | The first seeded fault tried for `history_intake` (unchecked `previous_position + 1` in the gap check) was not found in 91 seconds and 3.0 million executions: earlier validation rejects every input that could reach it, so it is not reachable through the public intake. It is not credited; the reachable fault below was used instead. | src/past/mod.rs |
| FND-6406 | low | On macOS libFuzzer counts the address sanitizer's freed memory toward RSS, so the default 2048 MB limit aborted runs whose live heap was about 34 MB, and the abort report then hung for minutes; recorded runs pass `-rss_limit_mb=8192`. Not a target defect. | fuzz/README |

## Limitations

The runs are three 60-second repetitions, not the 3 x 900 s baseline profile of
FR-021, and carry no feature-identity snapshots, so FR-021-AC-1 to AC-4 and
TC-109 to TC-112 (the campaign-record protocol) are not claimed and remain
planned. `evaluator_differential` compares up to 32 nodes, intervals up to width
5 starting at up to 4, three propositions and traces of up to 12 positions, and
skips oracle refusals. The traced obligations are the closed evaluation
criterion FR-001-AC-1 and the owner wire criteria FR-018-AC-1 and FR-018-AC-2.
The checked-in corpora are the hand-picked seeds of the two decode targets and a
coverage-minimised set of generated inputs for `evaluator_differential`.

