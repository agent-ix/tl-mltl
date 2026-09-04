---
id: PLAN-003
title: Context-bound temporal result implementation plan
type: Plan
status: in_progress
relationships:
  - target: ix://agent-ix/tl-mltl/FR-007
    type: references
  - target: ix://agent-ix/tl-mltl/StR-003
    type: references
  - target: ix://agent-ix/tl-syntax/FR-007
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/FR-007
    type: depends_on
---

# PLAN-003: Context-bound temporal result implementation plan

## Objective

Implement `agent-ix/tl-mltl#24` by consuming the exact shared tl-syntax signal
catalog and caller context, using validated Boolean names in C2PO expressions,
and binding the same identities into evaluation, horizon, mapping, external-
verdict and differential records. Preserve every context-free API, CLI behavior,
and v1 byte contract, and extend only native domain results and the existing
shared intake path.

## Base and landing

Specification begins from merged tl-mltl main revision
`4aeb62cb5fefc924a3921b22ab9074569b5537e2`, isolated from PR #23 at
`a2acc590b70cd3abea996a502f84e8f6f224a4e3`.

Implementation is blocked until tl-syntax#15 and tl-rewrite#21 land on reviewed
reachable revisions. The published pre-merge heads are specification inputs,
not temporary Cargo pins. Once both land, this branch shall incorporate current
main, repin tl-syntax to the exact reviewed revision, update
`TL_SYNTAX_REVISION` and the lockfile together, and verify that all other
consumers resolve the intended single syntax revision. tl-rewrite is a sequencing
dependency only and cannot become a Cargo dependency because it already depends
on tl-mltl.

## Dependency DAG

```text
tl-syntax#14 reviewed landing
  -> tl-syntax#15 reviewed landing
    -> tl-rewrite#21 implementation + reviewed landing
      -> exact tl-syntax dependency/revision pin in tl-mltl
        -> v1 snapshots + closed contextual v2 report forms
          -> shared catalog/context digest and binding helpers
            -> contextual closed/prefix evaluation + horizon
              -> exact named-Boolean C2PO mapping + refusal
                -> contextual external verdict + differential comparison
                  -> #57-shaped bounded overlay-response fixture
                    -> existing native producer + Quoin intake exercise
                      -> full local gate + code review + gap analysis
```

## Task File Mapping

| Task | Scope | Exit evidence |
|---|---|---|
| Task-001 | FR-007, StR-003, assurance impacts, matrix, C2PO source reconciliation, composite review | Grammar-clean specification and SR-012 with no unresolved blocking finding |
| Task-002 | Exact shared pin, native v2 fields, version-aware strict serde, digest helpers and v1 snapshots | V2 construction/round-trip controls and the wire portion of TC-029 |
| Task-003 | Context-aware closed/prefix evaluation and horizon analysis with pre-work formula binding | TC-025 and applicable TC-028 mutation classes |
| Task-004 | Exact valid signal-name rendering, C2PO refusals, contextual external verdict and differential comparison | TC-026, TC-027, and applicable TC-028 mutation classes |
| Task-005 | #57-shaped fixture, existing native Quoin intake, compatibility completion, full verification and closing reviews | TC-029 through TC-031, exact-head local gate, and resolved reviews |

## Implementation shape

- Reuse `tl_syntax::SignalCatalogDocument`, `SignalCatalog`,
  `RequirementContextDocument`, `FormulaBindingError`, `SignalId`, and
  `PropositionId` directly. Do not add local signal/context schema types or
  reproduce upstream catalog-validation variants.
- Add context-aware entry points alongside the current closed/prefix evaluation,
  horizon, mapping and comparison functions. Factor shared execution internally;
  existing functions continue producing v1 without fabricating an empty catalog
  or caller context.
- Evolve `EvaluationReport`, `HorizonReport`, `MappingManifest`,
  `ExternalVerdict`, and `DifferentialReport` as native schema families with
  closed contextual v2 forms. Version-aware serde enforces that v1 contains no
  contextual fields and v2 contains required catalog identity and a required
  context field whose value is the exact shared document or explicit `null`.
- Keep formula, trace, and expression content digest meanings intact. Add
  operation/schema-domain-separated `requestSha256`, `resultSha256`, and
  `comparisonSha256` fields as applicable, with self-digest fields excluded from
  their own preimages.
- Bind the formula against the shared catalog before evaluation, horizon, or
  mapping work. Traces remain ordered Boolean proposition observations; this
  ticket adds neither scalar samples nor a predicate language. A missing formula
  binding is a typed error using the shared proposition identity.
- In contextual mapping, resolve a proposition to its Boolean signal and render
  the exact signal name. Implement the pinned C2PO lexical predicate and closed
  reserved-token table in Rust. A bad name is a typed refusal identifying the
  signal, with no expression or manifest. Do not normalize, escape, alias,
  truncate, or invoke C2PO to validate it.
- Contextual external verdicts identify their claimed catalog and exact context.
  Comparison refuses mixed versions or identity/context mismatch before value/
  time agreement. It preserves pending, unsupported, and tool-error states and
  executes no external process.
- The #57-shaped fixture constructs an accepted-overlay and bounded-response
  formula from shared syntax values, two valid named Boolean signals, and one
  exact requirement revision/clause/anchor/span. It exercises every contextual
  operation but does not parse FRETish, derive IR fields, call tl-rewrite, or
  imply the future adapter is validated.
- Extend an existing native producer and current intake declaration only as
  necessary to retain one contextual domain result through Quoin. Add no generic
  runner, collector, evidence envelope, adapter framework, retention store,
  bespoke assurance schema, Make target, or hosted workflow behavior.

## Verification method

Before changing report structs, capture exact v1 JSON snapshots for all five
native record families and the existing CLI requests/outputs. TC-029 proves
these remain unchanged and exercises strict v2 decoding: exact document and
explicit `null`, missing field, unknown field, unsupported version, and mixed
version/field combinations.

TC-026 uses property/boundary tests for the ASCII identifier predicate and a
table test for every pinned reserved token, with accepted neighboring names.
TC-028 changes one catalog declaration, name, domain, binding, context field,
presence marker, formula, trace, limit, source/dependency revision, expression,
external identity, or outcome at a time and asserts the corresponding request,
result, or comparison identity changes or the operation refuses. Controls retain
the unchanged case beside each mutation class.

TC-030 uses the existing producer-owned shared path. Quoin retains the native
bytes; Quoin and Quire run no producer; C2PO and R2U2 are never executed. No
Python test helper or new orchestration target is introduced.

The full local `make ci CARGO_TARGET_DIR=target/cargo-review` gate runs at an
exact candidate head after implementation and closing fixes. Hosted CI remains
manual-only and is not dispatched by this plan.

## Exit Criteria

1. tl-syntax#15 and tl-rewrite#21 are reviewed and landed; Cargo resolves the
   exact reviewed tl-syntax revision and no tl-rewrite dependency cycle exists.
2. Every FR-007 and StR-003 criterion has a named executing trace symbol and all
   new matrix rows are marked implemented only after they run.
3. Context-aware evaluation, horizon, mapping, external verdict, and comparison
   preserve exact shared context, revisions, content identities, and typed
   non-success distinctions.
4. C2PO mapping uses exact valid non-reserved Boolean signal names and refuses
   every unsupported name or unresolved binding without output or external
   execution.
5. Existing APIs, CLI, semantic outcomes and exact v1 snapshots remain
   unchanged; all contextual v2 forms are strict and deterministic.
6. The #57-shaped fixture and one existing native producer demonstrate the
   shared context through local operations and Quoin intake without importing
   contract IR, tl-rewrite, Quire, Quoin, or a generic evidence/runtime layer.
7. The crate remains `publish = false`, `MIT OR Apache-2.0`, and makes no claim
   of provenance truth, external acceptance, universal rewrite equivalence,
   consuming-monitor qualification, accreditation, certification, or release.
8. The exact-head full local gate passes, code review and gap analysis contain
   no unresolved high or medium finding, and hosted CI was not dispatched.
