# Semantics

tl-mltl has three evaluators. The future evaluator covers `mltl.closed-trace/v1` and `mltl.online-prefix/v1`. The past evaluator covers `mltl.origin-complete-history/v1`. The infinite evaluator covers `mltl.infinite-trace/v1` and is available with the `infinite-trace` feature.

## Future operators

Time is discrete. An interval `[a,b]` is inclusive and its bounds are instants relative to the instant being evaluated. The evaluators implement these operators:

| Node | Value at instant `t` |
| --- | --- |
| `Proposition p` | `p` is in the trace instant at `t`. |
| `Not`, `And`, `Or`, `Implies`, `Equivalent` | Boolean connectives, extended to three values below. |
| `Future [a,b] x` | `x` holds at some `t+k` with `a <= k <= b`. |
| `Globally [a,b] x` | `x` holds at every `t+k` with `a <= k <= b`. |
| `Until [a,b] x y` | `y` holds at some `t+k` with `a <= k <= b`, and `x` holds at every `t+j` with `a <= j < k`. |
| `Release [a,b] x y` | The negation of `Until [a,b]` over the negations of `x` and `y`. |

The lower bound `a` is part of every window. For `Until`, `x` is required only from `a` up to the witness, so an instant before `a` never matters. Derived operators such as weak until are lowered to these nodes by tl-syntax before evaluation. The evaluator refuses a past operator with `UnsupportedPastNode`.

## Closed traces

`evaluate_closed` and `evaluate_closed_at` use `mltl.closed-trace/v1`. The trace is complete, and every proposition is false at an instant after its last one. A formula that reads beyond the end therefore sees false there. The verdict is `True` or `False`.

`evaluate_closed_at` takes the verdict time. A time past the end of the trace is valid and evaluates over the all-false tail.

## Open prefixes

`evaluate_prefix` and `evaluate_prefix_at` use `mltl.online-prefix/v1`. The `closed` argument states whether the trace is complete.

- With `closed = true`, instants after the last one are all false, as for a closed trace.
- With `closed = false`, a proposition at an unobserved instant is `Pending`.

The connectives follow the strong Kleene tables:

| Operator | Result |
| --- | --- |
| `Not` | Swaps `True` and `False` and keeps `Pending`. |
| `And` | `False` if either side is `False`, `True` if both are `True`, otherwise `Pending`. |
| `Or` | `True` if either side is `True`, `False` if both are `False`, otherwise `Pending`. |

The temporal operators fold their window with these tables and stop early when the result is decided. A verdict of `True` or `False` holds for every possible continuation of the prefix. `Pending` means the observed instants do not decide the formula. `observed_through` in the report is the last instant that was available, or `None` for an empty trace.

## Verdict and report

| Field | Meaning |
| --- | --- |
| `verdict` | `TruthValue::True`, `False` or `Pending`. A closed evaluation never returns `Pending`. |
| `verdict_time` | The instant the verdict refers to. |
| `trace_closed` | Whether the trace was declared complete. |
| `horizon` | The static lookahead of the formula; see [horizon.md](horizon.md). |
| `proposition_ids` | The propositions that the formula references, sorted. |

The report has the schema `tl-mltl.evaluation/v1`. It serializes with camelCase field names, and `deny_unknown_fields` applies when it is read.

## Refusals

`EvaluationError` has no variant that carries a fallback verdict.

| Variant | Cause |
| --- | --- |
| `UnsupportedProfile` | The formula uses another profile than the function implements. |
| `TraceNotStrictlyOrdered` | An instant lists a proposition twice or out of order. |
| `TimeOverflow` | Verdict time plus an offset exceeds `u64`. |
| `TemporalSpanExceeded` | One interval has more offsets than `max_temporal_span`. |
| `WorkLimitExceeded` | Node evaluations exceeded `max_node_evaluations`. |
| `RecursionDepthExceeded` | Formula nesting exceeded `max_recursion_depth`. |
| `Horizon` | The horizon computation failed. |
| `UnsupportedPastNode` | A past operator is in the formula. |
| `InvalidNodeReference` | A validated formula exposed a missing node. |

## Past-time evaluation

`evaluate_past` evaluates a formula under `mltl.origin-complete-history/v1` at an explicit anchor in a complete `PositionHistoryDocument`. The history holds, for each absolute position from the origin through a declared last position, the propositions that are true there.

| Node | Value at position `p` |
| --- | --- |
| `Once [a,b] x` | `x` holds at some `p-k` with `a <= k <= b`. |
| `Historically [a,b] x` | `x` holds at every `p-k` with `a <= k <= b`. |
| `StrongPrevious x` | `x` holds at `p-1`. |
| `Since [a,b] x y` | `y` holds at some `p-k` with `a <= k <= b`, and `x` holds at every `p-j` with `a <= j < k`. |
| `Triggered [a,b] x y` | The negation of `Since [a,b]` over the negations of `x` and `y`. |

A position below zero evaluates every proposition as false. The result is a two-valued `bool` with finality `Final`, because a later observation cannot change an anchored past result. No `Pending` exists on this wire.

A history declares a clock. `ClockBinding::EventPosition` states that positions are event counts and observations carry no timestamp. `ClockBinding::FixedSample` maps position `n` to `epoch + n * period` in an opaque unit, using exact rational numbers (`ExactNumber`). Dense, timestamped, duration, rounded, resampled and wall-clock inputs are refused as typed `UnsupportedClockKind` values.

The result is an immutable `PastEvaluationReport` with its own digest, the history identity and digest, the anchor, the clock, the evaluator revision, the limits and the observed work. A correction is a new result with a higher revision. Its relation is `Superseding` when new data replaces a valid predecessor, or `Invalidating` when the predecessor used invalid input. Both name the exact predecessor and the corrected history.

When the owner of a history reports that it is incomplete, unavailable, unsupported, failed, refused or in conflict, pass `PositionHistorySource::NonValue(state)`. The evaluator returns `PastEvaluationError::OwnerStatePreserved` with that state and no verdict.

## Infinite traces

With the `infinite-trace` feature, the `infinite` module evaluates the unbounded formula edition of tl-syntax over an ultimately periodic (lasso) trace: a finite prefix followed by a loop that repeats forever. The module does not call the finite evaluators. A result is evidence about the one trace it was given.

`evaluate_lasso` takes an `InfiniteFormulaDocument`, a `LassoTraceDocument`, optional fairness premises, an evidence-closure declaration, the exact graph and trace identities and an `EvaluationLimit`. Observations may be partial. The evaluator checks every Boolean completion of the unknown cells that satisfies the premises.

| Disposition | Meaning |
| --- | --- |
| `Proved` | Every admitted completion satisfies the formula at the selected position. |
| `Refuted` | Every admitted completion falsifies it. |
| `Inconclusive` | The admitted completions disagree, or none is admitted. |
| `Unsupported` | No procedure exists for the request. |
| `Failed` | Evaluation failed, including exhausted limits. |

The result also reports an execution axis, a truth axis, an evidence basis and a typed reason, together with the identities that bind it to its graph, proposition map and trace. `evaluate_model` returns `Unsupported` with the reason `ModelProcedureUnavailable`, because no procedure covers a whole transition system. `evaluate_prefix_safety` refutes `G[0,)` safety formulas from a finite prefix only when the prefix contains a violation that every continuation preserves, and otherwise returns `Inconclusive`. `ProviderRegistry` routes a request through the tl-syntax liveness capability.

`export_safety_monitor` renders the finite-horizon body of a safety formula as a C2PO expression that can supply refutation evidence only. `replay_target_step` compares one observed target verdict with the exact prefix. A target pass is reported as `Inconclusive` and never as proof of the infinite property.
