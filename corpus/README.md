# tl-mltl conformance corpus

`tl-syntax-v1/` is a byte-identical retained copy of the MIT OR Apache-2.0
shared temporal corpus from `agent-ix/tl-syntax` revision
`740182f13b84858008d6f176f75136737d405c1b`, path `corpus/`. Its own
`SHA256SUMS` remains authoritative and is verified by `make check-corpus`.

The copy is not restamped when the compiled `tl-syntax` dependency advances.
The crate is built against `953ee825e5060335b4c79682f5f41a78c5a1bfae` on
`tl-syntax` main; these bytes were taken at `740182f13b84858008d6f176f75136737d405c1b`
and the formula fixtures are identical between the two revisions. The compiled
revision and the corpus basis are two separate declared facts, cross-checked by
`scripts/check_shared_pins.py`, which refuses a tree in which they have been
collapsed into one string.

tl-mltl consumes the formula, profile, trace, horizon, and closed-verdict fields
without changing their meaning. Evaluator-specific and external-monitor cases
live in separate versioned manifests so the upstream corpus bytes remain
reviewable and substitutable by digest.
