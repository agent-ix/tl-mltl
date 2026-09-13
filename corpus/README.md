# tl-mltl conformance corpus

`tl-syntax-v1/` is a byte-identical retained copy of the MIT OR Apache-2.0
shared temporal corpus from `agent-ix/tl-syntax` revision
`740182f13b84858008d6f176f75136737d405c1b`, path `corpus/`. Its own
`SHA256SUMS` remains authoritative and is verified by `make check-corpus`.

The copy is not restamped when the compiled `tl-syntax` dependency advances.
The crate is built against `5b1c13440e54d5a851df2d33cc88944135574bc6` on
`tl-syntax` main after the reviewed W/M future-operator corpus was squash-merged; these
bytes were taken at `740182f13b84858008d6f176f75136737d405c1b`
and the formula fixtures are identical between the two revisions. The compiled
revision and the corpus basis are two separate declared facts, cross-checked by
`scripts/check_shared_pins.py`, which refuses a tree in which they have been
collapsed into one string.

`future-operators/` is a byte-identical copy of `corpus/future-operators` at
the compiled revision `5b1c13440e54d5a851df2d33cc88944135574bc6`. Its upstream
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
