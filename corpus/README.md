# tl-mltl conformance corpus

The MIT OR Apache-2.0 shared temporal corpus (formula fixtures, malformed
cases, and their manifest), the future-operator corpus
(`corpus/future-operators`), and the past-history corpus
(`corpus/past-history`) are all read straight out of the compiled `tl-syntax`
dependency via `tl_syntax::CORPUS_DIR`, currently
`d52d89549b0a6c0c429261bab912cd5396c4a19e` on `tl-syntax` main. There is no
retained copy of any of them in this repository and no separate corpus-basis
revision to track for any of them: each tracks whatever `TL_SYNTAX_REVISION`
names.

The future-operator corpus's manifest records the tl-parse revision it was
cross-checked against upstream; tl-mltl neither depends on nor re-runs that
parser. In this repository the manifest digest is `CORPUS_MANIFEST_SHA256` in
`tests/future_interop.rs` (TC-081 through TC-083).

tl-mltl consumes the formula, profile, trace, horizon, and closed-verdict fields
without changing their meaning. Evaluator-specific and external-monitor cases
live in separate versioned manifests so the upstream corpus bytes remain
reviewable and substitutable by digest.
