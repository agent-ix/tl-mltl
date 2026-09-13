---
id: MP-006
title: "TL bounded Kani claim results"
type: MeasurementPlan
status: proposed
owner: tl-verification-campaign-owner
metric: tl.bounded-proof-result
definition_version: tl-mltl.bounded-proof-result/v1
stage: baseline
statistical_design:
  population: every exact proof-candidate-ledger row classified as applicable, excluded, or blocked, including rationale, harness, proposition, assumptions, finite domains, bounds, checks, and explicit non-claims when applicable
  sampling: complete classification of the candidate ledger and complete execution of every applicable harness under exact verifier, solver, toolchain, configuration, timeout, and memory identities
  repetitions: 1
  estimator: an identity-keyed categorical result vector with no aggregate proof ratio; each candidate remains proved-within-bounds, counterexample, unwind-incomplete, solver-unknown, vacuous, timed-out, unsupported, tool-error, not-run, excluded, or blocked
  error_model: stale harness or candidate, hidden assumptions, uncovered partitions, disabled unwind checks, insufficient bounds, unsupported constructs, solver/tool drift, timeout, semantic forks, and claim widening
  uncertainty: a conclusive result applies only to the explicitly finite domain and assumptions; no inference is made to larger formulas, traces, histories, arithmetic domains, callers, or unbounded MLTL
  decision_rule: permit a bounded claim for one applicable row only when every assertion, partition cover, supported-path, and applicable unwind check succeeds at the exact identities and loop-free unwind is not-applicable only after a retained exact census; otherwise preserve the specific result, exclusion, or blocked state
relationships:
  - target: ix://agent-ix/tl-mltl/FR-014
    type: measures
  - target: ix://agent-ix/tl-mltl/NFR-005
    type: measures
---

# TL bounded Kani claim results

## Decision Use

The measure decides whether each applicable finite proposition was established at
its exact assumptions and bounds. It cannot qualify unselected behavior,
unbounded semantics, a source language, a release, or a monitor.

## Population

The population is every classified row in the reviewed tl-mltl proof-candidate
ledger at one source revision. Selection rationale, reviewed matrix priority,
harness/proposition/non-claim text,
domains, assumptions, stubs, limits, toolchain, solver, and environment are
frozen before execution. Historical closed-unmerged runs are outside it.

Sibling TL repositories require their own accepted local MeasurementPlan and
owner-native proof ledger. They may reuse this plan as a reviewed template but
are outside its population and authority.

## Collection Procedure

Run every exact harness with all required cover, assertion, supported-path, and
unwind checks. Retain command, output, statuses, digests, resource use, and any
counterexample through the Rust producer and Quoin intake. Reproduce a
counterexample before routing it as a regression.

## Interpretation

Report every candidate and state without a proof-coverage fraction.
`proved_within_bounds` means exactly the proposition written for exactly the admitted finite domain.
Timeout, unknown, vacuity, partial execution, unsupported paths, or unchecked
unwinding remain visible and confer no proof credit.
