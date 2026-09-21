# tl-mltl

[![Discord](https://img.shields.io/badge/Discord-Join%20us-5865F2?logo=discord&logoColor=white)](https://discord.gg/6qsdhSPE)

Deterministic finite-trace MLTL reference evaluation, checked horizon analysis,
pending-aware prefix semantics, and versioned R2U2/C2PO interoperability.

## Build

```bash
make ci
make spec
```

The library requires Rust 1.98 or later and consumes validated `tl-syntax` formulas
pinned to exact revision `d52d89549b0a6c0c429261bab912cd5396c4a19e`.
Temporal owner requests additionally consume constructor-private Quire
Observation clock, progress, closure, completeness, and availability views at
accepted owner revision `924006300f45b38483be1cbdf99b68f899b7d368`
from merged QObs PR #27. tl-mltl does not mirror or reconstruct those owner types. The
shared temporal corpus is read directly from the compiled `tl-syntax` dependency
via `tl_syntax::CORPUS_DIR`, tracking `TL_SYNTAX_REVISION`. The future-operator
corpus is a byte-identical copy retained at the earlier revision
`5b1c13440e54d5a851df2d33cc88944135574bc6`, not restamped; `TL_SYNTAX_REVISION`
and `TL_SYNTAX_FUTURE_CORPUS_BASIS` name the two separate facts. `evaluate_closed` implements
the declared all-false-after-closure profile at time zero, while
`evaluate_closed_at` selects another verdict time. `evaluate_prefix` and
`evaluate_prefix_at` preserve unknown future observations as `pending`.
`analyze_horizon` reports checked
lookahead, propagation delay, and buffer length. `map_to_c2po` emits a
digest-bearing mapping manifest without claiming that an external monitor ran.

`evaluate_past` implements the closed `mltl.origin-complete-history/v1`
profile over strict `tl-mltl.position-history/v1` inputs. It evaluates bounded
O/H/Y/S/T at an explicit anchor, reports checked required history and work,
accepts exact event-position or fixed-sample clocks, preserves owner non-values
as typed errors, and emits immutable original/superseding/invalidating results.
It never returns the future evaluator's `pending` value.

The public subsystem layout is `future`,
`past::{history,requirement,evaluate,result}`,
`wire::{trace,command,observation,request,report}`,
`clock`, and `mapping::{legacy,contract_ir}`. Each temporal owner contract
publishes immutable schema bytes and a pinned digest. `wire::request::read`
admits one exact future or past request against independently supplied owner
views; `wire::report::{evaluate,read}` emits and revalidates one immutable
result; and `mapping::contract_ir::{map,read}` derives a TL-owned value or typed
non-value without importing Contract-IR vocabulary or coercing unavailable
states to Boolean values. Existing root-level evaluation, history, and legacy
mapping paths remain available as compatibility re-exports.
`wire::observation` delegates supported temporal handoff to the bounded request
adapter and reports QObs repair-plan and closed-population-query contracts as
typed unsupported compatibility rows.

The `tl-mltl` binary accepts one `tl-mltl.command/v1` JSON document, either by
path or on stdin with `-`, and emits a versioned evaluation, horizon, or mapping
record.

## Bounded formal check

Kani is an optional, manual-only supplementary check. With Kani 0.68.0
installed, run:

```bash
cargo kani --lib \
  --harness future::horizon::kani_proofs::horizon_bound_addition_matches_checked_add \
  --exact --unwind 4
```

The harness proves the checked horizon-bound addition primitive for all `u32` /
`u64` operands, including overflow refusal. It does not claim an unbounded MLTL
evaluator proof. It is part of local `make ci` and the dispatch-only hosted
gate, which installs the pinned Kani verifier before running the aggregate.

## Corpora

- The shared temporal corpus is read straight out of the compiled `tl-syntax`
  dependency via `tl_syntax::CORPUS_DIR`, tracking `TL_SYNTAX_REVISION`; there
  is no retained copy of it in this repository.
- `corpus/r2u2-v4.2/` retains a real differential run of canonical R2U2 tag
  `4.2-release` at commit `336a2453…`, including C2PO inputs, compiled binary,
  raw verdicts, exact tool/configuration digests, and 8/8 supported formula/time
  agreements across unary, Until, Release, nested, and nonzero-time cases.
- `corpus/future-operators/` is a byte-identical copy of the tl-syntax W/M
  future-operator corpus at the compiled revision. Lowered W/M graphs map to
  C2PO only through the canonical graph; see FR-017.
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
option. The required `quire-observation` dependency is AGPL-3.0-or-later and its
required `agent-ix-baseline-producer` dependency is AGPL-3.0-only; consumers
and distributors of the combined dependency graph must comply with that
dependency's license terms.
