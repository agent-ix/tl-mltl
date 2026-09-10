---
id: FR-014
title: "Constrain and retain bounded Kani claims"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-003
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-011
    type: depends_on
---

# FR-014: Constrain and retain bounded Kani claims

## Description

When a reviewed TL obligation is selected for Kani, its owning repository shall
bind the proof result to one exact harness, candidate, domain, assumptions,
bounds, unwinding checks, verifier/solver configuration, and claim statement
and refuse any broader interpretation.

## Inputs

- Applicable FR-011 obligation and selection rationale:
  `property_gap`, `fuzz_plateau`, `fuzz_counterexample`,
  `mutation_survivor`, or `reviewed_bounded_arithmetic`.
- Exact candidate, harness path/symbol/digest, Kani/Rust/solver versions,
  command arguments, target/configuration, assumptions, stubs, symbolic types,
  loop/recursion/unwind/object limits, timeout, and memory cap.
- Exact proposition to prove and its explicit non-claims.

## Outputs

- An owner-native proof-candidate ledger and bounded-proof record; tl-mltl uses
  `tl-mltl.proof-candidate-ledger/v1` and `tl-mltl.bounded-proof/v1`.
- A result status `proved_within_bounds`, `counterexample`,
  `unwind_incomplete`, `solver_unknown`, `vacuous`, `timed_out`, `unsupported`,
  `tool_error`, or `not_run`.
- Assertion, cover/vacuity, unwind, and unsupported-feature outcomes plus a
  reproducible counterexample when one exists.
- A typed refusal for missing identity, implicit bounds, unchecked unwind,
  contradictory assumptions, semantic forks, or claim widening.

## Behavior

Every candidate trigger from an FR-011 property gap, an FR-012
plateau/counterexample, an FR-013 missed/timeout disposition, or an exact
reviewed bounded-arithmetic proposition appears exactly once as `applicable`,
`excluded`, or `blocked`. Its domain-separated identity binds source revision,
requirement/statement hash, subject symbols, rationale, and claimed property but
excludes classification and execution result. Reclassification or removal
requires a reviewed successor ledger and tombstone. Each ledger carries a
domain-separated digest over its complete ordered identities,
classifications, reasons, dependencies, propositions, non-claims, and declared
bounds plus the predecessor-ledger digest; changing any covered fact creates a
new successor digest.

`proved_within_bounds` requires successful verification of every claimed
assertion, enabled unwind checks, no unwinding failure, no unsupported construct
on the proof path, and evidence that the assumptions admit at least one case in
every declared partition. An assumption is part of the claim and names the
excluded population. A timeout, solver unknown, missing cover result, disabled
unwind check, partial harness, or unsupported operation retains its distinct
named status, never proof.

A harness classified as loop- and recursion-free records `unwind_check:
not_applicable` only after retaining an exact subject and reachable-call-graph
loop/recursion census. Supplying an inert numeric unwind option is not an unwind
check and cannot satisfy this condition.

The claim text repeats the exact finite types/cardinalities, formula/trace or
arithmetic structure, loop/recursion and unwind bounds, assumptions, operation,
and output property. A proof of a private checked-addition primitive does not
prove arbitrary horizon traversal; an atomic evaluator harness does not prove
nested formulas; a finite formula/trace bound does not prove unbounded MLTL.

Only the proof module and visibility needed to call an existing subject may be
gated under `cfg(kani)`; the subject function body compiled for verification
must be identical to the ordinary production body.

Verifier-only code shall not
add a proof-only subject primitive, alternate production semantic branch,
changed ordinary-build result, or Kani runtime dependency.

Ordinary stable/MSRV builds shall reject accidental verifier-only configuration
drift. Loom remains inapplicable to the non-concurrent core. Verus and concolic
execution require separate reviewed architecture decisions.

CBMC internal assertion/property counts are diagnostics, not proof counts or a
coverage denominator. The unit of evidence is one exact candidate-ledger row,
harness, claim, and declared finite model.

The closed-unmerged tl-mltl PR #35 and its historical runs are feasibility
inputs only. No record may cite them as evidence for current main; a successor
harness must rerun at its exact landed candidate and retain a new result.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-014-AC-1 | Every candidate trigger is classified exactly once under a lifecycle-independent identity; the ordered ledger digest covers classifications, reasons, dependencies, propositions, non-claims, bounds, and predecessor digest; and every proof record round-trips exact candidate, harness, proposition, non-claims, toolchain/solver/configuration, symbolic domains, assumptions, stubs, resource limits, loop/recursion/unwind bounds or a retained exact not-applicable census, checks, outcomes, and artifact digests. | Test (TC-065, TC-067) |
| FR-014-AC-2 | A proved-within-bounds result is possible only when every assertion, applicable unwind check, supported-path check, and partition cover succeeds; loop-free unwind is not-applicable only after an exact retained census; timeout, unknown, vacuity, partial execution, disabled checks, and inert unwind flags remain non-conclusive, and CBMC internal property counts never become proof counts. | Test (TC-065, TC-066) |
| FR-014-AC-3 | Mutating any bound, assumption, harness digest, candidate, solver/configuration, claim scope, or outcome invalidates the record or makes the owning gate red, while a counterexample remains reproducible. | Test (TC-066, TC-067) |
| FR-014-AC-4 | Verifier-only code changes only proof-module/visibility reachability over an identical subject body, ordinary/MSRV controls detect any divergence, and no Kani, Loom, Verus, concolic, Java, Node, or Electron runtime enters production. | Test (TC-069) |
| FR-014-AC-5 | Historical #35 evidence is classified closed-unmerged and cannot satisfy a current obligation; only a new exact-candidate run may support a bounded claim. | Test (TC-070) |

## Dependencies

Depends on FR-011 grounding and a reviewed selection rationale. Mutation- or
fuzz-driven selections conditionally depend on the exact FR-012/FR-013 result;
reviewed bounded-arithmetic selections do not. Kani is
the selected initial bounded-proof tool, not a source-language, runtime, or
universal verification dependency.
