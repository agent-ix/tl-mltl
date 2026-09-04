---
id: MRS-001
title: tl-mltl v0.1 master requirements
type: MasterRequirements
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: depends_on
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

PGM-01 at `ix://agent-ix/quire-contract-ir/PGM-01` governs compatibility,
provenance, evidence, human authority, and qualification boundaries. Formula
and profile identities come from the exact tl-syntax revisions declared in
`assurance/pins.json`: the compiled dependency, and the separate revision whose
corpus bytes `corpus/tl-syntax-v1` retains.

## Scope

### In Scope

- Boolean and bounded Future, Globally, Until, and Release evaluation.
- Checked lookahead, propagation-delay, and required-buffer analysis.
- Explicit pending verdicts for incomplete prefixes.
- Versioned R2U2/C2PO mapping manifests and a deterministic CLI.
- Shared-corpus and external-tool differential evidence.
- Context-bound native evaluation, horizon, mapping, external-verdict, and
  differential report versions using shared tl-syntax types.

### Out of Scope

- Parsing source text or rewriting formulas.
- Signal-schema ownership, scalar predicate lowering, or contract-IR/FRETish
  translation.
- Continuous time, unbounded LTL, or probabilistic semantics.
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
typed context propagation. NFR-001 constrains determinism and resource failure;
NFR-002 constrains identity, provenance, and qualification claims.

## References

- [tl-mltl epic](https://github.com/agent-ix/tl-mltl/issues/7).
- [Typed context child](https://github.com/agent-ix/tl-mltl/issues/24).
- [Future FRETish consumer](https://github.com/agent-ix/quire-contract-ir/issues/57).
- [Pinned C2PO language](https://github.com/R2U2/r2u2/blob/336a2453dd2bd89bd26e9e45fb772a4bf77e4a6a/compiler/docs/user/language.md).
- [Pinned C2PO lexer](https://github.com/R2U2/r2u2/blob/336a2453dd2bd89bd26e9e45fb772a4bf77e4a6a/compiler/c2po/parse_c2po.py).
- [tl-syntax corpus](https://github.com/agent-ix/tl-syntax/tree/feat/tl-syntax-v0.1/corpus).
- [PGM-01](https://github.com/agent-ix/quire-contract-ir/blob/main/spec/program/PGM-01-governance.md).
