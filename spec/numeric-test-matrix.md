---
id: TM-016
title: "Numeric predicates and contracts test matrix (tl-mltl)"
type: TestMatrix
relationships:
  - target: ix://agent-ix/tl-mltl/StR-014
    type: covers
---

# Numeric predicates and contracts test matrix (tl-mltl)

Project P-TL-150, tl-mltl portion: TL-422, TL-423, the evaluation half of TL-424
and TL-425, and the R2U2 differential evidence plan. Test ids TC-1570 through
TC-1609 are reserved for this project in tl-mltl. Every row is planned; rows
whose evidence requires fresh R2U2 or C2PO output are marked DEFERRED.

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-1570 | Numeric trace round-trips at scales 0, 1, 2, 9, 18; schema digests match | Integration | P0 | FR-053-AC-1 | 🚧 planned (TL-422) |
| TC-1571 | Every admission and wire error class from a minimal input | Unit | P0 | FR-053-AC-2 | 🚧 planned (TL-422) |
| TC-1572 | tl-mltl.trace/v1 unchanged and rejected by the numeric reader | Integration | P0 | FR-053-AC-3 | 🚧 planned (TL-422) |
| TC-1573 | Predicate truth equals independent table on the numeric corpus (closed) | Integration | P0 | FR-054-AC-1 | 🚧 planned (TL-422) |
| TC-1574 | Verdicts equal tl-oracle on the induced trace (closed and prefix) | Integration | P0 | FR-054-AC-1 | 🚧 planned (TL-422) |
| TC-1575 | Numeric-free traces give byte-identical results to existing evaluators | Integration | P0 | FR-054-AC-2 | 🚧 planned (TL-422) |
| TC-1576 | Generated expressions equal an independent exact reference, no overflow | Property | P0 | FR-054-AC-3 | 🚧 planned (TL-422) |
| TC-1577 | Aggregation expansion equals hand-written form on 567 population traces | Integration | P0 | FR-054-AC-4 | 🚧 planned (TL-424) |
| TC-1578 | Closed profile: unobserved predicate false, counted, negation true | Unit | P0 | FR-055-AC-1 | 🚧 planned (TL-422) |
| TC-1579 | Prefix profile: unobserved predicate Pending under all Kleene pairs | Unit | P0 | FR-055-AC-2 | 🚧 planned (TL-422) |
| TC-1580 | Origin-complete profile: gap refused, pre-origin false | Unit | P0 | FR-055-AC-3 | 🚧 planned (TL-422) |
| TC-1581 | Admitted predicates export to parseable C2PO with identities kept | Property | P0 | FR-056-AC-1 | 🚧 planned (TL-423) |
| TC-1582 | Legacy mapping manifests byte-identical | Integration | P0 | FR-056-AC-1 | 🚧 planned (TL-423) |
| TC-1583 | Each loss cause, strict read, no partial output | Unit | P0 | FR-056-AC-2 | 🚧 planned (TL-423) |
| TC-1584 | Node-kind by context partition, no wildcard | Unit | P0 | FR-056-AC-3 | 🚧 planned (TL-423) |
| TC-1585 | Admission test equals closed form on the synthetic-profile grid | Property | P0 | FR-057-AC-1 | 🚧 planned (TL-423) |
| TC-1586 | Admitted exports equal exact semantics at every grid value including boundaries | Property | P0 | FR-057-AC-1 | 🚧 planned (TL-423) |
| TC-1587 | Unpinned profile refuses all numeric predicates; Boolean export unchanged | Unit | P0 | FR-057-AC-2 | 🚧 planned (TL-423) |
| TC-1588 | R plus T at or above g/2 refuses with exact rationals | Unit | P0 | FR-057-AC-3 | 🚧 planned (TL-423) |
| TC-1589 | Retained numeric exchange replays; one-byte alteration fails | Integration | P0 | FR-058-AC-1 | ⛔ blocked: retained bytes DEFERRED |
| TC-1590 | Per-step verdicts recorded, three dispositions counted separately | Integration | P0 | FR-058-AC-2 | ⛔ blocked: retained bytes DEFERRED |
| TC-1591 | Nothing spawns R2U2 or C2PO; live target planned in campaign inventory | Inspection | P0 | FR-058-AC-3 | 🚧 planned (TL-423) |
| TC-1592 | Retained exchange for the 216-formula population exists with pinned profile | Demonstration | P1 | FR-058-AC-4 | ⛔ blocked: generation DEFERRED |
| TC-1593 | Nine-row contract status table on closed and prefix profiles | Unit | P0 | FR-059-AC-1 | 🚧 planned (TL-425) |
| TC-1594 | Never-exercised, invalid and verified contracts on the corpus | Integration | P0 | FR-059-AC-2 | 🚧 planned (TL-425) |
| TC-1595 | Graph-identity mismatch refused; report strict round-trip | Unit | P1 | FR-059-AC-3 | 🚧 planned (TL-425) |
| TC-1596 | Decomposition export of the six-contract grid parses to three formulas each | Unit | P0 | FR-060-AC-1 | 🚧 planned (TL-425) |
| TC-1597 | Unexportable clause refuses whole contract with loss record | Unit | P0 | FR-060-AC-2 | 🚧 planned (TL-425) |
| TC-1598 | Retained contract differential, 192 per-step records | Demonstration | P1 | FR-060-AC-3 | ⛔ blocked: generation DEFERRED |
| TC-1599 | Byte-identical repeat runs, no float types in numeric modules | Unit | P0 | NFR-017-AC-1 | 🚧 planned (TL-422) |
| TC-1600 | Two runs on one numeric trace give identical reports | Demonstration | P2 | StR-014-VC-1 | 🚧 planned (TL-422) |
| TC-1601 | Reviewer sees admitted, refused and disagreeing counted separately | Demonstration | P2 | StR-014-VC-2 | ⛔ blocked: generation DEFERRED |
| TC-1602 | Reviewer sees contract status per step beside R2U2's | Demonstration | P2 | StR-014-VC-3 | ⛔ blocked: generation DEFERRED |

## Differential populations

- Numeric: 36 Integer predicates (6 operators x 6 constants) + 36 Decimal predicates,
  3 wrappers each = 216 formulas; 8 traces x 4 steps; 1,728 case rows; 6,912 per-step records.
- Contracts: 3 assumptions x 2 guarantees = 6; 8 traces x 4 steps; 192 per-step records.
- Aggregation equivalence: 567 compared traces (tl-syntax FR-308).

## Ticket acceptance-criteria map (tl-mltl scope)

| Ticket | Acceptance criterion | Requirements | Test cases |
|---|---|---|---|
| TL-422 | Trace documents carry Integer and fixed-Decimal observations with a strict reader | FR-053 | TC-1570, TC-1571, TC-1572 |
| TL-422 | Missing numeric observations follow each profile's rules | FR-055 | TC-1578, TC-1579, TC-1580 |
| TL-422 | Agreement with the outside reference on the corpus | FR-054 | TC-1573, TC-1574, TC-1576 |
| TL-423 | R2U2 differential on the numeric corpus, per-step verdicts | FR-058 | TC-1589, TC-1590, TC-1591, TC-1592 (DEFERRED) |
| TL-423 | Unexpressible constructs refused with a typed loss record | FR-056, FR-057 | TC-1583, TC-1584, TC-1587, TC-1588 |
| TL-424 | Expanded form equivalent to hand-written form | FR-054-AC-4 | TC-1577 |
| TL-425 | Evaluated with three statuses | FR-059 | TC-1593, TC-1594, TC-1595 |
| TL-425 | Exported to C2PO contracts; R2U2 differential recorded | FR-060 | TC-1596, TC-1597, TC-1598 (DEFERRED) |

The master map for all tickets is in tl-syntax `spec/numeric-predicates-test-matrix.md`.
