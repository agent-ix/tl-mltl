---
id: FR-008
title: "Define a closed MLTL corpus coverage model"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-002
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-001
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-002
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-003
    type: depends_on
---

# FR-008: Define a closed MLTL corpus coverage model

## Description

When a corpus campaign manifest is validated, tl-mltl shall classify every
declared operator/profile/interval/trace cell as applicable, excluded, or
blocked and shall reject silent omissions, duplicates, and incompatible cells.

## Inputs

- Campaign identity `tl-mltl.corpus-campaign/v1` and coverage-cell identity
  `tl-mltl.coverage-cell/v1`.
- Exact admitted formula-schema, semantic-profile, operator-profile, clock, and
  history-profile identities.
- A closed dimension catalog and its complete list of coverage cells.

## Outputs

- A deterministic ordered cell census with separate applicable, excluded, and
  blocked populations.
- A typed refusal identifying every duplicate, omission, unknown dimension,
  stale dependency, invalid combination, or status contradiction.
- No conformance or release claim from the census alone.

## Behavior

The v1 dimension catalog is closed:

| Dimension | Required classes |
|---|---|
| formula/operator | constants, proposition, Not, And, Or, F, G, U, R; canonical-lowering pairs for landed W/M; conditional O/H/S/T; incompatible mixed-time graph |
| interval | not-applicable for Boolean nodes; `[0,0]`; `[0,b]` where `b>0`; `[a,a]` where `a>0`; `[a,b]` where `0<a<b`; `[u32::MAX,u32::MAX]`; inverted/malformed; resource-exceeding cardinality |
| observation shape | empty, singleton, one-short, exact required extent, longer, early witness, early counterexample, no witness/counterexample; conditional pre-origin, gapped, duplicate, out-of-order, and late-data history |
| semantic profile | `mltl.closed-trace/v1`, `mltl.online-prefix/v1`; conditional `mltl.origin-complete-history/v1`; unknown and profile-incompatible |
| result/progress | true, false, Pending, typed refusal; conditional anchored-final and superseding-history result |
| operation | validation, closed evaluation, prefix evaluation, horizon/history analysis, serialization, canonical formatting, rewrite equivalence, mapping/loss, CLI, and corpus replay |

The catalog enumerates equivalence and boundary classes, not every numeric value
or graph. Property and fuzz populations cover values inside a class; their
generated inputs never become implicit authoritative fixtures.

The v1 universe is the reviewed ordered cell-obligation registry, not the full
Cartesian product of the dimension enums. Every required class above appears
in at least one registry tuple; operation-specific inapplicable combinations
are represented by an explicit excluded tuple when they are a boundary the
campaign intends to retain. The registry is sorted lexicographically by the
UTF-8 bytes of the tuple tokens. Its `cellSetSha256` is pinned by every consumer
and by the acceptance test; deleting, inserting, or reordering a tuple requires
a reviewed successor campaign identity and leaves the old population available
for comparison.

Each cell is unique by this exact ordered JSON-array preimage, serialized as
UTF-8 with no insignificant whitespace:

```text
[campaignId,formulaOperator,intervalClass,observationClass,semanticProfile,resultClass,operationClass]
```

Every value is a closed ASCII enum token. `cellSha256` is lowercase hexadecimal
SHA-256 over the UTF-8 bytes of `tl-mltl.coverage-cell/v1`, one zero byte, and
that JSON array. `cellSetSha256` uses domain
`tl-mltl.coverage-cell-set/v1`, one zero byte, and the no-whitespace JSON array
of ordered lowercase `cellSha256` strings. Lifecycle state is excluded from a
cell key so a blocked-to-applicable transition cannot disguise the same concern
as a new cell; it remains included in the enclosing manifest digest. Each
record also names the owning fixture family, required oracle/evidence method,
dependency revisions, and one state:

- `applicable` requires exactly one canonical fixture reference and its
  independent expected result; one fixture may serve multiple cells, but each
  cell-to-fixture reference remains explicit;
- `excluded` requires a stable reason code proving the combination is invalid,
  irrelevant, redundant under canonical lowering, or outside scope; and
- `blocked` requires an unresolved ticket plus exact admission condition and
  cannot carry a conformance verdict or count as covered.

Boolean operators use `not-applicable` for interval rather than fabricated
`[0,0]`. W/M cells are applicable in the current baseline and bind their source
notation and exact primitive graph without creating a second
evaluator-semantic cell. Past cells bind an anchor and position-history class.
Mixed future/past cells are excluded unless a later reviewed profile admits
them.

For each applicable semantic cell, expected truth/progress and horizon/history
values are stored before replay and identify an independent Rust oracle or a
reviewed hand-derived boundary proof. An independent oracle shares only the
public canonical input types: it cannot call a production evaluator, horizon,
history, parser, formatter, rewrite, mapping, or CLI path, nor read the expected
record it is deriving. The production path cannot generate its own expected
value during the conformance run. Malformed and unsupported cells name the
rejection stage and typed reason instead of an expected Boolean.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-008-AC-1 | Every tuple in the reviewed v1 cell-obligation registry appears exactly once as applicable, excluded, or blocked under the specified tuple preimage and ordered-set identity; every required dimension class appears, and deleting, duplicating, reordering, reclassifying, or adding an unknown value makes the census fail while naming the complete conflicting cell set. | Test (TC-037, TC-038) |
| FR-008-AC-2 | Every applicable cell has exactly one canonical fixture and independent expected result, every excluded cell has a stable reason, and every blocked cell has an unresolved dependency and no verdict. | Test (TC-038, TC-039) |
| FR-008-AC-3 | Interval and observation partitions exercise every stated minimum, maximum, short/exact/long, witness/counterexample, progress, malformed, and resource boundary without treating a representative fixture as exhaustive value coverage. | Test (TC-039, TC-040, TC-041) |
| FR-008-AC-4 | Current W/M cells compare source lowering with the byte-identical canonical graph without adding evaluator semantics; past cells remain blocked and bind anchor/history when admitted; mixed-time and incompatible-profile cells remain excluded under their exact profile revisions. | Test (TC-042, TC-043, TC-047) |
| FR-008-AC-5 | A conformance replay consumes stored expected results and refuses an oracle that calls any owning production path, stale oracle identity, missing rejection stage/reason, or a generated campaign input presented as a canonical fixture. | Test (TC-039, TC-045) |

## Dependencies

Current future-v1 cells depend on FR-001 through FR-003. W/M admission is
satisfied by the exact landed revisions enumerated in MRS-002. Past/history
admission depends on accepted tl-syntax #38 and its routed tl-mltl
implementation. Native predicate cells remain blocked on quire-contract-ir #63
and native temporal bridge cells on #63/#64.
