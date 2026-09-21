# tl-mltl conformance corpus

The MIT OR Apache-2.0 shared temporal corpus (formula fixtures, malformed
cases, and their manifest) is read straight out of the compiled `tl-syntax`
dependency via `tl_syntax::CORPUS_DIR`, currently
`d52d89549b0a6c0c429261bab912cd5396c4a19e` on `tl-syntax` main. There is no
retained copy of it in this repository and no separate corpus-basis revision
to track: the corpus tracks whatever `TL_SYNTAX_REVISION` names.

`future-operators/` is a byte-identical copy of `corpus/future-operators` at
revision `5b1c13440e54d5a851df2d33cc88944135574bc6`, recorded separately as
`TL_SYNTAX_FUTURE_CORPUS_BASIS`; it is not restamped to the newer compiled
revision. Its upstream
`SHA256SUMS` names repository-root paths and is verified unchanged from the
repository root by `make check-corpus`. Its manifest records the tl-parse
revision it was cross-checked against; tl-mltl neither depends on nor re-runs
that parser. In this repository the manifest digest is `CORPUS_MANIFEST_SHA256`
in `tests/future_interop.rs` (TC-081 through TC-083); the vendored
`future-operators/README.md` names tl-syntax tests and constants and is itself
not digest-pinned.

tl-mltl consumes the formula, profile, trace, horizon, and closed-verdict fields
without changing their meaning. Evaluator-specific and external-monitor cases
live in separate versioned manifests so the upstream corpus bytes remain
reviewable and substitutable by digest.
