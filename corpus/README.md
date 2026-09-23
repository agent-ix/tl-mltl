# tl-mltl conformance corpus

The MIT OR Apache-2.0 shared temporal corpus (formula fixtures, malformed
cases, and their manifest), the future-operator corpus (`future-operators`
under `tl_syntax::CORPUS_DIR`), and the past-history corpus (`past-history`
under `tl_syntax::CORPUS_DIR`) are all read straight out of the compiled
`tl-syntax` dependency via `tl_syntax::CORPUS_DIR`, currently
`8bcbce984f7ec3d86a92f90d866e842cc98b39fb` (tl-syntax
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
computed past result at positions zero through two. Those six sample traces do
not qualify all intervals. A second retained exchange proves that R2U2 4.2
returns true for `p S[0,2] q` at position two when the origin-complete result
is false. The adapter refuses that interval and other unqualified cells with a
typed error before producing an expression. R2U2's terminal flush at position
three is preserved as raw output and carries no source observation.

The origin grid used the same pinned 4.2 source and executable with all 1,024
five-position Boolean `p,q` traces, 25 past forms, and verdicts at positions
zero through four (128,000 comparisons). `S` and dual `T` at `[0,0]` and
`[0,1]` matched all 5,120 cells per form. At `[0,2]` each had 96 mismatches;
at `[1,1]` each had 1,024; at `[1,2]` each had 1,216. At `[2,2]` each had
1,152 mismatches and 1,024 missing origin verdicts. `O` and `H` matched for
`[0,0]`, `[0,1]`, `[0,2]`, `[1,1]`, and `[1,2]`, while `[2,2]` omitted the
origin verdict on all 1,024 traces. `Y` mapped to `O[1,1]` and matched all
5,120 cells. These are finite campaign results; the retained counterexample
above supplies the replayable unsoundness witness.

tl-mltl consumes the formula, profile, trace, horizon, and closed-verdict fields
without changing their meaning. Evaluator-specific and external-monitor cases
live in separate versioned manifests so the upstream corpus bytes remain
reviewable and substitutable by digest.
