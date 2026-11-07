# API reference

Most items are re-exported at the crate root. The public modules are `future`, `past`, `mapping`, `wire`, `clock`, `ci_guard` and, with the `infinite-trace` feature, `infinite`. Errors are plain enums; match every variant or use a wildcard arm. The API is not stable.

## future

| Item | Purpose |
| --- | --- |
| `evaluate_closed(formula, formula_id, trace, trace_id, limits)` | Verdict at time 0 under `mltl.closed-trace/v1`. |
| `evaluate_closed_at(..., verdict_time, limits)` | The same at a chosen verdict time. |
| `evaluate_prefix(formula, formula_id, trace, trace_id, closed, limits)` | Verdict at time 0 under `mltl.online-prefix/v1`. |
| `evaluate_prefix_at(..., closed, verdict_time, limits)` | The same at a chosen verdict time. |
| `evaluate_closed_at_with_stats`, `evaluate_prefix_at_with_stats` | Return the result and `EvaluationStats` (node evaluations, deepest recursion). |
| `evaluate_closed_with_context`, `evaluate_prefix_with_context` | Bind the formula to a `SignalCatalogDocument` and return a `ContextualEvaluationReport`. |
| `analyze_horizon(formula, formula_id)` | `HorizonReport`: lookahead, propagation delay, required buffer. |
| `analyze_horizon_with_context` | The same with a catalog and context; returns `ContextualHorizonReport`. |
| `TruthValue` | `True`, `False` or `Pending`. |
| `EvaluationLimits` | `max_node_evaluations` (1,000,000), `max_temporal_span` (100,000), `max_recursion_depth` (512). |
| `EvaluationReport` | Schema `tl-mltl.evaluation/v1`. |
| `EvaluationError` | `UnsupportedProfile`, `TraceNotStrictlyOrdered`, `TimeOverflow`, `TemporalSpanExceeded`, `WorkLimitExceeded`, `RecursionDepthExceeded`, `Horizon`, `InvalidNodeReference`, `UnsupportedPastNode`. |
| `HorizonError` | `UnsupportedProfile`, `ArithmeticOverflow`, `InvalidNodeReference`, `UnsupportedPastNode`. |
| `ContextualEvaluationError`, `ContextualHorizonError`, `ContextualBindingError` | Failures of the contextual forms, including a missing proposition binding. |

## past

| Item | Purpose |
| --- | --- |
| `evaluate_past(formula, formula_id, history_source, anchor, proposition_map_id, result_revision, relation, limits)` | Anchored two-valued result as a `PastEvaluationReport`. |
| `evaluate_past_with_stats` | The same with `PastEvaluationStats`. |
| `analyze_required_history(formula, formula_id)` | `HistoryRequirementReport` with `required_positions`. |
| `PositionHistoryDocument` | `new`, `from_declared`, `corrected`, `validate` and accessors. Carries a digest. |
| `PositionObservation`, `ClockSample` | One position with its true propositions and, for a fixed-sample clock, its sample. |
| `PositionHistorySource` | `History(&document)` or `NonValue(OwnerHistoryState)`. |
| `OwnerHistoryState` | `Incomplete`, `Unavailable`, `Unsupported`, `Failed`, `Refused`, `Conflict`. |
| `ClockBinding` | `EventPosition`, `FixedSample { epoch, period, unit }` or `Unsupported { clock_kind }`. |
| `ExactNumber`, `fixed_sample_instant` | Exact rational arithmetic for fixed-sample clocks. |
| `PastEvaluationLimits` | `max_steps`, `max_temporal_span`, `max_recursion_depth`, `max_input_positions`, clamped to hard ceilings. |
| `PastEvaluationRelationInput` | `Original`, `Superseding(&prior)` or `Invalidating(&prior)`. |
| `PastResultRelationKind`, `PastResultFinality` | The relation kinds and the single finality, `Final`. |
| `PastEvaluationError` | Refusals, including `OwnerStatePreserved`, `AnchorOutOfRange`, `StepLimitExceeded` and `FutureNodeUnsupported`. |
| `POSITION_HISTORY_V1`, `HISTORY_REQUIREMENT_V1`, `PAST_EVALUATION_V1`, `PAST_EVALUATOR_V1` | Schema and evaluator identifiers. |
| `past::{history, requirement, result}` | Per document: `SCHEMA_BYTES`, `SCHEMA_SHA256`, `read(bytes, expected, limits)` and `derive`. |

## infinite (feature `infinite-trace`)

| Item | Purpose |
| --- | --- |
| `evaluate_lasso(&LassoRequest)` | Exact evaluation of a lasso trace and every admitted Boolean completion. |
| `evaluate_prefix_safety(&PrefixRequest)` | Refutes `G[0,)` safety formulas from a decisive finite prefix. |
| `evaluate_model(formula, graph_id, model_id, map_id)` | Returns `Unsupported` with `ModelProcedureUnavailable`. |
| `InfiniteProvider`, `ProviderRegistry` | Routes a request through the tl-syntax liveness capability; `settle_detailed` returns the full `InfiniteResult`. |
| `EvaluationLimit` | `max_nodes`, `max_positions`, `max_valuation_cells`, `max_states`, `max_completions`, `max_steps`, optional `deadline`. |
| `InfiniteResult`, `ResultIdentity` | The result axes and the identities that bind it to its graph and trace. |
| `Disposition` | `Proved`, `Refuted`, `Inconclusive`, `Unsupported`, `Failed`. |
| `EvidenceClosure` | `Closed` or `ProgressDeclared`. |
| `export_safety_monitor`, `replay_target_step` | Safety export to C2PO and replay of one target verdict, with `SafetyMappingManifest` and `SafetyReplayDisposition`. |
| `FEATURE`, `PROFILE`, `CAPABILITY` | `infinite-trace`, `mltl.infinite-trace/v1`, `tl-syntax.liveness/v1`. |

## mapping and differential

| Item | Purpose |
| --- | --- |
| `map_to_c2po(formula, formula_id, formula_bytes, source, external_tool, work_limit)` | C2PO expression for an online-prefix formula, as a `MappingManifest`. |
| `map_to_c2po_with_context` | The same with exact signal names; returns `ContextualMappingManifest`. |
| `map_past_to_c2po(formula, formula_id, formula_bytes, source, catalog, origin, work_limit)` | C2PO expression for a past formula, as a `PastMappingManifest`. |
| `MappingSourceIdentity`, `MappingSourceState` | Source revision and `Clean` or `Modified` tree state. |
| `TargetOriginContract` | The target and the operators with observed origin behavior. `reviewed_r2u2_4_2()` builds the retained one. |
| `MappingError`, `PastMappingError` | Typed refusals with no partial output. |
| `compare_external(&reference, external)` | `DifferentialReport` with `ComparisonStatus`. |
| `compare_external_with_context` | Contextual comparison with `ContextualComparisonStatus`. |
| `ExternalVerdict`, `ExternalStatus`, `ToolIdentity` | An external result, its status (`Conclusive`, `Pending`, `Unsupported`, `ToolError`) and the tool identity. |

## wire and clock

| Item | Purpose |
| --- | --- |
| `CommandDocument`, `Operation`, `CommandSchemaVersion` | The `tl-mltl.command/v1` request used by the CLI. |
| `TraceDocument`, `TraceSchemaVersion` | The `tl-mltl.trace/v1` trace. |
| `wire::command`, `wire::trace` | `SCHEMA_BYTES`, `SCHEMA_SHA256`, `read`, `derive`, `ValidatedCommand`, `ValidatedTrace`. |
| `OwnerLimits`, `OwnerReadError`, `OwnerReadErrorCode` | Bounded canonical readers and their refusal codes. |
| `clock` | Re-exports `ClockBinding`, `ClockSample`, `ExactNumber`, `fixed_sample_instant` and the clock errors. |

## Constants

| Item | Value |
| --- | --- |
| `TL_SYNTAX_CORPUS_REVISION` | `tl-syntax-corpus/v1` |
| `TL_MLTL_SOURCE_REVISION` | The tl-mltl revision the crate was built from, set by `build.rs`. |
