---
id: TM-047
title: "LTLf evaluation, interchange and mission-closure test matrix"
type: TestMatrix
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-001
    type: covers
---

# TM-047: LTLf evaluation, interchange and mission-closure test matrix

Specification stage (TL-427). Every row is `🚧` until its build ticket lands it.
Rows marked DEFERRED need an outside tool or a solver that no one may run during
specification.

## Ticket acceptance ledger (P-TL-151)

Each row maps one acceptance criterion of a ticket to its requirement. Solver
requirements live in tl-solve (`FR-300` to `FR-399`, matrix TM-300).

| Ticket | Criterion | Requirement | Where |
|---|---|---|---|
| TL-427 | ADRs accepted | FR-093-AC-2 | tl-mltl |
| TL-427 | Requirements and test matrix written | FR-093-AC-1 | tl-mltl |
| TL-427 | Spec-review recorded | FR-093-AC-2 | tl-mltl |
| TL-428 | New profile identifier and admission rules; existing profiles unchanged | tl-syntax FR-332, FR-334 | tl-syntax |
| TL-428 | Weak and strong next both representable | tl-syntax FR-333; FR-083-AC-1 | tl-syntax, tl-mltl |
| TL-428 | Agreement with outside reference including the counterexample | FR-084-AC-1, FR-084-AC-2 | tl-mltl |
| TL-429 | New dialect; existing dialects unchanged | tl-parse FR-058, FR-062 | tl-parse |
| TL-429 | Profile always selected explicitly | tl-parse FR-059 | tl-parse |
| TL-429 | Round-trip, fuzz and hostile-input coverage | tl-parse FR-060, FR-061, NFR-046 | tl-parse |
| TL-430 | Witnesses re-evaluated under the LTLf profile | tl-solve FR-301 | tl-solve |
| TL-430 | Bounded miss is Inconclusive, never unsatisfiable | tl-solve FR-302 | tl-solve |
| TL-431 | Published FRET examples import with attribution | FR-087-AC-1, FR-088-AC-1, FR-089 | tl-mltl |
| TL-431 | Unsupported constructs produce a typed loss record | FR-091-AC-1, FR-091-AC-2 | tl-mltl |
| TL-431 | Mission-bounding report using horizon analysis (scope) | FR-090 | tl-mltl |
| TL-432 | Export then import is the identity on the supported subset | FR-086-AC-1 | tl-mltl |
| TL-432 | Constructs outside the subset refused naming the construct | FR-086-AC-2 | tl-mltl |
| TL-433 | Round-trip on the WEST corpus subset used by tl-rewrite | FR-092-AC-1, FR-092-AC-2 | tl-mltl |
| TL-434 | Accepted by at least two SMT solvers; verdicts agree with the CNF path | tl-solve FR-304, FR-305 | tl-solve |
| TL-434 | Solver identity and version recorded with each result | tl-solve FR-306 | tl-solve |
| TL-435 | Accepted by BLACK and by nuXmv | tl-solve FR-307, FR-308 | tl-solve |
| TL-435 | Results reported as external; witnesses re-evaluated | tl-solve FR-309 | tl-solve |

## Corpora and independently computed counts

| Corpus | Definition | Count and derivation |
|---|---|---|
| LTLf depth-one population | constants and atoms {p0,p1,true,false} (4); 5 unary operators on those 4; 6 binary operators on the 16 ordered pairs | 4 + 5x4 + 6x16 = 120 |
| Finite traces | length 1 to 4 over 2 atoms, 4 letters per position | 4 + 16 + 64 + 256 = 340 |
| LTLf reference rows | 120 formulas x 340 traces | 40,800; plus 20 rule and counterexample rows = 40,820 |
| C2PO identity set | formulas in `corpus/r2u2-v4.2/formulas.c2po` (FTSPEC) and `corpus/past-c2po-v1/target-4.2/past.c2po` (PTSPEC) | 6 + 6 = 12, counted by `;` terminators below each section header |
| C2PO refused constructs | one per enumerated construct kind | 10 |
| Loss kinds | one fixture per `LossKind` | 10 |
| Reader hostile cases | 8 classes x 3 readers | 24 |
| WEST lines | `corpus/west-v1/formulas_d1.txt` in tl-rewrite | 20 (`wc -l`), of which 10 `selectedCases` |
| Mission-closure comparisons | 40 temporal formulas x 2 mission times x 340 traces | 27,200 |
| Mission report population | 12 hand-authored formulas at M = 5 | 12 |
| FRET published examples | fixture manifest at retrieval time | not fixed here; directory census (DEFERRED retrieval) |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---------|-------|------|----------|-----------|--------|
| TC-227 | Evaluate the 18 rule cases and the LTLf reading of 2 counterexamples against manifest verdicts | Unit | P0 | FR-083-AC-1 | 🚧 not implemented (TL-428) |
| TC-228 | Refuse empty trace, foreign profile, unbound signal and cross-profile calls | Unit | P0 | FR-083-AC-2 | 🚧 not implemented (TL-428) |
| TC-229 | Check Xw, G and R duality laws over 120 formulas and 340 traces | Property | P0 | FR-083-AC-3 | 🚧 not implemented (TL-428) |
| TC-230 | Return the typed unbounded horizon and refuse at work limit minus one | Unit | P1 | FR-083-AC-4 | 🚧 not implemented (TL-428) |
| TC-231 | Replay 40,820 retained reference rows | Integration | P0 | FR-084-AC-1, StR-044-VC-1 | 🚧 blocked on retained corpus; generation DEFERRED |
| TC-232 | Replay the 4 counterexample entries with differing finite and infinite verdicts | Integration | P0 | FR-084-AC-2, StR-044-VC-1 | 🚧 blocked on retained corpus; generation DEFERRED |
| TC-233 | Refuse a corpus with missing provenance or altered rows | Unit | P1 | FR-084-AC-3 | 🚧 not implemented (TL-428) |
| TC-234 | Agree with the direct test-only evaluator on 40,820 rows and detect a flipped rule | Property | P0 | FR-084-AC-4 | 🚧 not implemented (TL-428) |
| TC-235 | Regenerate the reference corpus with the outside tool and diff against the retained bytes | Demonstration | P2 | FR-084-AC-1 | 🚧 DEFERRED: no outside tool is run during specification |
| TC-236 | Map all 11 result variants to LTL3, leave serialized names unchanged | Unit | P1 | FR-085-AC-1, FR-085-AC-2 | 🚧 not implemented (TL-428) |
| TC-237 | Export then import the 12 retained formulas to the identical graph and map | Integration | P0 | FR-086-AC-1 | 🚧 not implemented (TL-432) |
| TC-238 | Refuse each of 10 C2PO constructs naming the construct | Unit | P0 | FR-086-AC-2 | 🚧 not implemented (TL-432) |
| TC-239 | Refuse profile mismatch and absent profile on C2PO import | Unit | P0 | FR-086-AC-3 | 🚧 not implemented (TL-432) |
| TC-240 | Import each FRET MLTL fixture to its declared graph or loss records | Integration | P0 | FR-087-AC-1 | 🚧 blocked on fixture retrieval (TL-431) |
| TC-241 | Report a bare next as AmbiguousNext | Unit | P0 | FR-087-AC-2 | 🚧 not implemented (TL-431) |
| TC-242 | Import each FRET bundle with variables bound and header fields preserved | Integration | P0 | FR-088-AC-1 | 🚧 blocked on fixture retrieval (TL-431) |
| TC-243 | Produce loss records for unmapped, duplicate, non-Boolean variables and unknown header fields | Unit | P0 | FR-088-AC-2 | 🚧 not implemented (TL-431) |
| TC-244 | Carry attribution and licence on every foreign example and refuse one missing a field | Unit | P0 | FR-089-AC-1, FR-089-AC-2 | 🚧 not implemented (TL-431) |
| TC-245 | Classify the 12-formula mission report population at M = 5 | Unit | P0 | FR-090-AC-1 | 🚧 not implemented (TL-431) |
| TC-246 | Report mission-time-absent without a claim and refuse overflowing horizon | Unit | P1 | FR-090-AC-2 | 🚧 not implemented (TL-431) |
| TC-247 | Produce each of 10 loss kinds from its fixture | Unit | P0 | FR-091-AC-1, StR-044-VC-2 | 🚧 not implemented (TL-431) |
| TC-248 | Check imported plus unimported equals declared requirement count | Property | P0 | FR-091-AC-2, StR-044-VC-2 | 🚧 not implemented (TL-431) |
| TC-249 | Round-trip the 10 tl-rewrite WEST cases and compare graphs | Integration | P0 | FR-092-AC-1 | 🚧 not implemented (TL-433) |
| TC-250 | Round-trip all 20 WEST lines byte for byte | Integration | P0 | FR-092-AC-2 | 🚧 not implemented (TL-433) |
| TC-251 | Refuse the five unexportable WEST classes with loss records | Unit | P0 | FR-092-AC-3 | 🚧 not implemented (TL-433) |
| TC-252 | Verify retained WEST bytes and licence by digest against tl-rewrite | Integration | P1 | FR-092-AC-4 | 🚧 not implemented (TL-433) |
| TC-253 | Inspect that the ledger covers every ticket criterion and ADR acceptance and review are recorded | Inspection | P1 | FR-093-AC-1, FR-093-AC-2 | 🚧 pending human acceptance |
| TC-254 | Refuse unbounded to bounded conversion on every path without acknowledgement | Unit | P0 | FR-094-AC-1 | 🚧 not implemented (TL-431) |
| TC-255 | Compute the known-case table over 27,200 comparisons against an independent enumerator | Property | P0 | FR-094-AC-2 | 🚧 not implemented (TL-431) |
| TC-256 | Refuse an acknowledgement for a different formula, mission time or digest | Unit | P0 | FR-094-AC-3 | 🚧 not implemented (TL-431) |
| TC-257 | Check the 24 hostile reader cases against the manifest | Integration | P0 | NFR-047-AC-1 | 🚧 not implemented (TL-431) |
| TC-258 | Build the 3 reader fuzz targets and replay seeds | Fuzz | P1 | NFR-047-AC-2 | 🚧 not implemented (TL-431) |
