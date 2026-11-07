# Horizon analysis

The horizon of a future formula is the number of instants after the verdict time that the formula can read. It determines how long a monitor must wait before a verdict is final and how many instants of trace it must buffer.

## Rule

`analyze_horizon` computes a value for each node, operands first, and reports the value of the root.

| Node | Value |
| --- | --- |
| `False`, `True`, `Proposition` | 0 |
| `Not x` | the value of `x` |
| `And`, `Or`, `Implies`, `Equivalent` | the larger of the two operand values |
| `Future [a,b] x`, `Globally [a,b] x` | `b` plus the value of `x` |
| `Until [a,b] x y`, `Release [a,b] x y` | `b` plus the larger of the values of `x` and `y` |

Only the upper bound of an interval counts. Every addition is checked. A sum that exceeds `u64` returns `HorizonError::ArithmeticOverflow` and names the node. A past operator returns `HorizonError::UnsupportedPastNode`.

## Report

`analyze_horizon(formula, formula_id)` returns a `HorizonReport` with the schema `tl-mltl.horizon/v1`.

| Field | Meaning |
| --- | --- |
| `lookahead` | The value of the root. |
| `propagation_delay` | The same value. A monitor decides the formula at time `t` once it has seen instant `t + lookahead`. |
| `required_buffer` | `lookahead + 1`, the number of instants a monitor holds. |
| `unit` | `discrete-instants`. |
| `corpus_revision` | The shared corpus identity the semantics follow, `tl-syntax-corpus/v1`. |

The function accepts `mltl.closed-trace/v1` and `mltl.online-prefix/v1`. Any other profile returns `HorizonError::UnsupportedProfile`.

For `F[1,3] p0` the lookahead is 3 and the buffer is 4. For `F[0,1] G[0,2] p0` it is 3, because the bounds of nested operators add. The evaluators report the same lookahead as `horizon` in every `EvaluationReport`.

`analyze_horizon_with_context` also checks that every proposition is bound to a Boolean signal in a supplied `SignalCatalogDocument` and returns a `ContextualHorizonReport` (schema `tl-mltl.horizon/v2`) that carries the catalog identity, the requirement context and the request and result digests.

## Required history

A past formula looks backward. `analyze_required_history` returns a `HistoryRequirementReport` with `required_positions`, the largest reverse offset the formula reads, measured in positions. It follows the same rule as the table above, except that `StrongPrevious` adds 1, and `Since` and `Triggered` add their upper bound to the larger operand value. `evaluate_past` stores the same number as `required_history`. The function accepts only `mltl.origin-complete-history/v1`.

## Checked arithmetic proof

The addition primitive used by the traversal has a Kani harness, `future::horizon::kani_proofs::horizon_bound_addition_matches_checked_add`. It proves that the sum equals `checked_add` for all `u32` bounds and `u64` operand values, including the overflow refusal. It does not prove the traversal or the evaluators. Kani is optional and `make kani-check` runs the harness.
