---
id: FR-012
title: "Run bounded and retained fuzz campaigns"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-003
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-009
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-011
    type: depends_on
---

# FR-012: Run bounded and retained fuzz campaigns

## Description

When a reviewed Fuzz-kind obligation enters the campaign, the owning repository shall
execute a finite predeclared fuzz budget and retain the target, starting population,
discoveries, stopping reason, and limitations without interpreting no crash or a
coverage plateau as semantic proof.

## Inputs

- Applicable FR-011 row whose authored method resolves through the catalog to
  `evidence_kind: Fuzz`, or an accepted specification amendment selecting a
  byte-oriented or structured-input trust boundary.
- Exact source, target binary, compiler, cargo-fuzz/libFuzzer, sanitizer,
  target-triple, feature/configuration, dictionary, and environment identities.
- Starting corpus manifest and three distinct declared RNG seeds.
- Baseline budget profile `tl-mltl.fuzz-baseline/v1`.

## Outputs

- One owner-native fuzz-campaign record per target and exact revision;
  tl-mltl uses `tl-mltl.fuzz-campaign/v1`.
- Per-repetition snapshots, raw outcome, final corpus/crash digests, minimized
  reproducer when available, stopping reason, and plateau classification.
- A typed refusal or non-conclusive result for incomplete identity, corrupted
  artifacts, invalid budgets, instrumentation loss, or irreproducible failure.

## Behavior

The baseline profile fixes a first bounded observation rather than a strength
threshold: it has three RNG-seeded repetitions. Each repetition predeclares both a
900-second wall-clock cap and 1,000,000 executed-input cap and stops at the first
cap reached, target crash, explicit operator cancellation, resource refusal, or
tool failure. The record preserves requested and observed budgets; startup,
build, minimization, and replay time are separate from fuzz-execution time.

Coverage snapshots record monotonically increasing executed-input count,
elapsed monotonic time while the fuzz target executes, corpus entries/bytes,
and the digest plus sorted identity set (or stable engine bitmap) of exact
instrumented features at least every 10 target-execution seconds or 10,000
executions, whichever occurs first. A count without the identities is
insufficient because one disappearing feature can mask one newly found feature.
A repetition is `plateau_observed` only when its final 300 target-execution
seconds and 100,000 executed inputs both add zero instrumented features and the run reaches
a budget cap without crash, cancellation, resource refusal, counter reset, or
instrumentation loss. Otherwise it is `growth_observed` or `non_conclusive`.
Feature identities are comparable only within one exact target/toolchain/
instrumentation identity. Plateau is a prioritization signal for seed review or
bounded analysis, never proof that undiscovered inputs are safe.

The 3 × 900-second × 1,000,000-input profile and 300-second/100,000-input
plateau window are versioned M5 baseline policy values chosen to make the first
observation finite and repeatable. They are not asserted to be sufficient for
defect discovery; changing one requires a successor profile and prevents direct
comparison with the original.

Outcomes remain distinct: `budget_complete`, `crash`, `timeout_input`,
`resource_refused`, `cancelled`, `tool_error`, `instrumentation_lost`, and
`not_run`. A crash record retains the original and minimized bytes, signal/
sanitizer class, replay command, and independent reproduction outcome. Timeout,
out-of-memory, parser refusal, harness panic, and product panic are not silently
collapsed. An irreproducible crash stays visible and non-conclusive.

The campaign reports feature identities and growth curves, not a coverage
percentage: the reachable instrumented-feature denominator is unknown.

Starting and final corpus paths obey FR-009 path/resource/digest rules. Fuzz
discoveries remain generated evidence. A regression fixture is promoted only
through FR-009; a raw crash file, seed count, elapsed duration, or final corpus
size is not conformance evidence.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-012-AC-1 | Every baseline campaign executes or explicitly records all three predeclared RNG seeds with both finite caps, stops at the first named condition, and preserves requested versus observed target-execution, build, minimization, and replay time. | Test (TC-055, TC-056) |
| FR-012-AC-2 | Plateau is reported only after exact feature identities show that both the final 300 target-execution seconds and 100,000 inputs add no feature under one unchanged instrumentation identity; every count-only, early-stop, reset, loss, or insufficient window is non-conclusive. | Test (TC-056, TC-057) |
| FR-012-AC-3 | Every crash, timeout-input, resource refusal, cancellation, tool error, instrumentation loss, and not-run state round-trips separately, and a crash cannot become conclusive until its exact bytes reproduce independently. | Test (TC-057, TC-058) |
| FR-012-AC-4 | Every target, toolchain, seed, dictionary, starting/final corpus, configuration, environment, snapshot, stopping reason, artifact digest, and limitation is retained; missing or stale identity refuses before a result is credited. | Test (TC-055, TC-058, TC-067) |
| FR-012-AC-5 | No-crash, duration, execution count, corpus growth, and plateau records remain bounded observations and never become correctness, release, qualification, or certification verdicts. | Test (TC-068) |

## Dependencies

Depends on an applicable FR-011 grounding and FR-009 generated/canonical
lifecycle. The owner of the fuzzed entry point owns its Rust harness and domain
producer; Quoin owns retention. A target requiring Java, Node, Electron, R2U2,
C2PO, or another foreign runtime is unavailable, not silently substituted.
