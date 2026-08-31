---
id: SR-003
title: Gap analysis of tl-mltl v0.1 candidate
type: SpecReview
analysis: gap-analysis
scope: spec/**/*.md, src/**/*.rs, tests/**/*.rs, corpus/**/*, CI and repository settings
review_set: all
---

# Gap analysis of tl-mltl v0.1 candidate

## Summary

Strict specification coverage reports all 46 rows backed and all 18 Rust
evidence symbols tagged. All-feature tests, lint, documentation, corpus
integrity, schema checks, and the retained eight-case R2U2 differential pass
locally. The candidate pins the exact tl-syntax, merged PGM-01, shared corpus,
R2U2/C2PO source, executable, configuration, and retained input/output
identities. Human review and source-release authority remain open; downstream
monitor timing, memory use, qualification, and accreditation remain outside the
candidate claim.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-301 | medium | The installed process module expects `Status` while the validated functional-coverage table contract requires `Coverage Status`; these two module contracts cannot both be satisfied by one header. Aggregate status classification is skipped and disclosed, while every underlying requirement criterion and test row is independently backed. | TM-001, SUITE-003 |
| FND-302 | low | Optional inspection-archetype and generic property-shape diagnostics are advisory and create neither an unbacked row nor a contradicted implementation status. | SUITE-003 |
| FND-303 | medium | The differential population covers eight unary, Until, Release, nested, and nonzero-time formula/time cases; closed-profile mapping remains explicitly unsupported, and this does not qualify R2U2 or establish production timing or memory behavior. | FR-005, AP-001, AA-001 |
| FND-304 | medium | AP-001 requires independent human code review and a source-release decision for the exact candidate; automation cannot approve, tag, publish, accredit, or certify it. | AP-001, AA-001 |
