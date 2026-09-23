# Changelog

All notable user-visible changes to `tl-mltl` are recorded here. The crate is
distributed as a git source release (`publish = false`); versions are git tags.

## 0.3.0

Part of the first coordinated release of the MLTL crates (`tl-syntax`,
`tl-parse`, `tl-mltl`, `tl-rewrite`), which all share the version 0.3.0. For
`tl-mltl` this release rebuilds against `tl-syntax` v0.3.0 and raises the MSRV
to match it. The first section lists changes since v0.2.0. The summary after it
covers everything since v0.1.0, the last release most consumers are upgrading
from.

### Changes since v0.2.0

- **Depends on the `tl-syntax` v0.3.0 release.** The dependency is pinned to
  the `v0.3.0` tag commit `4a5614193d21e5ae99950ae683b04ba0ec931358`, with the
  requirement `version = "=0.3.0"` (it was `=0.1.0` at `d52d8954`).
  `TL_SYNTAX_REVISION` and the `syntaxRevision` field of the C2PO mapping
  manifest report the new revision.
- **The past-history corpus digest follows the new pin.** tl-syntax v0.3.0
  rewrote two sentences in the past-history corpus README, so the corpus
  manifest digest changed from `59b86e7c…` to `0bb49748…`. The cases
  themselves did not change. The shared temporal and future-operator corpora
  are byte-identical to the previous pin.
- **MSRV is now Rust 1.98.1** (was 1.98). tl-syntax v0.3.0 needs 1.98.1, so the
  old value could not actually build the crate. `clippy.toml` and
  `rust-toolchain.toml` agree with it.
- **Guarded CI entry point.** `make guarded-ci` runs the full local gate
  through the `ci_guard` binary (`src/bin/ci_guard.rs`, logic in the public
  `ci_guard` module). The guard refuses to start Make when the Makefile or the
  environment could hide a failing gate, such as `.IGNORE:`, a dash-prefixed
  recipe, `MAKEFLAGS=-i`/`-k`, or `|| true`. After Make finishes, it checks
  that every declared `ci` gate wrote a completion record. `make ci` still
  works, but it is no longer the assured gate.
- **Deep formulas no longer overflow the stack in debug builds.** A formula
  nested deeper than the 512-level recursion limit now returns
  `EvaluationError::RecursionDepthExceeded` in unoptimized builds too. Before
  this fix, those builds could overflow the stack first.

### Summary since v0.1.0

- **Past-time (history) evaluation.** `evaluate_past` evaluates the closed
  `mltl.origin-complete-history/v1` profile (bounded Once, Historically, strong
  Previous, Since and Triggered) at an explicit anchor over a strict
  `tl-mltl.position-history/v1` input. It supports exact event-position and
  fixed-sample clocks (`ClockBinding`, `fixed_sample_instant`) and reports
  checked work (`evaluate_past_with_stats`). It never returns the future
  evaluator's `pending` value. `analyze_required_history` reports how much
  history a formula needs (`tl-mltl.history-requirement/v1`). Results are
  immutable original, superseding or invalidating records
  (`tl-mltl.past-evaluation/v1`, `PastResultRelation`). The shared
  past-history corpus is replayed through the evaluator.
- **W/M parity.** Weak-until (`W`) and strong-release (`M`) formulas, which
  tl-syntax lowers to the canonical F/G/U/R core, evaluate and map identically
  to their hand-written canonical equivalents. The evaluator has no separate
  W/M branch. C2PO mapping only ever sees the canonical graph, and the target's
  loss of the derived form is recorded as evidence.
- **Strict temporal request/result owner contracts.** Each contract this crate
  owns (`tl-mltl.trace/v1`, `tl-mltl.command/v1`,
  `tl-mltl.position-history/v1`, `tl-mltl.history-requirement/v1`,
  `tl-mltl.past-evaluation/v1`) ships its schema bytes under `schemas/` and a
  pinned SHA-256. Each is read through a bounded public
  `read(bytes, expected, limits)` that returns a validated view only
  (`wire::{trace, command}`, `OwnerLimits`, `OwnerReadError`).
- **Typed signals.** The contextual APIs (`evaluate_*_with_context`,
  `analyze_horizon_with_context`, `map_to_c2po_with_context`) check each
  formula against tl-syntax's typed signal catalog and proposition bindings.
  This release builds on the strict signal-catalog and requirement-context
  owner documents that tl-syntax v0.3.0 provides.
- **Public subsystem layout.** The modules `future`, `past`, `wire`, `clock`,
  `mapping` and `ci_guard` are now public. The existing root-level re-exports
  still work.
- **Evaluation stats.** `evaluate_closed_at_with_stats` and
  `evaluate_prefix_at_with_stats` return `EvaluationStats` alongside the
  report.
- **quire-observation dependency removed.** The temporal-assessment
  request/report and Contract-IR mapping boundary, which depended on the
  AGPL-licensed quire-observation crate, moved to the `quire-mltl` bridge crate
  in v0.2.0. tl-mltl now depends only on `serde`, `serde_json`, `sha2` and
  `tl-syntax`, with no Quire-ecosystem dependency, and its licence allow-list
  no longer admits AGPL.
- **No vendored corpora.** The shared temporal, future-operator and
  past-history corpora are read directly from the compiled dependency through
  `tl_syntax::CORPUS_DIR`. This repository no longer keeps a copy of them.
- **CI gate-set binding.** `make guarded-ci` (see above) is the assured local
  gate. Hosted CI runs only on manual dispatch.
- **Dependency on tl-syntax 0.3.0.** The dependency moved from v0.1.0
  (`26b801d4`) to v0.3.0 (`4a561419`). The requirement is exact (`=0.3.0`) and
  the runtime dependencies are pinned exactly.

Release gate: the full local gate (`make guarded-ci`) does not pass. It fails
only in its `spec` gate, whose strict coverage check (`quire coverage
--strict`) reports 122 rows with no backing test; every other gate passes. Each
of those rows belongs to work this release does not claim:
- the M4 corpus campaign (FR-008..010, NFR-004, TC-091..103)
- the M5 verification-effectiveness campaign (FR-020..024, NFR-005,
  TC-104..129)
- infinite-trace and liveness (FR-027..029, TC-086..089)
- FR-019, superseded by quire-mltl FR-002 and kept only as a historical record
  (TC-085)
- rows verified by inspection, which strict mode never counts as backed
  (FR-018 / TC-090, which is still planned, and NFR-006-AC-8 / TC-137)

The owner accepted this exception for 0.3.0. Later releases follow the release
gates specified in the V1 spec cycle. The repository has no
`make spec-release` target.

### Breaking changes

- **Rust older than 1.98.1 can no longer build the crate** (v0.1.0 declared
  1.75 and v0.2.0 declared 1.98). *Migration:* build with Rust 1.98.1 or newer
  and raise your own `rust-version` to match.
- **tl-syntax is required at exactly `=0.3.0`.** A dependent that also uses
  tl-syntax directly must use the same revision, or Cargo resolves two
  incompatible copies. *Migration:* pin tl-syntax to
  `rev = "4a5614193d21e5ae99950ae683b04ba0ec931358"`,
  `version = "=0.3.0"`.
- **`tl_syntax::SemanticProfile`, `NodeKind` and `FormulaSchemaVersion` have
  new variants**, which reach you through tl-mltl's API. *Migration:* handle
  `OriginCompleteHistoryV1`, the five past `NodeKind` variants and
  `FormulaSchemaVersion::V2` in every exhaustive `match` (see the tl-syntax
  0.3.0 changelog).
- **`EvaluationError` has a new variant `UnsupportedPastNode(NodeId)`.** The
  future evaluators return it when they are given a past-time node. *Migration:*
  add an arm for it to every exhaustive `match` on `EvaluationError`. Send
  history formulas to `evaluate_past`.
- **The CLI refuses history-profile formulas.** When a `tl-mltl.command/v1`
  evaluate request carries an `mltl.origin-complete-history/v1` formula, the
  CLI exits 2 with "origin-complete history formulas require the typed
  past-evaluation API". *Migration:* call `evaluate_past` from the library.
- **`TL_SYNTAX_CORPUS_BASIS` was removed** in v0.2.0, along with the vendored
  `corpus/tl-syntax-v1`, `corpus/past-history` and `corpus/future-operators`
  copies. *Migration:* read the corpora from `tl_syntax::CORPUS_DIR`. There is
  no separate corpus revision to track any more, because `TL_SYNTAX_REVISION`
  identifies it.
- **The quire-observation-coupled surface was removed** in v0.2.0. This covers
  `wire::{request, observation, report}`, `mapping::contract_ir`, the
  `temporal-assessment-request-v1`, `temporal-assessment-result-v1` and
  `contract-ir-result-map-v1` schemas, and `QUIRE_OBSERVATION_REVISION`. That
  surface never shipped in a tagged release before v0.2.0. *Migration:* use
  the `quire-mltl` crate for the owner-assertion request/result boundary.
- **The past-history corpus manifest digest changed.** This affects any
  consumer that pins the manifest bytes read through `tl_syntax::CORPUS_DIR`.
  *Migration:* update the pinned digest to
  `0bb497481a08d82ae74db794657eb6e7c57e6d1e5b5a8471b3559f82f405afd1` in the
  same change that moves the tl-syntax pin.
