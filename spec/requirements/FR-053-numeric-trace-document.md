---
id: FR-053
title: Carry Integer and fixed-Decimal observations in a strict numeric trace document
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-014
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-018
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-007
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-303
    type: depends_on
---

# FR-053: Carry Integer and fixed-Decimal observations in a strict numeric trace document

## Description

When a trace carries numeric signal values, tl-mltl shall read it only as
`tl-mltl.numeric-trace/v1` through a bounded strict reader that validates every
observation against the supplied signal catalog, and shall leave
`tl-mltl.trace/v1` unchanged.

## Inputs

- Canonical JSON bytes and a validated tl-syntax signal catalog.

## Outputs

- A validated `NumericTraceDocument` with a checked-in schema
  `schemas/numeric-trace-v1.schema.json`, an embedded-bytes constant and SHA-256
  constant, and typed `OwnerReadError` codes following the owner-contract pattern
  of `src/wire/`.

## Behavior

Wire shape (camelCase, `deny_unknown_fields`):

```
{ "schemaVersion": "tl-mltl.numeric-trace/v1", "traceId": str, "closed": bool,
  "catalogIdentity": hex-sha256,
  "instants": [ { "propositions": [PropositionId...],
                  "values": [ {"signal": u32, "integer": i64}
                            | {"signal": u32, "decimal": {"coefficient": i64, "scale": u8}} ] } ] }
```

- `propositions` is the existing strictly increasing true-set of Boolean-bound
  propositions. `values` is strictly increasing by `signal`; a signal absent from
  an instant is unobserved there (FR-055). A signal appears at most once per
  instant.
- `catalogIdentity` shall equal the content identity of the catalog supplied to
  the reader (`CatalogIdentityMismatch`).
- Admission against the catalog: the signal is declared (`UnknownSignal`) and
  numeric (`NonNumericSignal`); the kind matches the domain
  (`KindMismatch`); a decimal's scale equals the declared scale
  (`ScaleMismatch`); the coefficient or integer lies within the declared bounds
  (`ValueOutOfDomain`). Numbers are JSON integers; floats, exponents, `-0`,
  leading zeros, duplicate members, unknown members, trailing data and
  documents beyond the owner limits are rejected, with no partial output.
- Limits reuse `OwnerLimits` (positions, propositions) and add
  `max_values` (default and maximum 16,777,216 values per document).
- A trace document without `values` instants is legal and equals a Boolean trace
  of its `propositions`; the existing trace document is not a subset on the
  wire (distinct schema identity) and is never accepted by this reader.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-053-AC-1 | Integer and fixed-Decimal observations at scales 0, 1, 2, 9, 18 round-trip byte-identically through the strict reader; schema digest constants match the checked-in bytes. | integration-testing (TC-1570) |
| FR-053-AC-2 | Each admission and wire error class is produced by a minimal input with its own code and no output. | unit-testing (TC-1571) |
| FR-053-AC-3 | `tl-mltl.trace/v1` fixtures, schema and reader outcomes are unchanged, and the numeric reader rejects them. | integration-testing (TC-1572) |

## Dependencies

FR-018 owns the owner-wire pattern. tl-syntax FR-007 supplies catalog and domains.
