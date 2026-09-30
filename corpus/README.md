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
