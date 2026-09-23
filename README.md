# tl-mltl

[![Discord](https://img.shields.io/badge/Discord-Join%20us-5865F2?logo=discord&logoColor=white)](https://discord.gg/6qsdhSPE)

Deterministic finite-trace MLTL reference evaluation, checked horizon analysis,
pending-aware prefix semantics, and versioned R2U2/C2PO interoperability.

## Build

```bash
make guarded-ci
make spec
```

The library requires Rust 1.98.1 or later and consumes validated `tl-syntax` formulas
pinned to exact revision `43dcd3646d14922e20dee1be17b258be00ebf027`. The
shared temporal corpus, the future-operator corpus, and the past-history
corpus are all read directly from the compiled `tl-syntax` dependency via
`tl_syntax::CORPUS_DIR`, tracking `TL_SYNTAX_REVISION`. `evaluate_closed` implements
the declared all-false-after-closure profile at time zero, while
`evaluate_closed_at` selects another verdict time. `evaluate_prefix` and
`evaluate_prefix_at` preserve unknown future observations as `pending`.
`analyze_horizon` reports checked
lookahead, propagation delay, and buffer length. `map_to_c2po` emits a
digest-bearing mapping manifest without claiming that an external monitor ran.

The non-default `infinite-trace` feature exposes `tl_mltl::infinite` for
trace-scoped lasso evaluation, fairness-filtered partial observations, and
finite-prefix safety refutation. `G[0,)ψ` can be exported as a refutation-only
C2PO expression when its body has an admitted finite decision horizon and
verified target form. Model-wide proof is not supplied by this provider.

`evaluate_past` implements the closed `mltl.origin-complete-history/v1`
profile over strict `tl-mltl.position-history/v1` inputs. It evaluates bounded
O/H/Y/S/T at an explicit anchor, reports checked required history and work,
accepts exact event-position or fixed-sample clocks, preserves owner non-values
as typed errors, and emits immutable original/superseding/invalidating results.
It never returns the future evaluator's `pending` value.

The public subsystem layout is `future`,
`past::{history,requirement,evaluate,result}`, `wire::{trace,command}`,
`clock`, and `mapping::legacy`. Each retained owner contract (`tl-mltl.trace/v1`,
`tl-mltl.command/v1`, `tl-mltl.position-history/v1`,
`tl-mltl.history-requirement/v1`, `tl-mltl.past-evaluation/v1`) publishes
immutable schema bytes and a pinned digest through a bounded public
`read(bytes, expected, limits)` returning a constructor-private validated
view. Existing root-level evaluation, history, and legacy mapping paths
remain available as compatibility re-exports. tl-mltl publishes no
owner-assertion request/result contract of its own; binding this crate's
future/past evaluators to an external owner's assertion views (for example
Quire Observation) is left entirely to a dedicated integration crate, per the
architect ruling that keeps every `tl-*` crate independent of the Quire
ecosystem.

The `tl-mltl` binary accepts one `tl-mltl.command/v1` JSON document, either by
path or on stdin with `-`, and emits a versioned evaluation, horizon, or mapping
record.

## Bounded formal check

Kani is an optional, manual-only supplementary check. With Kani 0.68.0
installed, run:

```bash
cargo kani --lib \
  --harness future::horizon::kani_proofs::horizon_bound_addition_matches_checked_add \
  --exact --unwind 2 --solver cadical
```

The harness proves the checked horizon-bound addition primitive for all `u32` /
`u64` operands, including overflow refusal. It does not claim an unbounded MLTL
evaluator proof. It is part of local `make guarded-ci` and the dispatch-only
hosted gate, which installs the pinned Kani verifier before running the
aggregate.

## Corpora

- The shared `tl-syntax-corpus/v1`, the W/M future-operator corpus, and the
  past-history corpus are all read from the compiled `tl-syntax` dependency
  via `tl_syntax::CORPUS_DIR`; none is retained as a copy in this repository.
  Lowered W/M graphs map to C2PO only through the canonical graph; see FR-017.
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

Licensed under the MIT license. See [LICENSE](LICENSE).
