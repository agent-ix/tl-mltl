---
id: SR-1514
title: "Scope and boundary analysis of P-TL-151"
type: SpecReview
analysis: scope-boundary
scope: "importer placement, crate boundaries, FRETish scope"
review_set: subset
---

## Summary

Importer placement in tl-mltl (ADR-046) is coherent: tl-mltl Cargo.toml depends on serde, serde_json, sha2 and tl-syntax only, no tl-parse, no quire crate; README states it neither depends on nor runs tl-parse; tl-parse NFR-002 provenance boundary is a sound reason to keep foreign grammars out. FRETish is out of scope consistently in ADR-046 and tl-parse ADR-041 item 6. Boundary defects below.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-15141 | high | FR-094-AC-1 refuses any unbounded formula at "the C2PO exporter" but tl-mltl already ships `export_safety_monitor` (src/infinite/export.rs:530) which deliberately accepts `G[0,)psi` with finite body and refuses with its own SafetyExportError variants (UnboundedLiveness, UnboundedUntilRelease, UnsupportedShape). The new refusal collides with or duplicates this boundary; carve-out or reconciliation is unspecified. | FR-094, src/infinite/export.rs |
| FND-15142 | medium | tl-solve FR-305 corpus consumes tl-mltl WEST and C2PO corpora (and so TL-432/TL-433 readers) but PLAN-300 and FR-305 Dependencies name neither; also C2PO formulas are `online-prefix` while FR-304 admits closed-trace only. | tl-solve FR-304, FR-305, PLAN-300 |
| FND-15143 | medium | tl-syntax changes: `NodeKind` is exhaustive (not non_exhaustive) and its `Future`, `Globally`, `Until`, `Release` carry a mandatory interval, so LTLf nodes cannot reuse them; the specs do not say whether LTLf is a new type (like InfiniteFormula) or new NodeKind variants (source-breaking for tl-mltl, tl-rewrite, tl-parse). ADR-044 consequences only mention SemanticProfile::ALL. tl-mltl pins tl-syntax by git rev 9f7b1c8 and tl-parse by 9607722; the pin bump sequencing after TL-428 is absent from PLAN-050. | tl-syntax ADR-044, PLAN-050 |
| FND-15144 | low | Closed WEST copy-versus-move decision (ADR-046 consequences) leaves two owners of corpus bytes until Peter decides. | ADR-046 |
