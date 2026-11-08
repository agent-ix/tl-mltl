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
