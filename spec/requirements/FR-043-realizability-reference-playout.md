---
id: FR-043
title: Decide requirement realizability by exhaustive play-out of the closed-trace semantics
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-004
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-001
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-002
    type: depends_on
  - target: ix://agent-ix/tl-mltl/ADR-004
    type: references
  - target: ix://agent-ix/tl-syntax/FR-034
    type: depends_on
  - target: ix://agent-ix/tl-solve/FR-104
    type: references
---

# FR-043: Decide requirement realizability by exhaustive play-out of the closed-trace semantics

## Description

When a set of validated closed-trace future formulas and a declared
input/output partition of their propositions are supplied, tl-mltl shall
decide requirement realizability by enumerating every play of the fixed-horizon
game of [ADR-004](../decisions/ADR-004-requirement-realizability-semantics.md)
and judging each complete play with `evaluate_closed_at`. The result is a
verdict with the counts that show how it was reached. The module is an
independent reference oracle for tl-solve. It generates no strategy and no
code, and it is not a production decision procedure.

## Inputs

- One or more `tl_syntax::Formula` values in profile
  `SemanticProfile::ClosedTraceV1`, with caller-supplied formula identities.
- A `tl_syntax::SignalPartition` (tl-syntax FR-034) listing input and output
  `PropositionId` values. Every proposition occurring in any formula must be in
  exactly one list.
- `PlayOutLimits`: maximum trace length in steps, maximum input-plus-output
  bits per step, and maximum number of complete plays evaluated.

## Outputs

- `PlayOutReport`: the verdict `Realizable` or `Unrealizable`; the trace length
  `L`; the horizon report identity that produced it; the number of complete
  plays evaluated; and `every_input_sequence_has_response`, the truth of
  "for every input sequence some output sequence makes every formula true".
- `StrategyJudgement`, returned by `judge_strategy` (see Behavior): `Wins` or `Loses` with the first losing input sequence, and the number of input sequences played.
- `PlayOutError`, a closed set of typed refusals: unsupported profile, horizon
  failure, empty requirement set, proposition not partitioned, trace-length
  limit, alphabet-width limit, play-count limit, and evaluation failure.

## Behavior

- The trace length is `L = H + 1`, where `H` is the maximum `lookahead` that
  `analyze_horizon` reports over the formulas, checked. The caller supplies no
  length. This is the fixed horizon of ADR-004 decision 2.
- A play is a trace of `L` instants. In step `t` the input valuation is chosen
  first, then the output valuation. The trace is the sequence of chosen
  valuations, each written as the strictly increasing list of true
  `PropositionId` values that `evaluate_closed_at` accepts.
- The verdict is the value of the alternating game
  `forall i_0 exists o_0 forall i_1 exists o_1 ... exists o_H` of the
  predicate "every formula evaluates to `TruthValue::True` at position zero".
  `Realizable` when that value is true.
- `every_input_sequence_has_response` is the value of
  `forall i_0 ... forall i_H exists o_0 ... exists o_H` of the same predicate.
  `Realizable` implies it. The reverse does not hold, and the report shows both
  so a necessary-check pass is never read as realizable.
- A partition with no input propositions reduces the verdict to satisfiability
  of the conjunction on `L` instants; a partition with no output propositions
  reduces it to validity.
- Evaluation uses the same `EvaluationLimits` as the closed-trace evaluator. A
  limit or evaluation failure on any play returns that failure and no verdict.
- The number of plays evaluated is `2^((|I|+|O|)*L)` when no limit intervenes.
  A play count, alphabet width or trace length above its limit refuses before
  any play is evaluated; each limit admits the value exactly at the limit and
  refuses one more, with its own variant.
- Results depend only on the formulas, the partition and the limits.
- Strategy judgement interface. `judge_strategy(formulas, partition, limits,
  respond)` takes the same formulas, partition and limits as the verdict, and
  a callback `respond(&[Vec<PropositionId>]) -> Vec<PropositionId>` that maps
  the input history through step `t` (one strictly increasing list of true
  input `PropositionId` values per step) to the output valuation for step
  `t`. The judge enumerates every input sequence of `L` steps, calls `respond`
  at each step in order, builds the complete play, and judges it with
  `evaluate_closed_at`. It returns `Wins` exactly when every play satisfies
  every formula, else `Loses` with the lexicographically first losing input
  sequence. A `respond` result that names a proposition not in the output list
  is refused with its own `PlayOutError` variant. The callback type mentions no
  tl-solve type, so tl-mltl keeps its independence from tl-solve (tl-mltl
  FR-113); the adapter that turns a tl-solve strategy certificate into a
  callback lives in tl-solve test code and is owned by tl-solve FR-104-AC-6
  (ticket TL-416), not by this crate. The judge shares no code with tl-solve and
  is not a production decision procedure.
- Population ownership. The declared population is specified by rule in tl-solve
  TM-100 (section Declared small population) and restated in TM-007. tl-mltl
  tests generate it from that rule with a generator written in tl-mltl test
  code; the counts 126960, 44566, 82394 and 768 are compared as numbers
  recorded in the rule, never by importing tl-solve code or data. tl-solve
  tests generate it with its own generator. A mismatch between the two
  generators' counts is a finding against the rule.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-043-AC-1 | For every formula set and partition in the declared population of tl-solve TM-100, the trace length equals one plus the maximum `lookahead` from `analyze_horizon`, a past-profile, online-prefix or other non-closed-trace formula is refused with the unsupported-profile variant, and the API accepts no caller length. | property-based-testing (TC-188) |
| FR-043-AC-2 | The verdict equals an independently computed answer to the quantified game on every case of the declared population, and a game in which the output at step `t` equals the input at step `t` is `Realizable` while one in which it equals the input at step `t+1` is `Unrealizable`. | property-based-testing (TC-188, TC-189) |
| FR-043-AC-3 | `every_input_sequence_has_response` equals the independently computed `forall exists` value on every case of the declared population, `Realizable` never occurs with it false, and at least one case in the population has it true with the verdict `Unrealizable`. | property-based-testing (TC-189) |
| FR-043-AC-4 | The all-output partition reports the same verdict as satisfiability of the conjunction by direct enumeration, and the all-input partition reports validity by direct enumeration. | property-based-testing (TC-190) |
| FR-043-AC-5 | Trace-length, alphabet-width and play-count limits each admit a value exactly at the limit and refuse a value one past it with their own typed variant before evaluating any play, and a proposition occurring in a formula but absent from the partition is refused with its own variant. | unit-testing (TC-191) |
| FR-043-AC-6 | Repeated runs on equal inputs return equal reports, and an evaluation failure on any play returns the failure and no verdict. | unit-testing (TC-192) |
| FR-043-AC-7 | `judge_strategy` returns `Wins` for a callback that copies the input at step `t` to the output at step `t` on the game where that wins, returns `Loses` with the lexicographically first losing input sequence for a callback that violates one requirement on exactly one input sequence, equals the verdict on every case of the declared population when the callback is the lowest winning response derived inside the test by direct enumeration, and refuses a callback output outside the output list with its own variant. | unit-testing (TC-193) |

## Dependencies

[FR-001](./FR-001-closed-evaluation.md) owns the evaluation function that
judges a play, and [FR-002](./FR-002-horizon-analysis.md) owns the horizon that
fixes its length. [NFR-001](./NFR-001-determinism-resources.md) governs
limit-edge and error-variant coverage. tl-syntax FR-034 supplies the partition.
tl-solve FR-104 (the game search) is compared against this module and shares no
code with it.
