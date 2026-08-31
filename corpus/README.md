# tl-mltl conformance corpus

`tl-syntax-v1/` is a byte-identical retained copy of the MIT OR Apache-2.0
shared temporal corpus from `agent-ix/tl-syntax` revision
`740182f13b84858008d6f176f75136737d405c1b`, path `corpus/`. Its own
`SHA256SUMS` remains authoritative and is verified by `make check-corpus`.

tl-mltl consumes the formula, profile, trace, horizon, and closed-verdict fields
without changing their meaning. Evaluator-specific and external-monitor cases
live in separate versioned manifests so the upstream corpus bytes remain
reviewable and substitutable by digest.
