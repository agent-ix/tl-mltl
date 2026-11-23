---
id: NFR-047
title: "Keep the foreign-format readers bounded, deterministic and fuzzed"
type: NFR
quality_attribute: security
relationships:
  - target: ix://agent-ix/tl-mltl/FR-086
    type: constrains
  - target: ix://agent-ix/tl-mltl/FR-087
    type: constrains
  - target: ix://agent-ix/tl-mltl/FR-092
    type: constrains
---

# NFR-047: Keep the foreign-format readers bounded, deterministic and fuzzed

## Statement

When any of the C2PO, FRET MLTL or WEST readers processes untrusted bytes,
tl-mltl shall apply source-size, nesting and token ceilings before excess
retention and never panic.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Hostile cases per reader refused or accepted as manifest says (8 classes x 3 readers = 24) | 24 of 24 | 24 of 24 | Test |
| Fuzz targets `c2po_import`, `fret_mltl_import`, `west_import` replay checked seeds with no panic | 3 of 3 | 3 of 3 | Test |

## Verification

Each reader has the same 8 hostile classes: empty, non-UTF-8, NUL byte, source
limit exact and over (2), nesting limit exact and over (2), duplicate signal.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-047-AC-1 | The 24 hostile cases yield their manifest outcomes with stable typed errors. | Test (TC-330) |
| NFR-047-AC-2 | The three fuzz targets build and replay their seeds with no panic and assert export-import identity or loss-record presence on accepted input. | Test (TC-331) |
