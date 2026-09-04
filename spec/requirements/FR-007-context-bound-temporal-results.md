---
id: FR-007
title: Bind typed signals and caller context into temporal results
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-003
    type: implements
  - target: ix://agent-ix/tl-syntax/FR-007
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/FR-007
    type: depends_on
---

# FR-007: Bind typed signals and caller context into temporal results

## Description

When a caller supplies the shared tl-syntax signal catalog and optional
requirement context, tl-mltl shall resolve the formula through that catalog and
carry the exact context through evaluation, horizon analysis, C2PO mapping,
external-verdict comparison, and native structured evidence without changing
existing context-free APIs or v1 wire bytes.

## Inputs

- A validated tl-syntax formula, the operation's existing identity/options, and
  an ordered proposition trace where evaluation requires one.
- One validated `tl-syntax.signal-catalog/v1` document.
- Either no caller context or one complete validated
  `tl-syntax.requirement-context/v1` document.
- For contextual differential comparison, an externally supplied verdict that
  identifies the catalog and exact context its producer claims to have used.

## Shared context and binding behavior

- Context-aware entry points consume tl-syntax documents and binding APIs
  directly. They define no local signal, domain, proposition-binding,
  requirement, clause, anchor, or source-context schema.
- A shared signal-catalog document is intrinsically valid: catalog construction
  already rejects missing targets and non-Boolean direct proposition bindings.
  tl-mltl owns the later typed refusal when a formula proposition has no binding
  in that valid catalog, and reuses the shared proposition identity in the
  error. Extra declarations and bindings remain permitted and participate in
  the complete catalog identity.
- Context-aware closed evaluation, prefix evaluation, and horizon analysis bind
  the formula before semantic or resource work. They do not reinterpret trace
  observations as scalar values and do not invent predicates for integer or
  fixed-decimal declarations.
- Contextual reports carry the SHA-256 identity of the complete deterministic
  signal-catalog document, the exact shared requirement-context document or
  explicit absence, the exact tl-mltl source revision, and the exact tl-syntax
  dependency revision. The retained tl-syntax corpus basis remains a separate
  identity and is included only where the existing operation uses it.
- Operation- and schema-specific request digests bind the complete catalog
  document, exact context presence/value, formula structure and identity, trace
  and limits where applicable, source/dependency revisions, and every existing
  operation input. Output or comparison digests bind the corresponding request
  identity and native outcome so context mutation cannot preserve the result
  identity.

## C2PO mapping behavior

- A contextual C2PO mapping renders each proposition with the exact name of its
  bound Boolean signal instead of the legacy synthetic `p<ID>` alias.
- A usable C2PO name matches `[A-Za-z_][A-Za-z0-9_]*` and is not a token reserved
  by the exact pinned C2PO lexer: `STRUCT`, `ENUM`, `INPUT`, `DEFINE`, `FTSPEC`,
  `PTSPEC`, `foreach`, `forsome`, `forexactly`, `foratleast`, `foratmost`,
  `TAU`, `pow`, `sqrt`, `abs`, `xor`, `prev`, `G`, `F`, `H`, `O`, `U`, `R`,
  `S`, `T`, `M`, `true`, or `false`.
- A bound name outside that vocabulary is a typed unsupported-name refusal that
  identifies the signal and emits no expression or manifest. Names are never
  normalized, escaped, truncated, or replaced with an alias because doing so
  would silently change the shared identity.
- Only direct Boolean proposition bindings can reach mapping. Integer and
  fixed-decimal declarations may coexist unused in the catalog, but tl-mltl
  defines no predicate-lowering language and never coerces them to Boolean.
- Mapping remains a deterministic expression/manifest operation. It does not
  emit a full C2PO program, execute C2PO, execute R2U2, or claim external syntax
  acceptance beyond the pinned lexical contract.

The identifier and reserved-token set above comes from the `C2POLexer` at the
retained R2U2 source revision
`336a2453dd2bd89bd26e9e45fb772a4bf77e4a6a`. The lexer is authoritative where
its user documentation omits reserved `TAU` and `T`.

## Contextual differential behavior

- Context-aware external verdicts carry their claimed signal-catalog identity,
  exact requirement context or explicit absence, formula/trace identities, tool
  identity, status, value, and verdict time.
- Context-aware comparison requires the reference evaluation and external
  verdict to use the contextual v2 form. Catalog identity, exact context
  presence/value, formula identity, and trace identity must all match before
  truth value or verdict time may produce `agreement`.
- Missing contextual identity, mixed v1/v2 inputs, or any identity/context
  mismatch is a typed non-success distinct from semantic mismatch. Pending,
  unsupported, and tool-error external states remain non-conclusive and are
  never promoted to agreement.
- The comparison layer never executes or impersonates an external tool. An
  external context claim is preserved and compared; tl-mltl does not attest
  that the external producer actually used it.

## Versioning and compatibility

- Existing evaluation, prefix, horizon, mapping, comparison, CLI request, CLI
  output, shared-corpus, and retained R2U2 differential behavior remains
  unchanged when no catalog/context is supplied. Existing functions preserve
  their signatures and exact v1 native record bytes for identical inputs.
- Context-aware entry points emit closed v2 forms of the existing native
  `EvaluationReport`, `HorizonReport`, `MappingManifest`, `ExternalVerdict`, and
  `DifferentialReport` families. They do not introduce a generic or nested
  evidence envelope.
- V1 rejects contextual fields. Every v2 record requires its catalog identity
  and always serializes its requirement-context field as the exact shared
  document or explicit `null`; omission is not another spelling of absence.
  Missing fields, unknown fields, unsupported versions, and invalid version/
  field combinations fail decoding.
- The existing v1 CLI remains the context-free compatibility surface. A new CLI
  request schema, contract-IR/FRETish parser, or scalar trace representation is
  outside this ticket.
- Participating TL revisions in this crate are tl-mltl and tl-syntax. tl-rewrite
  cannot become a runtime dependency because it already consumes tl-mltl; the
  campaign composes rewrite and evaluation evidence through their shared
  context identities without introducing a dependency cycle.
- Native contextual records may traverse the existing Quoin intake, but
  tl-mltl imports or executes no Quoin, Quire, Engineering Assurance,
  contract-IR, rewrite, parser, runner, collector, or retention runtime.
- The crate remains `publish = false`, `MIT OR Apache-2.0`, and a reference and
  interoperability layer rather than a qualified production monitor.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-007-AC-1 | Context-aware closed/prefix evaluation and horizon analysis resolve every formula proposition through one shared catalog and carry its complete digest identity, exact optional requirement context, distinct clause span, operation inputs, outcomes, and exact tl-mltl/tl-syntax revisions. | Test (TC-025) |
| FR-007-AC-2 | Contextual mapping renders each proposition with its exact bound Boolean signal name; every invalid or pinned-reserved C2PO name and every unresolved proposition returns a typed refusal identifying the signal or proposition and emits no expression/manifest, while unused bounded scalar declarations are neither coerced nor rejected. | Test (TC-026) |
| FR-007-AC-3 | Contextual comparison returns agreement only when reference and external contextual versions, catalog identity, exact context presence/value, formula/trace identities, truth value, and verdict time agree; identity/context mismatch is distinct from semantic mismatch and pending/unsupported/tool-error remain non-conclusive. | Test (TC-027) |
| FR-007-AC-4 | Independently changing or dropping any catalog declaration, name, domain, binding, requirement id, revision, clause, anchor, span, presence marker, formula, trace, limit, source/dependency revision, mapping expression, external identity, or native outcome changes the applicable request/output/comparison digest or produces typed non-success. | Test (TC-028) |
| FR-007-AC-5 | Existing context-free APIs and CLI retain their signatures, semantic behavior, and exact v1 serialized snapshots; all five contextual v2 record families round-trip strictly and reject contextual field smuggling, omitted required fields, unknown fields, mixed versions, and unsupported versions. | Test (TC-029) |
| FR-007-AC-6 | Contextual native domain records traverse the existing producer-owned Quoin intake without Quoin, Quire, C2PO, or R2U2 executing a producer or monitor; no new generic runner, collector, evidence envelope, adapter framework, or retention path is added, and package license/publication settings remain unchanged. | Test (TC-030) |
| FR-007-AC-7 | A fixture shaped like quire-contract-ir#57's bounded “overlay change accepted, response within N cycles” requirement uses named Boolean signals and one exact requirement revision/clause/anchor/span consistently across contextual evaluation, horizon, mapping, external verdict, and differential records, with no contract-IR or rewrite runtime dependency. | Test (TC-031) |

## Dependencies

Depends on FR-001 through FR-006, the exact shared types introduced by
`agent-ix/tl-syntax#15`, and the reviewed shared context/digest contract from
`agent-ix/tl-rewrite#21`. Implementation lands only after both dependencies are
reviewed and reachable from their landing branches.
