---
id: MRS-001
title: tl-mltl v0.1 master requirements
type: MasterRequirements
relationships:
  - target: ix://agent-ix/tl-syntax/MRS-001
    type: depends_on
---

# Master Requirements Specification

## Purpose

This specification defines a deterministic reference implementation of bounded
Mission-time Linear Temporal Logic over finite traces. It owns closed-trace
evaluation, decision horizons, pending-aware prefix verdicts, external-monitor
mapping, and differential evidence. When supplied, it also preserves the shared
typed signal catalog and caller requirement context through those native
results.

Formula and profile identities come from the single compiled tl-syntax
revision declared in `assurance/pins.json`, which this repository reads
directly via `tl_syntax::CORPUS_DIR`: the shared temporal corpus, the
future-operator corpus, and the past-history corpus alike.

## Scope

### In Scope

- Boolean and bounded Future, Globally, Until, and Release evaluation.
- Checked lookahead, propagation-delay, and required-buffer analysis.
- Explicit pending verdicts for incomplete prefixes.
- Versioned R2U2/C2PO mapping manifests and a deterministic CLI.
- Shared-corpus and external-tool differential evidence.
- Context-bound native evaluation, horizon, mapping, external-verdict, and
  differential report versions using shared tl-syntax types.
- Direct-versus-lowered parity controls for the tl-syntax derived W/M future
  operators, which reach the evaluator only as canonical primitive graphs.
- W/M canonical-graph C2PO export and explicit target-profile refusal over the
  retained tl-syntax future-operator corpus.

tl-mltl evaluates closed (`mltl.closed-trace/v1`) and open-prefix
(`mltl.online-prefix/v1`) bounded finite traces. Its non-default
`infinite-trace` feature exposes `tl_mltl::infinite`, the provider for the
TL-native `mltl.infinite-trace/v1` profile. That module consumes
`tl-syntax.formula-unbounded/v1`, lasso, fairness and partial-valuation
documents and registers the one `tl-syntax.liveness/v1` backend. QSL's
`quire.temporal.infinite-trace/v1` member is a correspondence for result
comparison. See FR-027 through FR-034, AD-002 and ADR-003.

### Out of Scope

- Parsing source text or rewriting formulas.
- Signal-schema ownership, scalar predicate lowering, or contract-IR/FRETish
  translation.
- Continuous time, unbounded temporal semantics outside the opt-in
  `mltl.infinite-trace/v1` profile, or probabilistic semantics.
- Reimplementing or qualifying R2U2 as a production monitor.
- Treating a local or differential pass as a release decision.

## System Overview

tl-mltl consumes validated tl-syntax graphs and ordered finite traces. Its
contextual entry points additionally consume the shared validated signal
catalog and optional requirement context. Its pure library layer returns typed
verdicts and resource estimates. The CLI is a thin serde adapter. External
monitor execution remains outside the crate; differential comparison consumes
identified external records without running their producer.

## Requirements Architecture

FR-001 owns reference evaluation, FR-002 owns horizon/resource analysis,
FR-003 owns prefix semantics, FR-004 owns monitor mapping, and FR-005 owns CLI
and differential reports. FR-006 owns shared assurance intake and FR-007 owns
typed context propagation. FR-016 owns direct-versus-lowered W/M parity
controls and adds no derived evaluator semantics. FR-017 owns W/M
canonical-graph interoperability and target loss evidence. FR-027 owns the
infinite-trace crate boundary, FR-028 owns liveness-backend registration
routing, and FR-029 owns infinite-trace downstream evidence and dependency
order. FR-030 through FR-034 specify the opt-in evaluator semantics. FR-038
through FR-042 own past C2PO mapping and the refutation-only infinite safety
fragment. FR-043 through FR-055 and NFR-007 through NFR-009 own the
executable V1 verification campaign in MRS-004. NFR-001 constrains
determinism and resource failure; NFR-002 constrains identity, provenance,
and qualification claims.

## References

FR-018 publishes the five artifact contracts tl-mltl itself owns (`trace`,
`command`, `position-history`, `history-requirement`, `past-evaluation`) with
their immutable schema/digest and strict-reading discipline. It changes no
existing future or past semantic profile. TL-179 removed the
`quire-observation`-coupled request/result/mapping owner boundary FR-018
previously described; that layer, and the QObs C00 compatibility dispatch
FR-019 pinned it to, now live in `quire-mltl` unchanged in behavior.

FR-019 pins that temporal owner boundary to accepted QObs C00 merge
`2bdeb833a330bfa777c19eb4c28c423f856f3ba6` from merged QObs PR #27 and
publishes explicit compatibility dispatch. It
delegates the supported temporal path to FR-018 and reports QObs repair-plan and
aggregate-query inputs as typed unsupported outcomes instead of recreating
their owner semantics. The upstream enablement prerequisite is satisfied.
FR-019 is now `status: superseded` by [quire-mltl
FR-002](https://github.com/agent-ix/quire-mltl/blob/main/spec/requirements/FR-002-dispatch-qobs-c00-compatibility.md)
per [ADR-001](decisions/ADR-001-retire-pgm01-citation-and-fr-019.md); this
paragraph is retained as the historical record of what it specified while it
governed `tl-mltl`'s own boundary, and TL-179 has since removed the
`wire::observation` implementation it describes.

FR-027 through FR-029 settle where infinite-trace (lasso, fairness) semantics
live: the non-default `tl_mltl::infinite` provider selected by ADR-003 under
`mltl.infinite-trace/v1`. They change no existing FR-001 through FR-019
behavior in the bounded core.

- [V1 executable verification campaign](./v1-verification-campaign.md)
  (MRS-004), replacing the historical
  [corpus](./corpus-campaign.md) and
  [verification-effectiveness](./verification-effectiveness-campaign.md)
  paperwork while retaining their actual test and corpus evidence.
- [TL ecosystem campaign](https://linear.app/agent-ix/issue/TL-88).
- [Typed context child](https://github.com/agent-ix/tl-mltl/issues/24).
- [Future FRETish consumer](https://github.com/agent-ix/quire-contract-ir/issues/57).
- [Infinite-trace scope, tl-mltl#72](https://github.com/agent-ix/tl-mltl/issues/72).
- [Infinite-trace provider scope, tl-mltl#68](https://github.com/agent-ix/tl-mltl/issues/68).
- [tl-syntax infinite-trace facet, tl-syntax#75](https://github.com/agent-ix/tl-syntax/issues/75).
- [Liveness profile holes, quire-specification#112](https://github.com/agent-ix/quire-specification/issues/112).
- [Pinned C2PO language](https://github.com/R2U2/r2u2/blob/336a2453dd2bd89bd26e9e45fb772a4bf77e4a6a/compiler/docs/user/language.md).
- [Pinned C2PO lexer](https://github.com/R2U2/r2u2/blob/336a2453dd2bd89bd26e9e45fb772a4bf77e4a6a/compiler/c2po/parse_c2po.py).
- [tl-syntax corpus](https://github.com/agent-ix/tl-syntax/tree/feat/tl-syntax-v0.1/corpus).
