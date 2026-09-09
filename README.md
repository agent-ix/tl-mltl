# tl-mltl

Deterministic finite-trace MLTL reference evaluation, checked horizon analysis,
pending-aware prefix semantics, and versioned R2U2/C2PO interoperability.

## Build

```bash
make ci
make spec
```

The library consumes validated `tl-syntax` formulas pinned to exact revision
`26b801d4a68ebfe720062cfdb3c66b070ab60e92` on tl-syntax `main` after the reviewed
semantic change was squash-merged. The
retained shared temporal corpus under `corpus/tl-syntax-v1` is a byte-identical
copy taken at the earlier revision `740182f13b84858008d6f176f75136737d405c1b`
and is deliberately not restamped; `TL_SYNTAX_REVISION` and
`TL_SYNTAX_CORPUS_BASIS` are two separate constants for that reason. `evaluate_closed` implements
the declared all-false-after-closure profile at time zero, while
`evaluate_closed_at` selects another verdict time. `evaluate_prefix` and
`evaluate_prefix_at` preserve unknown future observations as `pending`.
`analyze_horizon` reports checked
lookahead, propagation delay, and buffer length. `map_to_c2po` emits a
digest-bearing mapping manifest without claiming that an external monitor ran.

The `tl-mltl` binary accepts one `tl-mltl.command/v1` JSON document, either by
path or on stdin with `-`, and emits a versioned evaluation, horizon, or mapping
record.

## Bounded formal check

Kani is an optional, manual-only supplementary check. With Kani 0.67.0
installed, run:

```bash
cargo kani --lib \
  --harness horizon::kani_proofs::horizon_bound_addition_preserves_zero_and_refuses_overflow \
  --exact --unwind 4
```

The harness proves the checked horizon-bound addition primitive over its stated
finite unwinding bound, including overflow refusal. It does not claim an
unbounded MLTL evaluator proof. It is deliberately not part of `make ci` or
hosted CI.

## Corpora

- `corpus/tl-syntax-v1/` is the byte-pinned shared `tl-syntax-corpus/v1`.
- `corpus/r2u2-v4.2/` retains a real differential run of canonical R2U2 tag
  `4.2-release` at commit `336a2453…`, including C2PO inputs, compiled binary,
  raw verdicts, exact tool/configuration digests, and 8/8 supported formula/time
  agreements across unary, Until, Release, nested, and nonzero-time cases.
- Closed-profile mapping remains explicitly unsupported; it is not silently
  reinterpreted as online-prefix semantics.

## Development status

This crate is being developed spec-first. Its public API is not stable yet, and
registry publication is disabled until the v0.1 assurance review is complete.

Agent-assisted contributions are reviewed under the same requirements,
testing, provenance, and human release gates as every other contribution.

This crate is a reference and interoperability layer. Its results do not
validate, accredit, or qualify R2U2, another monitor, or a consuming project.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your
option.
