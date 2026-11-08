# tl-mltl conformance corpus

The MIT OR Apache-2.0 shared temporal corpus (formula fixtures, malformed
cases, and their manifest), the future-operator corpus (`future-operators`
under `tl_syntax::CORPUS_DIR`), and the past-history corpus (`past-history`
under `tl_syntax::CORPUS_DIR`) are all read straight out of the compiled
`tl-syntax` dependency via `tl_syntax::CORPUS_DIR`. There is no
retained copy of any of them in this repository.

tl-mltl neither depends on nor runs tl-parse.

tl-mltl consumes the formula, profile, trace, horizon, and closed-verdict fields
without changing their meaning. Evaluator-specific and external-monitor cases
live in separate versioned manifests.

## Boundary corpus (`boundary-v1`)

The hand-authored boundary corpus is owned by tl-syntax (`boundary-v1` under
`tl_syntax::CORPUS_DIR`); there is no copy here. `tests/boundary_corpus.rs`
replays the closed-trace and origin-complete cases against both `tl-oracle` and
production, replays the online-prefix cases against production (the oracle does
not model that profile), and checks that every cell keeps its three cases. A case
tagged `work-limit` or `oracle-abstains` is checked by production only, because
the oracle's resource and empty-trace behaviour differs by design.

## Live R2U2 evidence (`r2u2-live-v1`)

`make r2u2-nightly` output against the pinned R2U2 (`campaign/tool-versions.json`,
revision `336a2453dd2bd89bd26e9e45fb772a4bf77e4a6a`, C monitor built with
`make -C monitors/c`, C2PO run by Python 3.14.7). `past/` is the TL-217 past-time
grid, `future/` the TL-237 exhaustive future suite. Each holds the generated
`spec.c2po` and `trace.csv`, R2U2's raw `r2u2.stdout`, and `report.json` with the
tool identities and input/output digests; `replay-report.json` is the source-bound
recomputation. `tests/r2u2_live.rs` replays both without running R2U2.
Past: 42 agree, 1791 refused at export (`TargetOriginShapeUnverified` 1683,
`TargetOriginIntervalMismatch` 108), 0 mismatches. Future: 1536 of 1536 case rows agree.
R2U2 prints one tuple per range of equal truth (aggregation), so the future
comparison expands each tuple over the positions it covers.
