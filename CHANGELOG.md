# Changelog

All notable user-visible changes to `tl-mltl` are recorded here. The crate is
distributed as a git source release (`publish = false`); versions are git tags.

## 0.4.0

Part of the second coordinated release of the MLTL crates (`tl-syntax`,
`tl-parse`, `tl-mltl`, `tl-rewrite`). All four crates move to 0.4.0 together,
and this crate depends on `tl-syntax` 0.4.0 (pinned to the release commit
`9607722f24ac1978b5e0e0ceeb0cb2a828e10177` with the requirement `=0.4.0`).
Changes are relative to v0.3.0. This is a breaking 0.x release; the API
migration inventory below lists every break reported by `cargo-semver-checks`
0.50.0.

### Added

- **Infinite-trace evaluation behind the `infinite-trace` feature.** Lasso and
  prefix evaluation, the successor-fixpoint reference procedure, partial
  valuations, and export of monitorable infinite safety refutations to C2PO.
  The U/R and S/T guard covers `[a,i)`, aligning the infinite-trace semantics
  with the finite ones.
- **Past-formula export to C2PO** (`map_past_to_c2po`, `PastMappingManifest`,
  `TargetOriginContract`).
- **Reproducible campaign.** `CampaignRun` with a member inventory
  (`campaign/members.json`), a report that states its populations, seeds, tool
  versions and non-claims before any ratio, and source-bound replay that
  refuses a moved or dirty source. Fuzz run records and Kani proof records are
  bound into the report, and stochastic repetitions are kept as separate runs.
  `make campaign-inventory`, `make campaign-run` and `make campaign-replay`
  are the entry points.
- **Verification evidence.** Seeded property laws, small-scope sweeps, a
  hand-authored boundary corpus, an independent infinite-trace reference
  corpus, six fuzz targets with recorded runs, bounded Kani proofs gated by
  `make kani-check` (each harness runs under a wall-clock budget,
  `TL_MLTL_KANI_BUDGET_SECS`, default 1800 s), a live R2U2 differential over
  past and future suites (`make r2u2-past-differential`, `make r2u2-nightly`),
  a limit register, an error-variant ledger, and `make miri`,
  `make test-features`, `make coverage` and `make coverage-branch`.
- **Mutation gate.** `make mutants` and `make mutants-diff` run cargo-mutants
  under a measured floor per group (`mutation-thresholds.toml`); the gate is the
  shared `tl-gate` tool built from tl-syntax, refuses a failing or unstable
  baseline, and requires a reason for every surviving mutant.

### Changed

- **Depends on the `tl-syntax` 0.4.0 release** (pinned to its release commit,
  requirement `=0.4.0`).
- **Digests of retained results change.** `resultSha256` of past evaluation
  reports and the digests of contextual evaluation, horizon and mapping records
  no longer cover a `syntaxRevision`, so a record written by v0.3.0 has a
  different digest from the same record written by 0.4.0.
- **Removed error variants that no input can produce (breaking).** The
  variants are listed one per symbol in the API migration inventory below. A
  validated formula, catalog, history or request already excludes each of them.
  The discriminants of the variants that follow a removed one shifted.
- `analyze_horizon_with_context` returns
  `ContextualHorizonError::Horizon(HorizonError::UnsupportedProfile)` for an
  unsupported profile, where it used to return `ContextualHorizonError::Identity`.
- `evaluate_past` no longer re-validates the freshly built report with an error
  path; it keeps a `debug_assert!` that the report validates, so a release build
  skips that check.
- `make miri` runs the unit and property suites under Miri and is a `ci` gate.

### Removed

- The compiled-in `tl-syntax` revision: `TL_SYNTAX_REVISION`, and the
  `syntax_revision` field of `MappingManifest`, `ContextualMappingManifest`,
  `ContextualEvaluationReport`, `ContextualHorizonReport` and
  `PastEvaluationReport`. The pinned revision is recorded in `Cargo.toml` only.
- `PGM01_POLICY_REVISION`, a policy revision constant no code consumed.
- `ToolIdentity::executable_sha256` and `ToolIdentity::configuration_sha256`.
- **Wire break (serialized records).** The record types reject unknown fields,
  so 0.4.0 refuses records written by v0.3.0 that still carry a removed field.
  Schema identities were not bumped, by decision for the pre-release 0.x line.
  Regenerate retained records with 0.4.0. The tl-syntax release gate's legacy
  decoder replay covers the `tl-parse` parser report only; it replays no
  tl-mltl record, so every refusal below is reviewer-checked from the serde
  definitions (`deny_unknown_fields`), not replay-proven.
  - `schemas/past-evaluation-v1.schema.json` (`tl-mltl.past-evaluation/v1`): Migration: drop `syntaxRevision` from records and regenerate them, because `resultSha256` no longer covers it; the schema (`additionalProperties: false`) rejects v0.3.0 records that carry it and 0.3.0 rejects new records. This is a declared golden exception of the release gate.
  - `tl-mltl.monitor-mapping/v1` (`MappingManifest`) and `tl-mltl.monitor-mapping/v2` (`ContextualMappingManifest`): Migration: drop `syntaxRevision` from manifests and regenerate them; v0.3.0 manifests carrying it are refused.
  - `tl-mltl.evaluation/v2` (`ContextualEvaluationReport`) and `tl-mltl.horizon/v2` (`ContextualHorizonReport`): Migration: drop `syntaxRevision` from reports and regenerate them; v0.3.0 reports carrying it are refused and their digests no longer match.
  - `tl-mltl.external-verdict/v1`, `tl-mltl.external-verdict/v2`, `tl-mltl.differential/v1` and `tl-mltl.differential/v2` (each embeds an external verdict whose `tool` is a `ToolIdentity`; `differential/v2` also embeds a contextual evaluation report): Migration: drop `executableSha256` and `configurationSha256` from the `tool` object, and `syntaxRevision` from an embedded contextual evaluation report, then regenerate the record; v0.3.0 records carrying them are refused.

### API migration inventory

- `enum_no_repr_variant_discriminant_changed` `HistoryRequirementError::ArithmeticOverflow`: Migration: do not cast `HistoryRequirementError` variants to integers with `as`; discriminants shifted when variants were removed, so match on the variant or use an explicit mapping.
- `enum_no_repr_variant_discriminant_changed` `HistoryRequirementError::FormulaIdentityEncoding`: Migration: do not cast `HistoryRequirementError` variants to integers with `as`; discriminants shifted when variants were removed, so match on the variant or use an explicit mapping.
- `enum_no_repr_variant_discriminant_changed` `HistoryRequirementError::InvalidFormulaIdentity`: Migration: do not cast `HistoryRequirementError` variants to integers with `as`; discriminants shifted when variants were removed, so match on the variant or use an explicit mapping.
- `enum_no_repr_variant_discriminant_changed` `OwnerReadErrorCode::Encoding`: Migration: do not cast `OwnerReadErrorCode` variants to integers with `as`; discriminants shifted when variants were removed, so match on the variant or use an explicit mapping.
- `enum_no_repr_variant_discriminant_changed` `OwnerReadErrorCode::ExpectedMismatch`: Migration: do not cast `OwnerReadErrorCode` variants to integers with `as`; discriminants shifted when variants were removed, so match on the variant or use an explicit mapping.
- `enum_no_repr_variant_discriminant_changed` `OwnerReadErrorCode::InvalidCombination`: Migration: do not cast `OwnerReadErrorCode` variants to integers with `as`; discriminants shifted when variants were removed, so match on the variant or use an explicit mapping.
- `enum_variant_missing` `ContextualBindingError::InvalidCatalog`: Migration: delete the match arm for `ContextualBindingError::InvalidCatalog`; a validated formula, catalog, history or request already excludes the case, so no input produces this variant.
- `enum_variant_missing` `EvaluationError::Horizon`: Migration: delete the match arm for `EvaluationError::Horizon`; a validated formula, catalog, history or request already excludes the case, so no input produces this variant.
- `enum_variant_missing` `EvaluationError::InvalidNodeReference`: Migration: delete the match arm for `EvaluationError::InvalidNodeReference`; a validated formula, catalog, history or request already excludes the case, so no input produces this variant.
- `enum_variant_missing` `EvaluationError::UnsupportedPastNode`: Migration: delete the match arm for `EvaluationError::UnsupportedPastNode`; a validated formula, catalog, history or request already excludes the case, so no input produces this variant.
- `enum_variant_missing` `HistoryError::IdentityEncoding`: Migration: delete the match arm for `HistoryError::IdentityEncoding`; a validated formula, catalog, history or request already excludes the case, so no input produces this variant.
- `enum_variant_missing` `HistoryRequirementError::FutureNodeUnsupported`: Migration: delete the match arm for `HistoryRequirementError::FutureNodeUnsupported`; a validated formula, catalog, history or request already excludes the case, so no input produces this variant.
- `enum_variant_missing` `HistoryRequirementError::InvalidNodeReference`: Migration: delete the match arm for `HistoryRequirementError::InvalidNodeReference`; a validated formula, catalog, history or request already excludes the case, so no input produces this variant.
- `enum_variant_missing` `HorizonError::InvalidNodeReference`: Migration: delete the match arm for `HorizonError::InvalidNodeReference`; a validated formula, catalog, history or request already excludes the case, so no input produces this variant.
- `enum_variant_missing` `HorizonError::UnsupportedPastNode`: Migration: delete the match arm for `HorizonError::UnsupportedPastNode`; a validated formula, catalog, history or request already excludes the case, so no input produces this variant.
- `enum_variant_missing` `MappingError::InvalidCatalog`: Migration: delete the match arm for `MappingError::InvalidCatalog`; a validated formula, catalog, history or request already excludes the case, so no input produces this variant.
- `enum_variant_missing` `MappingError::InvalidNodeReference`: Migration: delete the match arm for `MappingError::InvalidNodeReference`; a validated formula, catalog, history or request already excludes the case, so no input produces this variant.
- `enum_variant_missing` `MappingError::UnsupportedPastNode`: Migration: delete the match arm for `MappingError::UnsupportedPastNode`; a validated formula, catalog, history or request already excludes the case, so no input produces this variant.
- `enum_variant_missing` `OwnerReadErrorCode::ContractMismatch`: Migration: delete the match arm for `OwnerReadErrorCode::ContractMismatch`; a validated formula, catalog, history or request already excludes the case, so no input produces this variant.
- `enum_variant_missing` `OwnerReadErrorCode::EvaluationRefused`: Migration: delete the match arm for `OwnerReadErrorCode::EvaluationRefused`; a validated formula, catalog, history or request already excludes the case, so no input produces this variant.
- `enum_variant_missing` `OwnerReadErrorCode::IdentityMismatch`: Migration: delete the match arm for `OwnerReadErrorCode::IdentityMismatch`; a validated formula, catalog, history or request already excludes the case, so no input produces this variant.
- `enum_variant_missing` `PastEvaluationError::FutureNodeUnsupported`: Migration: delete the match arm for `PastEvaluationError::FutureNodeUnsupported`; a validated formula, catalog, history or request already excludes the case, so no input produces this variant.
- `enum_variant_missing` `PastEvaluationError::History`: Migration: delete the match arm for `PastEvaluationError::History`; a validated formula, catalog, history or request already excludes the case, so no input produces this variant.
- `enum_variant_missing` `PastEvaluationError::HistoryPositionAbsent`: Migration: delete the match arm for `PastEvaluationError::HistoryPositionAbsent`; a validated formula, catalog, history or request already excludes the case, so no input produces this variant.
- `enum_variant_missing` `PastEvaluationError::InvalidNodeReference`: Migration: delete the match arm for `PastEvaluationError::InvalidNodeReference`; a validated formula, catalog, history or request already excludes the case, so no input produces this variant.
- `enum_variant_missing` `PastEvaluationError::PositionArithmeticOverflow`: Migration: delete the match arm for `PastEvaluationError::PositionArithmeticOverflow`; a validated formula, catalog, history or request already excludes the case, so no input produces this variant.
- `enum_variant_missing` `PastEvaluationError::Result`: Migration: delete the match arm for `PastEvaluationError::Result`; a validated formula, catalog, history or request already excludes the case, so no input produces this variant.
- `enum_variant_missing` `PastResultValidationError::IdentityEncoding`: Migration: delete the match arm for `PastResultValidationError::IdentityEncoding`; a validated formula, catalog, history or request already excludes the case, so no input produces this variant.
- `pub_module_level_const_missing` `PGM01_POLICY_REVISION`: Migration: delete uses; the constant named a policy revision no code consumed.
- `pub_module_level_const_missing` `TL_SYNTAX_REVISION`: Migration: delete uses; the pinned tl-syntax revision is recorded in `Cargo.toml` only, and records no longer carry it.
- `struct_pub_field_missing` `ContextualEvaluationReport::syntax_revision`: Migration: delete reads and struct-literal initializers of the field; serialized records no longer carry `syntaxRevision` (see Removed).
- `struct_pub_field_missing` `ContextualHorizonReport::syntax_revision`: Migration: delete reads and struct-literal initializers of the field; serialized records no longer carry `syntaxRevision` (see Removed).
- `struct_pub_field_missing` `ContextualMappingManifest::syntax_revision`: Migration: delete reads and struct-literal initializers of the field; serialized records no longer carry `syntaxRevision` (see Removed).
- `struct_pub_field_missing` `MappingManifest::syntax_revision`: Migration: delete reads and struct-literal initializers of the field; serialized records no longer carry `syntaxRevision` (see Removed).
- `struct_pub_field_missing` `PastEvaluationReport::syntax_revision`: Migration: delete reads and struct-literal initializers of the field; serialized records no longer carry `syntaxRevision` (see Removed).
- `struct_pub_field_missing` `ToolIdentity::configuration_sha256`: Migration: delete the field from struct literals; serialized tool identities no longer carry `executableSha256` or `configurationSha256`, and records that still carry them are refused (see Removed).
- `struct_pub_field_missing` `ToolIdentity::executable_sha256`: Migration: delete the field from struct literals; serialized tool identities no longer carry `executableSha256` or `configurationSha256`, and records that still carry them are refused (see Removed).

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
- the M4 corpus campaign (FR-008..010, NFR-004, TC-092..101, TC-103; TC-091
  and TC-102 are retired)
- the M5 verification-effectiveness campaign (FR-020..024, NFR-005,
  TC-104..129 except the retired TC-119, TC-124, TC-128)
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
  copies. *Migration:* read the corpora from `tl_syntax::CORPUS_DIR`.
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
