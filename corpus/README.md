# tl-mltl conformance corpus

The MIT OR Apache-2.0 shared temporal corpus (formula fixtures, malformed
cases, and their manifest), the future-operator corpus (`future-operators`
under `tl_syntax::CORPUS_DIR`), and the past-history corpus (`past-history`
under `tl_syntax::CORPUS_DIR`) are all read straight out of the compiled
`tl-syntax` dependency via `tl_syntax::CORPUS_DIR`, currently
`75ebec8ec8d15dcdee3a821119ae3ceb18e61bb3` (tl-syntax
`v0.3.0` development revision). There is no
retained copy of any of them in this repository and no separate corpus-basis
revision to track for any of them: each tracks whatever `TL_SYNTAX_REVISION`
names.

The future-operator corpus's manifest records the tl-parse revision it was
cross-checked against upstream; tl-mltl neither depends on nor re-runs that
parser. In this repository the manifest digest is `CORPUS_MANIFEST_SHA256` in
`tests/future_interop.rs` (TC-081 through TC-083).

`past-c2po-v1/target-4.2` retains one fresh C2PO/R2U2 4.2 exchange over six
past expressions and three observed positions. The manifest pins source commit,
compiler and executable identities, input and output bytes, and the exact
commands. The test compares every retained target verdict to the independently
computed past result at positions zero through two; R2U2's terminal flush at
position three is preserved as raw output and carries no source observation.

tl-mltl consumes the formula, profile, trace, horizon, and closed-verdict fields
without changing their meaning. Evaluator-specific and external-monitor cases
live in separate versioned manifests so the upstream corpus bytes remain
reviewable and substitutable by digest.
