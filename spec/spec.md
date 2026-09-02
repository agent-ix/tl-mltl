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
mapping, and differential evidence.

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

### Out of Scope

- Parsing source text or rewriting formulas.
- Continuous time, unbounded LTL, or probabilistic semantics.
- Reimplementing or qualifying R2U2 as a production monitor.
- Treating a local or differential pass as a release decision.

## System Overview

tl-mltl consumes validated tl-syntax graphs and ordered finite traces. Its pure
library layer returns typed verdicts and resource estimates. The CLI is a thin
serde adapter. External monitor execution remains an explicitly identified,
optional evidence backend.

## Requirements Architecture

FR-001 owns reference evaluation, FR-002 owns horizon/resource analysis,
FR-003 owns prefix semantics, FR-004 owns monitor mapping, and FR-005 owns CLI
and differential reports. NFR-001 constrains determinism and resource failure;
NFR-002 constrains identity, provenance, and qualification claims.

## References

- [tl-mltl epic](https://github.com/agent-ix/tl-mltl/issues/7).
- [tl-syntax corpus](https://github.com/agent-ix/tl-syntax/tree/feat/tl-syntax-v0.1/corpus).
- [PGM-01](https://github.com/agent-ix/quire-contract-ir/blob/main/spec/program/PGM-01-governance.md).
