# Monitor interoperability

tl-mltl maps formulas to C2PO, the compiler of the R2U2 runtime monitor, and compares verdicts that an external monitor produced with its own. It never runs R2U2 or C2PO. The external run happens out of band, and the crate reads the recorded result.

## Mapping a future formula

`map_to_c2po(formula, formula_id, formula_bytes, source, external_tool, work_limit)` renders an `mltl.online-prefix/v1` formula as a C2PO expression and returns a `MappingManifest` with the schema `tl-mltl.monitor-mapping/v1`.

| Field | Meaning |
| --- | --- |
| `expression` | The C2PO text, for example `(p0 U[1,2] p1)`. Propositions are written `p<id>`. |
| `input_sha256` | SHA-256 of the `formula_bytes` the caller supplied. |
| `output_sha256` | SHA-256 of the UTF-8 bytes of `expression`. |
| `proposition_ids` | The referenced propositions, in sorted order. |
| `adapter_version` | The version of this crate. |
| `source_revision`, `source_state` | The tl-mltl revision from `MappingSourceIdentity`, and whether its tree was `clean` or `modified`. |
| `external_tool` | The identity of a tool that actually ran, or `None` when none did. |
| `limitation` | States that the mapping does not establish external monitor timing, memory or qualification. |

The output is deterministic for identical inputs. Changing the formula bytes changes `input_sha256`, so a manifest can be matched to the exact document it came from.

`map_to_c2po_with_context` takes a `SignalCatalogDocument` and writes the exact shared signal names in place of `p<id>`. It refuses a name that is not a valid C2PO identifier or that collides with a C2PO keyword, with `MappingError::UnsupportedSignalName`. It returns a `ContextualMappingManifest` (schema `tl-mltl.monitor-mapping/v2`) with the catalog identity, the requirement context and request and result digests.

The mapping covers the future operators `F`, `G`, `U`, `R` and the Boolean connectives. It refuses with a typed `MappingError` and returns no partial expression for:

| Variant | Cause |
| --- | --- |
| `UnsupportedProfile` | The formula is not `mltl.online-prefix/v1`. A closed-trace formula is not reinterpreted as an online one. |
| `UnsupportedPastNode` | A past operator is in the formula. |
| `WorkLimitExceeded` | Rendering visited more nodes than `work_limit`. |
| `RecursionDepthExceeded` | Nesting exceeded 512. |
| `Binding`, `InvalidCatalog`, `UnsupportedSignalName`, `Identity` | Failures of the contextual form. |

## Mapping a past formula

`map_past_to_c2po` renders an `mltl.origin-complete-history/v1` formula for a C2PO past-time section. It needs a signal catalog and a `TargetOriginContract`, which names the exact target and the operators whose behavior at the history origin has been observed. `TargetOriginContract::reviewed_r2u2_4_2()` holds the observations retained for R2U2 4.2 with C2PO v4.1.0.

The adapter refuses a formula whose operators, intervals or nesting fall outside those observations (`PastMappingError::TargetOriginUnverified`, `TargetOriginShapeUnverified`, `TargetOriginIntervalMismatch`). The retained past corpus includes a case where the target returns true for `S[0,2]` where the source semantics return false, which is why the admission is limited to observed shapes.

## Comparing external verdicts

An external result enters the crate as an `ExternalVerdict` with the schema `tl-mltl.external-verdict/v1`.

| Field | Meaning |
| --- | --- |
| `tool` | A `ToolIdentity` with the tool name and exact version. |
| `formula_id`, `trace_id` | The identities the tool used. |
| `status` | `Conclusive`, `Pending`, `Unsupported` or `ToolError`. |
| `value`, `verdict_time` | Present only when the status is conclusive. |
| `detail` | An explanation for an unsupported or failed run. |

`compare_external(&reference, external)` returns a `DifferentialReport` whose `status` is one of three values.

| Status | When |
| --- | --- |
| `Agreement` | The identities match, the external result is conclusive, and the value and verdict time equal the reference. |
| `Mismatch` | The identities differ, or both results are conclusive and differ. |
| `NonConclusive` | The external status is not conclusive, or the reference verdict is `Pending`. |

The report keeps the reference verdict, the reference verdict time and the whole external record as separate fields. A pending, unsupported or failed run is never counted as agreement.

`compare_external_with_context` compares two version 2 records. It also requires that the catalog identity and the requirement context match, and adds the status `IdentityMismatch` when they do not.

## Retained R2U2 exchange

`corpus/r2u2-v4.2/` holds one run of the monitor built from the R2U2 `4.2-release` tag with C2PO 4.1.0.

| File | Contents |
| --- | --- |
| `manifest.json` | The trace, eight formulas with their expected verdicts and verdict times, and one case that is declared unsupported. |
| `formulas.c2po`, `signals.map` | The C2PO input and the mapping of signals to trace columns. |
| `spec.bin`, `trace.csv`, `r2u2.stdout` | The compiled specification, the trace and the raw monitor output. |
| `differential-report.json`, `differential-report.md` | The classification of each case. |

Eight supported cases agree in value and verdict time. The unsupported case is a closed-trace formula that the adapter refuses before any external run. R2U2 merges adjacent equal verdicts in its output, so the comparison keeps the time index of each selected record. The run covers `F`, `G`, `U` and `R` with non-zero lower bounds, nesting and non-zero verdict times. It measures no resource use.

`make differential` replays these bytes through `compare_external` with the `r2u2_differential` example. `corpus/past-c2po-v1/` holds the equivalent retained run for past formulas.
