---
id: SR-012
title: Composite review of context-bound temporal result specification
type: SpecReview
analysis: base
scope: "agent-ix/tl-mltl#24; FR-007; StR-003; NFR-001; NFR-002; AD-001; CAC-001; AP-001; AA-001; test matrix"
review_set: all
relationships:
  - target: ix://agent-ix/tl-mltl/FR-007
    type: reviews
  - target: ix://agent-ix/tl-mltl/StR-003
    type: reviews
---

# SR-012: Composite review of context-bound temporal result specification

## Summary

Dependency, risk, evidence, integrity, scope, failure-domain, and EARS review
found no unresolved blocking ambiguity after eight findings were corrected. The
specification consumes the exact shared tl-syntax signal and caller-context
types, uses native v2 forms of the existing result families, and leaves existing
context-free library and CLI behavior at v1. No contract-IR, rewrite, external
monitor, or generic assurance runtime enters tl-mltl.

The mapping-name boundary was checked against the authoritative `C2POLexer` at
the exact retained R2U2 revision
`336a2453dd2bd89bd26e9e45fb772a4bf77e4a6a`, not inferred from current synthetic
`p<ID>` output. The shared syntax catalog intentionally accepts more UTF-8 names
than C2PO. Mapping therefore uses valid non-reserved names exactly and refuses
the rest; escaping or aliasing would break identity.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-1201 | high | Treating every tl-syntax signal name as a C2PO identifier would emit invalid or token-reinterpreted expressions. The pinned lexer accepts only ASCII identifier syntax and reserves tokens; mapping now refuses invalid/reserved names without normalization or aliasing. | FR-007 C2PO behavior, TC-026 |
| FND-1202 | high | Differential comparison could not detect context substitution if only the reference record carried context. Contextual external verdicts now carry their claimed catalog/context identity, and both v2 records must match before agreement. | FR-007 differential behavior, FR-007-AC-3, TC-027 |
| FND-1203 | high | A direct runtime dependency on tl-rewrite would create a Cargo cycle because tl-rewrite already calls tl-mltl for conformance. End-to-end composition now uses the shared context identities; tl-mltl names only participating tl-mltl/tl-syntax revisions. | FR-007 compatibility, FR-007-AC-7 |
| FND-1204 | medium | Reusing content digest fields for context-bound digests would destroy their current meaning and make an expression digest no longer identify expression bytes. Separate domain-separated request/result/comparison digests now bind context while content digests remain content-only. | FR-007 digest allocation, FR-007-AC-4 |
| FND-1205 | medium | “Non-Boolean direct use” risked duplicating an unreachable catalog error. A valid shared catalog already permits direct proposition bindings only to Boolean signals; tl-mltl owns missing formula binding and C2PO name refusal, not a second catalog validator. | FR-007 shared binding behavior, FR-007-AC-2 |
| FND-1206 | medium | Optional context represented by an omitted field would make omission indistinguishable from deliberate absence. Every contextual v2 record now requires the field and encodes absence as explicit `null`. | FR-007 versioning, FR-007-AC-5, TC-029 |
| FND-1207 | medium | “Consume typed signals” could expand this ticket into scalar trace evaluation or predicate lowering. Contextual evaluation remains Boolean proposition semantics; integer/decimal declarations may coexist unused but no scalar is coerced or interpreted. | FR-007 shared binding and C2PO behavior |
| FND-1208 | low | Preserved caller context and external context claims could be misread as verified provenance. The requirement and assurance argument now limit the claim to byte identity and comparison consistency. | StR-003 context, NFR-002-AC-4, AA-001 |

## Dispositions

| Finding | Disposition | Evidence |
|---|---|---|
| FND-1201 | **FIXED** | FR-007 states the lexer-derived regex and closed reserved set; TC-026 covers accepted neighbors and each refusal class. |
| FND-1202 | **FIXED** | ExternalVerdict and DifferentialReport gain contextual v2 forms; mixed/missing/mismatched context cannot agree. |
| FND-1203 | **FIXED** | FR-007 forbids the cycle and the #57-shaped fixture composes only shared tl-syntax values across local operations. |
| FND-1204 | **FIXED** | The digest-allocation section separates content, request, result, and comparison identities with schema/operation domains. |
| FND-1205 | **FIXED** | Catalog validity remains upstream; local errors cover only reachable formula resolution and mapping-name states. |
| FND-1206 | **FIXED** | V2 requires exact document or explicit `null`; omitted fields fail strict decoding. |
| FND-1207 | **FIXED** | Scalar predicate/value evaluation and a new trace schema are explicitly out of scope. |
| FND-1208 | **FIXED** | The specification preserves claims without attesting their truth or downstream qualification. |

## C2PO source reconciliation

The pinned user documentation states identifier syntax
`[a-zA-Z_][a-zA-Z0-9_]*` and lists reserved words. The exact lexer at the same
revision is the executable authority and additionally tokenizes `TAU` and `T`
as keywords, so the specification uses the lexer's superset. No gate executes
C2PO to rediscover that rule. Implementation tests the closed lexical predicate
and its reserved set locally, while retained differential evidence remains a
replay of the already pinned external artifacts.

The mapper emits an expression, not a complete `INPUT`/`FTSPEC` program. This
ticket changes atom rendering and manifest identity only. It does not add C2PO
declaration generation, signal-map generation, scalar predicate compilation, or
an assertion that C2PO accepted newly generated bytes.

## Failure-domain and compatibility review

- Formula binding precedes evaluation, horizon work, or mapping rendering.
  Missing binding is therefore not mislabeled as an evaluator result or partial
  mapping.
- Mapping-name refusal identifies the shared `SignalId`; missing binding
  identifies the shared `PropositionId`. Neither emits an expression.
- Differential context/version mismatch is distinct from truth/time mismatch;
  pending, unsupported, and tool error remain non-conclusive.
- Exact context absence is part of the request. Changing object to `null`,
  `null` to object, or omitting the field are three different cases.
- Existing content hashes and v1 snapshots retain their meaning and bytes.
  Contextual fields are accepted only under the corresponding v2 schema.
- The context-free CLI remains v1. A new command schema would require a separate
  reviewed need and is not smuggled into this change.

## Assurance and end-to-end boundary

The #57-shaped fixture uses named Boolean signals for an accepted overlay change
and a bounded response, plus one requirement revision, clause, anchor, and
clause span. It demonstrates that the same shared values survive every local
contextual operation. It does not parse FRETish, derive a catalog from IR, call
tl-rewrite, run C2PO/R2U2, or claim the future adapter is correct.

TC-030 may extend an existing domain producer and its current shared intake
declaration. It may not add a generic runner, collector, evidence envelope,
adapter framework, retention store, or new Make target. Hosted CI remains
manual-only.

## Review conclusion

FR-007 and StR-003 are sufficiently bounded for planning. Implementation must
wait for reviewed reachable tl-syntax#15 and tl-rewrite#21 revisions. It must
not copy their types or stack onto tl-mltl PR #23.
