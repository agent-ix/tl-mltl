# V5 fixed-selection mutation run at the c69 Stage 1 graph

This is a historical, source-bound V5 receipt. It passed the native
`v5_gate.verify` recomputation before this evidence directory was committed.
The commit carrying this directory changes the mlTL repository revision; a
later Stage 1 graph needs a fresh V5 run rather than credit from this one.

| Crate | Source revision |
| --- | --- |
| tl-syntax | `9de638dc4d14d0ae62a6825473a3a9bb6a9e57ac` |
| tl-parse | `89c2c1249c741bb94dc07da1eff46605af25e401` |
| tl-mltl | `c69a2d003f891edba1908dfe327cae03488dcd5f` |
| tl-rewrite | `e7ed9004e24803ac2611307cfe36b38480ec1a42` |

Cargo Mutants 27.0.0 ran each selected mutant with one worker, a 120-second
per-mutant timeout, the fixed test selection in `selection.json`, Rust 1.98.1,
offline Cargo, and an isolated writable Cargo home and temporary directory.
The four restored controls passed; no source tree was dirty. The full native
archives contain the baseline, selected identities, per-mutant build/test
outcomes and logs, invocation streams, and restored-control streams.

| Crate | Discovered | Selected | Unselected | Unviable | Viable | Caught | Missed | Timeout | Caught / viable |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tl-syntax | 67 | 18 | 49 | 1 | 17 | 17 | 0 | 0 | 100% |
| tl-parse | 249 | 23 | 226 | 3 | 20 | 20 | 0 | 0 | 100% |
| tl-mltl | 134 | 43 | 91 | 2 | 41 | 39 | 2 | 0 | 95.12% |
| tl-rewrite | 189 | 39 | 150 | 3 | 36 | 35 | 1 | 0 | 97.22% |

The selected viable population is 111 caught of 114 (97.37%). All four
selected critical scopes meet FR-047's 90% target. The 516 unselected mutants
remain explicit and carry no kill-rate credit.

## Missed-mutant dispositions

- `src/infinite/mod.rs:675:9`, `||` to `&&` in `evaluate_lasso`: proof candidate.
  Both admitted document constructors require `InfiniteTraceV1`, and
  `InfiniteClock` has only `EventPosition`. Neither side of this mismatch
  condition is reachable through the public API, so the mutant cannot alter
  an admitted result.
- `src/infinite/mod.rs:687:9`, `||` to `&&` in `evaluate_lasso`: proof candidate.
  The only newly admitted path has nodes over their limit while position and
  valuation-cell limits pass. `RawTraceRequest::validate` repeats the node
  ceiling, and `evaluate_lasso` maps that error to the same
  `ResourceIncomplete` result with the same identity and zero work counts.
  The fixed tests exercise one-over node, position, and cell limits.
- `src/infinite.rs:917:61`, `||` to `&&` in `check_infinite_rewrite`: proof
  candidate in tl-rewrite. Both admitted V1 document constructors require
  `InfiniteTraceV1` and the sole `EventPosition` clock. The mismatched
  profile/clock branch is unreachable.

These three identities remain in the selected and viable denominators. The
native report records each `proof_candidate` disposition and its detail; this
receipt does not assert a broader mutation score or human release acceptance.

## Artifact identities

- `selection.json`: `0e31c2a23130e36438c0d5a6c008369634c9aed3eabce79ec01e3b2269ba5698`
- `manifest.json`: `ba1c5b8350cfc0e88ac26af67093e19b0aefaf857a742c77f207afc43cb8a690`
- `report.json`: `164e6e0e7176dff94717e56f1f8d249215632f4734b16d1dec038a89a4424c4e`

The manifest binds each `discovery.json` and `native.tar.gz` digest. It names
the absolute run-time paths used by the native verifier, so a relocated copy
of these files is retained as immutable raw evidence rather than restamped
as a fresh execution.
