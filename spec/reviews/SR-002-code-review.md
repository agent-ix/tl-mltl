---
id: SR-002
title: Code review of tl-mltl v0.1 implementation
type: SpecReview
analysis: code-review
scope: src/**/*.rs, tests/**/*.rs, corpus/**/*, schemas/**/*.json, scripts/**/*, Cargo.toml, Makefile, .github/workflows/ci.yml
review_set: all
---

# Code review of tl-mltl v0.1 implementation

## Summary

The agent code review examined operator semantics, closed and open trace
boundaries, checked horizon arithmetic, temporal expansion, recursion safety,
deterministic wire output, identity retention, external-tool separation,
corpus integrity, unsafe usage, and CI configuration. The review found that a
work budget alone did not bound call-stack depth for deeply nested validated
formula graphs. The evaluator and C2PO renderer now reject nesting beyond an
explicit process-safe boundary, and requirement-tagged negative tests exercise
both paths. Follow-up review also verified temporal lower-bound semantics,
source-identity freshness, retained-evidence anchors, build-script freshness,
non-git builds, and typed mapping identities. No unresolved code defect or
uncovered blocking requirement was found. Independent human review remains
mandatory under AP-001.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-201 | medium | Resolved: recursive evaluator and renderer paths now reject excessive formula nesting before further descent; the work and temporal-span budgets remain independently enforced. | FR-001-AC-3, FR-004-AC-2, NFR-001-AC-2 |
| FND-202 | low | Closed missing observations, open unknown observations, and unsupported external comparisons remain distinct from Boolean agreement throughout the API, CLI, and evidence records. | FR-001, FR-003, FR-005 |
| FND-203 | low | Retained R2U2 evidence comes from the canonical 4.2-release source and records executable, C2PO configuration, input, compiled-spec, and raw-output digests; eight declared formula/time cases cover unary, Until, Release, nesting, and nonzero verdict times. | StR-002, FR-004, FR-005 |
| FND-204 | high | Resolved: a deterministic exhaustive oracle and mutation controls pin nonzero lower-bound Globally, Until, and Release semantics independently of the implementation. | FR-001-AC-1 |
| FND-205 | high | Resolved: mapping revision/state follows the live Git ref and dirty state; build metadata watches the resolved ref and every tracked source while remaining fresh across consecutive default-target builds. | NFR-002-AC-1 |
| FND-206 | medium | Resolved: every retained outer manifest has a required executable anchor, and outcome summaries are re-derived from retained status files. | NFR-002-AC-2, NFR-002-AC-3 |
| FND-207 | low | Resolved: CLI identity tests use the build-time identity contract and therefore work in non-git exports that supply the documented overrides. | NFR-002-AC-1 |
| FND-208 | low | Resolved: mapping accepts a named source identity with a closed source-state enum, preventing revision/state transposition at call sites. | FR-004, NFR-002 |
