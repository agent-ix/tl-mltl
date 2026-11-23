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

Identifier scope. Ids in this repository are local, and ids cited from another
repository are qualified with its name (`tl-syntax FR-332`, `tl-solve TC-307`,
`filament-specification FR-092`). The same number can therefore denote different
things in different repositories: this repository's FR-092 (WEST import and export)
is not `filament-specification` FR-092, which existing source cites as
"FSpec FR-092" (`src/infinite/periodic.rs`, `tests/tc149_semantic_laws.rs`) and as
"FR-092-AC-9" (`tests/infinite_trace.rs`). The rule is to qualify, not to renumber:
those citations are `filament-specification` ids and are requalified when the file is
next touched. This ledger's TC numbers start at TC-300 so that they cannot collide
with the TC-257 and TC-258 of the TL-147 branch.

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
| TL-430 | Bounded witness search (scope) | tl-solve FR-300 (population check needs the DEFERRED FR-084 corpus) | tl-solve |
| TL-430 | Bounded miss is Inconclusive, never unsatisfiable | tl-solve FR-302 | tl-solve |
| TL-431 | Published FRET examples import with attribution | FR-087-AC-1, FR-088-AC-1, FR-089 | tl-mltl |
| TL-431 | Unsupported constructs produce a typed loss record | FR-091-AC-1, FR-091-AC-2 | tl-mltl |
| TL-431 | Mission-bounding report using horizon analysis (scope) | FR-090 | tl-mltl |
| TL-427 | Mission-bound closure policy (decision item) | FR-094-AC-1 to FR-094-AC-4 | tl-mltl |
| TL-427 | LTL3 verdict names (decision item) | FR-085-AC-1, FR-085-AC-2 | tl-mltl |
| TL-432 | Export then import is the identity on the supported subset | FR-086-AC-1 | tl-mltl |
| TL-432 | Constructs outside the subset refused naming the construct | FR-086-AC-2, FR-086-AC-4 | tl-mltl |
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
| C2PO identity set | formulas in `corpus/r2u2-v4.2/formulas.c2po` (FTSPEC) and `corpus/past-c2po-v1/target-4.2/past.c2po` (PTSPEC) | 6 future + 6 past lines = 12 formula lines (counted by `;` terminators below each section header, `INPUT` lines excluded), of which `O[1,1](p);` appears twice in `past.c2po`, so 11 distinct (`sort -u`) |
| C2PO refused constructs | one per enumerated construct kind | 10 |
| Loss kinds | one fixture per `LossKind` | 10 |
| Reader hostile cases | 8 classes x 3 readers | 24 |
| WEST lines | `corpus/west-v1/formulas_d1.txt` in tl-rewrite | 20 (`wc -l`), of which 10 `selectedCases` |
| Mission-closure comparisons | 40 temporal formulas x 2 mission times x 340 traces, plus 8 next formulas (Xs, Xw on 4 operands) x 340 traces | 27,200 + 2,720 = 29,920 |
| Mission report population | 4 + 4 + 4 class assignments over hand-authored formulas at M = 5, one formula carrying two classes | 12 assignments realized by 11 formulas |
| FRET published examples | fixture manifest at retrieval time | not fixed here; directory census (DEFERRED retrieval) |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---------|-------|------|----------|-----------|--------|
| TC-300 | Evaluate the 18 rule cases and the LTLf reading of 2 counterexamples against manifest verdicts | Unit | P0 | FR-083-AC-1 | 🚧 planned |
| TC-301 | Refuse empty trace, foreign profile, unbound signal and cross-profile calls | Unit | P0 | FR-083-AC-2 | 🚧 planned |
| TC-302 | Check Xw, G and R duality laws over 120 formulas and 340 traces | Property | P0 | FR-083-AC-3 | 🚧 planned |
| TC-303 | Return the typed unbounded horizon and refuse at work limit minus one | Unit | P1 | FR-083-AC-4 | 🚧 planned |
| TC-304 | Replay 40,820 retained reference rows | Integration | P0 | FR-084-AC-1, StR-044-VC-1 | 🚧 blocked on retained corpus; generation DEFERRED |
| TC-305 | Replay the 4 counterexample entries with differing finite and infinite verdicts | Integration | P0 | FR-084-AC-2, StR-044-VC-1 | 🚧 blocked on retained corpus; generation DEFERRED |
| TC-306 | Refuse a corpus with missing provenance or altered rows | Unit | P1 | FR-084-AC-3 | 🚧 planned |
| TC-307 | Agree with the direct test-only evaluator on 40,820 rows and detect a flipped rule | Property | P0 | FR-084-AC-4 | 🚧 planned |
| TC-308 | Regenerate the reference corpus with the outside tool and diff against the retained bytes (DEFERRED: no outside tool is run during specification) | Demonstration | P2 | FR-084-AC-1 | 🚧 planned |
| TC-309 | Map all 11 result variants to LTL3, leave serialized names unchanged | Unit | P1 | FR-085-AC-1, FR-085-AC-2 | 🚧 planned |
| TC-310 | Export then import the 11 distinct retained formulas (12 lines) to the identical graph and map | Integration | P0 | FR-086-AC-1 | 🚧 planned |
| TC-311 | Refuse each of 10 C2PO constructs naming the construct | Unit | P0 | FR-086-AC-2 | 🚧 planned |
| TC-312 | Refuse profile mismatch and absent profile on C2PO import | Unit | P0 | FR-086-AC-3 | 🚧 planned |
| TC-313 | Import each FRET MLTL fixture to its declared graph or loss records | Integration | P0 | FR-087-AC-1 | 🚧 blocked on fixture retrieval (TL-431) |
| TC-314 | Report a bare next as AmbiguousNext | Unit | P0 | FR-087-AC-2 | 🚧 planned |
| TC-315 | Import each FRET bundle with variables bound and header fields preserved | Integration | P0 | FR-088-AC-1 | 🚧 blocked on fixture retrieval (TL-431) |
| TC-316 | Produce loss records for unmapped, duplicate, non-Boolean variables and unknown header fields | Unit | P0 | FR-088-AC-2 | 🚧 planned |
| TC-317 | Carry attribution and licence on every foreign example and refuse one missing a field | Unit | P0 | FR-089-AC-1, FR-089-AC-2, FR-089-AC-3 | 🚧 planned |
| TC-318 | Classify the 11-formula (12 class assignments) mission report population at M = 5 | Unit | P0 | FR-090-AC-1 | 🚧 planned |
| TC-319 | Report mission-time-absent without a claim and refuse overflowing horizon | Unit | P1 | FR-090-AC-2 | 🚧 planned |
| TC-320 | Produce each of 10 loss kinds from its fixture | Unit | P0 | FR-091-AC-1, StR-044-VC-2 | 🚧 planned |
| TC-321 | Check imported plus unimported equals declared requirement count | Property | P0 | FR-091-AC-2, StR-044-VC-2 | 🚧 planned |
| TC-322 | Round-trip the 10 tl-rewrite WEST cases and compare graphs | Integration | P0 | FR-092-AC-1 | 🚧 planned |
| TC-323 | Round-trip all 20 WEST lines byte for byte | Integration | P0 | FR-092-AC-2 | 🚧 planned |
| TC-324 | Refuse the five unexportable WEST classes with loss records | Unit | P0 | FR-092-AC-3 | 🚧 planned |
| TC-325 | Verify retained WEST file list, licence presence and byte equality with tl-rewrite's copy | Integration | P1 | FR-092-AC-4 | 🚧 planned |
| TC-326 | Inspect that the ledger covers every ticket criterion and ADR acceptance and review are recorded | Inspection | P1 | FR-093-AC-1, FR-093-AC-2 | 🚧 blocked on human acceptance of the ADRs |
| TC-327 | Refuse unbounded to bounded conversion on every importer and the closure API, and leave export_safety_monitor unchanged | Unit | P0 | FR-094-AC-1 | 🚧 planned |
| TC-328 | Compute the known-case table over 29,920 comparisons (including Xs and Xw) against an independent enumerator | Property | P0 | FR-094-AC-2 | 🚧 planned |
| TC-329 | Return the closure report rows for F, G, U, R, Xs and Xw, not-computed rows for infinite-trace and no bounded formula | Unit | P0 | FR-094-AC-3 | 🚧 planned |
| TC-330 | Check the 24 hostile reader cases against the manifest | Integration | P0 | NFR-047-AC-1 | 🚧 planned |
| TC-331 | Build the 3 reader fuzz targets and replay seeds | Fuzz | P1 | NFR-047-AC-2 | 🚧 planned |
| TC-332 | Import the other requirements and return one loss record for an expression-level unsupported construct, and abort the file on a file-level one | Unit | P0 | FR-086-AC-4 | 🚧 planned |
| TC-333 | Refuse a missing or mismatched ClosurePolicy and return the bounded formula with provenance for a matching one (withdrawn if the conversion is not built) | Unit | P1 | FR-094-AC-4 | 🚧 blocked on Open decision (closure conversion built) |
