---
id: PLAN-050
title: LTLf evaluation and interchange plan
type: Plan
status: not_started
relationships:
  - target: ix://agent-ix/tl-mltl/FR-083
    type: references
  - target: ix://agent-ix/tl-mltl/FR-084
    type: references
  - target: ix://agent-ix/tl-mltl/FR-085
    type: references
  - target: ix://agent-ix/tl-mltl/FR-086
    type: references
  - target: ix://agent-ix/tl-mltl/FR-087
    type: references
  - target: ix://agent-ix/tl-mltl/FR-088
    type: references
  - target: ix://agent-ix/tl-mltl/FR-089
    type: references
  - target: ix://agent-ix/tl-mltl/FR-090
    type: references
  - target: ix://agent-ix/tl-mltl/FR-091
    type: references
  - target: ix://agent-ix/tl-mltl/FR-092
    type: references
  - target: ix://agent-ix/tl-mltl/FR-093
    type: references
  - target: ix://agent-ix/tl-mltl/FR-094
    type: references
  - target: ix://agent-ix/tl-mltl/NFR-047
    type: references
---

# PLAN-050: LTLf evaluation and interchange plan

## Objective

Land the tl-mltl side of TL-428 (LTLf evaluation and reference agreement), TL-431
(FRET MLTL and C2PO import, loss records, mission-bounding report), TL-432 (C2PO
import) and TL-433 (WEST import and export). No build task starts before the
ADRs 044 to 047 here and the tl-syntax and tl-parse ADRs are accepted by a human.

## Dependency DAG

Layer numbers match `ix-board project-chunk P-TL-151`: layer 0 TL-393 and TL-427;
layer 1 TL-394, TL-428, TL-431, TL-432, TL-433; layer 2 TL-395, TL-397, TL-429;
layer 3 TL-399, TL-434; layer 4 TL-404, TL-430; layer 5 TL-435. All tl-mltl tasks
sit in layer 1.

```text
pin     tl-syntax TL-428 change lands and is released -> tl-parse pin bump -> tl-mltl pin bump
          (Task-001 step one; see Pin sequencing below)
layer 0  Task-000 (FR-093 gate: ADRs accepted, spec-review recorded)
layer 1  TL-428  Task-001 LTLf evaluator (FR-083, FR-085)
                   -> Task-002 reference corpus replay (FR-084)
         TL-432  Task-003 C2PO import (FR-086)
         TL-433  Task-004 WEST import and export (FR-092)
         TL-431  Task-005 loss records, FRET MLTL import, attribution (FR-091, FR-087, FR-089)
                   -> Task-006 FRET C2PO, variable map, header (FR-088)   needs Task-003
                   -> Task-007 mission report and closure refusal (FR-090, FR-094)   needs Task-001, Task-005
         all      Task-008 reader bounds and fuzz (NFR-047)   needs Task-003, Task-004, Task-005
```

Pin sequencing. tl-mltl pins tl-syntax and tl-parse by git revision in `Cargo.toml`
(tl-syntax `9f7b1c8`, tl-parse `9607722` at the time of writing). The LTLf change adds
`NodeKind` variants and a fifth `SemanticProfile`, which breaks every exhaustive
match downstream (tl-syntax ADR-044 type shape, PROPOSED). Order: (1) the tl-syntax
change (TL-428) lands on tl-syntax main and is released; (2) tl-parse bumps its
tl-syntax pin and lands v5 (TL-429); (3) tl-mltl bumps both pins in one PR that adds
the explicit arms (typed "not in this profile" refusals, no `_` defaults) and changes
nothing else, as Task-001 step one, before any LTLf evaluator code; (4) tl-solve bumps
last. Until step (1) no tl-mltl task that needs the LTLf graphs can compile.

Discrepancy for the planner: FR-088 (TL-431) reuses the C2PO reader of FR-086
(TL-432), but ix-board shows TL-431 and TL-432 in the same layer with no edge.
Either add TL-432 as a blocker of TL-431 or let Task-006 build the shared
reader first and TL-432 consume it; this plan assumes the former and flags it.

## Task File Mapping

| Task | Ticket | Scope | Exit evidence |
|---|---|---|---|
| Task-000 | TL-427 | FR-093 | TC-326 |
| Task-001 | TL-428 | FR-083, FR-085 | TC-300 to TC-303, TC-309 |
| Task-002 | TL-428 | FR-084 | TC-304 to TC-307 |
| Task-003 | TL-432 | FR-086 | TC-310 to TC-312, TC-332 |
| Task-004 | TL-433 | FR-092 | TC-322 to TC-325 |
| Task-005 | TL-431 | FR-091, FR-087, FR-089 | TC-313, TC-314, TC-317, TC-320, TC-321 |
| Task-006 | TL-431 | FR-088 | TC-315, TC-316 |
| Task-007 | TL-431 | FR-090, FR-094 | TC-318, TC-319, TC-327 to TC-329, TC-333 (only if the closure conversion is built) |
| Task-008 | TL-431/432/433 | NFR-047 | TC-330, TC-331 |

## Open decisions (Peter)

Left open on purpose. Each has a recommendation; none is decided by this specification.

| # | Decision | Options | Recommendation | Where |
|---|---|---|---|---|
| 1 | Mission-bound closure | build the conversion and `ClosurePolicy`, or refusal and report only | refusal and report only (TC-333 withdrawn) | ADR-045, FR-094 |
| 2 | WEST corpus | tl-mltl copies, or the corpus moves to one shared home | copy | ADR-046, FR-092 |
| 3 | Importers | live in tl-mltl, or in tl-parse or a new crate | keep in tl-mltl | ADR-046 |

### Proposed tracker change (not made)

Add the Linear blocker edge **TL-432 blocks TL-431**. FR-088 (TL-431, Task-006) reuses
the C2PO reader of FR-086 (TL-432, Task-003), and `ix-board chain TL-431` shows only
TL-427 as a blocker, with TL-431 and TL-432 both in layer 1. This is a Linear change,
so this specification does not make it; the plan assumes the edge. The alternative
(Task-006 builds the shared reader first and TL-432 consumes it) contradicts TL-432's
ownership of the reader.

## Quality gates

- Gate A after Task-001: the closed-trace and prefix evaluators are untouched
  (existing corpora byte-identical) before any importer lands.
- Gate B before Task-007: the independent known-case enumerator exists and agrees
  with the LTLf evaluator on its own cases.

## Deferred verification

| Item | Why deferred | Needed to run |
|---|---|---|
| Fresh generation of the LTLf reference corpus (TC-308; blocks TC-304, TC-305) | no outside LTLf tool may be run during specification | install the ADR-044 reference at a recorded version, run the documented procedure, retain with provenance |
| TL-430 population checks (tl-solve FR-300-AC-1, FR-302-AC-1) | they read the DEFERRED FR-084 corpus, so TL-430 depends on TL-428 corpus generation | the corpus row above |
| Upstream revision of the retained WEST corpus (FR-089) | tl-rewrite's manifest records none | whoever next re-retrieves the file |
| Retrieval of published FRET examples (blocks TC-313, TC-315) | no FRET export was available offline; layouts ungrounded | fetch examples with licence and revision into the fixture manifest |
| Live R2U2/C2PO re-check of imported C2PO | no outside tool run | existing `make r2u2-nightly` pin |
| `make ci`, fuzz campaigns, mutation | no build capacity | repository gates |
| Human acceptance of ADR-044 to ADR-047 | only a human accepts | Peter |
| UNBUILT (TL-416, P-TL-149, no PLAN of its own): FR-043-AC-1..7 (TC-188..TC-193), `src/requirement_realizability.rs`, `tests/requirement_realizability.rs` | written without cargo/rustc; nothing compiled or run | `cargo test --test requirement_realizability`, then `-- --ignored` for the full 126,960-case population (440,951,040 plays; expected R 44566, unrealizable 82394, of which 768 pass the relaxation) |
| UNBUILT-PIN: `tl_syntax::SignalPartition` with guessed `inputs()`/`outputs()` accessors and a guessed `SignalPartition::new(inputs, outputs)` test constructor | tl-syntax FR-034 is not merged; Cargo.toml pins tl-syntax `a80d09e` (origin/main) | replace the pin with the merged FR-034 rev, fix the two adapters, refresh Cargo.lock |
| UNBUILT (TL-393, P-TL-147): FR-113-AC-1/2 (TC-257, TC-258), `tests/solver_independence.rs` | not run; TC-257 reads `Cargo.lock` and `fuzz/Cargo.lock` instead of `cargo tree` (guess that the lock is a superset of every feature view) | `cargo test --test solver_independence` and one manual `cargo tree -e all --all-features` cross-check |
| UNBUILT (TL-428): FR-083-AC-1..4, FR-084-AC-4, FR-085-AC-1/2 (TC-300..TC-303, TC-307, TC-309); `src/ltlf.rs`, `HorizonError::UnboundedByTraceLength`, explicit LTLf arms in every exhaustive `NodeKind`/`SemanticProfile` match of `src/` and `tests/future_parity.rs`; `tests/ltlf_evaluation.rs` | written without cargo/rustc; nothing compiled or run | `cargo test --test ltlf_evaluation`, `cargo test --features infinite-trace --test ltlf_evaluation` (Disposition arm), full suite for the new match arms |
| UNBUILT-PIN: `NodeKind::{LtlfFuture, LtlfGlobally, LtlfUntil, LtlfRelease, StrongNext, WeakNext}` and `SemanticProfile::LtlfFiniteTraceV1` (tl-syntax ADR-044 PROPOSED shape) | tl-syntax TL-428 change not merged; pin sequencing step (3) of this plan is pre-empted by unbuilt code | bump tl-syntax (and tl-parse) to the merged revs; if ADR-044 picks a separate LTLf graph type, rename in `src/ltlf.rs` only |
| Fuzz crate `fuzz/fuzz_targets/evaluator_differential.rs` exhaustive `NodeKind` matches have no LTLf arms | fuzz crate pins tl-syntax `v0.4.0` separately and is not bumped here | add arms when the fuzz pin moves |
| TC-300 uses hand-written rule cases, not the tl-syntax `ltlf` corpus (FR-335) | corpus not merged | read `tl_syntax::CORPUS_DIR/ltlf` |
| TC-306 (corpus provenance/tamper refusal) not written | the corpus format needs an "input digest" and a rows digest; decision 7 (2026-11-23) forbids new digests (SPEC-GAP) and no corpus exists | decide the integrity mechanism (byte comparison or a registered digest) with the corpus |
| UNBUILT (TL-432): FR-086-AC-1..4 (TC-310, TC-311, TC-312, TC-332); `src/interop/{mod,loss,c2po}.rs`, `tests/c2po_import.rs` | written without cargo/rustc; nothing compiled or run | `cargo test --test c2po_import` |
| C2PO construct list and keyword spellings (`CONTRACT`, `ORDER`, `foreach`..`foratmost`, `Y` as a prefix operator) not checked against the C2PO 4.1.0 grammar (SPEC-GAP, UNGROUNDED) | no grammar retained; nothing may run C2PO | reconcile with the pinned C2PO checkout; split, never drop, entries |
| UNBUILT (TL-431): FR-087-AC-2, FR-088-AC-2, FR-089-AC-1/3, FR-090-AC-1/2, FR-091-AC-1/2, FR-094-AC-1..3 (TC-314, TC-316, TC-317, TC-318..TC-321, TC-327..TC-329); `src/interop/{attribution,fret,mission}.rs`, unbounded-operator refusal added to the C2PO reader; `tests/fret_import.rs`, `tests/mission_closure.rs` | written without cargo/rustc; nothing compiled or run | `cargo test --test fret_import --test mission_closure`, plus `--features infinite-trace` for the infinite-trace closure row |
| TC-313, TC-315 (published FRET examples) not written | no FRET export retained; retrieval DEFERRED (row above); the FRET MLTL, variable-map (`variable:signal-id`) and header (`# a,b`, `# key: value`) spellings are provisional (UNGROUNDED) | retrieve fixtures with licence and revision, rewrite the reader against their bytes, add the manifest census test |
| TC-333 withdrawn | decision 4 (2026-11-23): refusal and report only; `request_mission_closure` always refuses, no `ClosurePolicy` | none unless the decision changes |
| FR-090 report SHA-256 identity omitted (SPEC-GAP) | decision 7 forbids digests outside the ADR-019 / tl-solve ADR-004 register | owner decides whether to register it |
| TC-319 horizon-overflow refusal not exercised | an overflowing `u64` horizon needs ~2^32 nested `u32`-bounded nodes | inspection of the error mapping |
| UNBUILT-PIN: `InfiniteFormulaDocument::validate()` returning `InfiniteFormula` (TC-329 infinite row) | guessed tl-syntax API | confirm against tl-syntax |
| UNBUILT (TL-433): FR-092-AC-1..4, FR-089-AC-2, NFR-047-AC-1 (TC-322..TC-325, TC-330, WEST part of TC-327); `src/interop/west.rs`, `corpus/west-v1/{formulas_d1.txt,LICENSE,manifest.json}` (first two copied byte-identically from tl-rewrite origin/main `7a907d9`), `tests/west_interop.rs`; reader nesting now counts parentheses and prefix operators | written without cargo/rustc; nothing compiled or run | `cargo test --test west_interop`; TC-325's byte comparison needs `../tl-rewrite` or `TL_REWRITE_WEST_DIR`, otherwise it prints that it did not run (SPEC-GAP) |
| TC-331 / NFR-047-AC-2 (fuzz targets `c2po_import`, `fret_mltl_import`, `west_import`) not written | new fuzz targets need campaign members (CLAUDE.md inventory rule) and a fuzz build; no build capacity | add the three targets, seeds and campaign members together |
| FR-092-AC-3 fifth class (`W`/`M`) has no graph node, so no export test (SPEC-GAP) | tl-syntax has no W/M node kind | spec amendment: drop the class or define it on import |
| UNBUILT-PIN (review fixes): `[patch."https://github.com/agent-ix/tl-syntax.git"]` points both `Cargo.toml` and `fuzz/Cargo.toml` at `../TL-code-tl-syntax`; `Cargo.lock` and `fuzz/Cargo.lock` are stale | no cargo may run; tl-syntax not merged. Unverified: whether the patch also redirects tl-oracle's own tl-syntax pin (a different rev), which could break tl-oracle if it matches `NodeKind` exhaustively | merge tl-syntax, pin its rev in both manifests, drop the patch, refresh both locks, build tl-oracle against it |
| FR-043 contract assumptions (review H5; decision 1): `play_out`/`judge_strategy` take `RealizabilityContract`s; tests tagged proposed TC-194 / FR-043-AC-8 | no AC or TC exists in TM-007 | add FR-043-AC-8 and TC-194 (proposed text in the coder report) and amend ADR-004 decision 5 |
| FR-043 play counts: the partition comes from `partition_all`, which lists only occurring propositions, so plays are `2^(\|occurring\| * L)`, not TM-100's `16^L`; verdict counts (126960, 44566, 82394, 768) are unaffected because a non-occurring signal cannot change a verdict | TM-100 play totals (254,300,160 + 186,650,880) assume all four signals per play | amend TM-100/TM-007 play-total text (proposed in the coder report) |
| C2PO reader refuses chained or mixed unparenthesized binary operators (`a -> b -> c`, `a && b U[0,1] c`) as `UnrecognizedToken` | C2PO precedence and associativity are not retained (SPEC-GAP added to FR-086) | pin the grammar, then admit the forms it defines |
| TC-325 byte comparison: looks up `TL_REWRITE_WEST_DIR`, then `../TL-code-tl-rewrite`, `../tl-rewrite`, `../../tl-rewrite`; fails when none exists unless `TL_REWRITE_WEST_SKIP=1` | a clean checkout has no tl-rewrite | set `TL_REWRITE_WEST_DIR` in any gate that runs it |
| TOML dependency lines over 100 columns (`Cargo.toml`, `fuzz/Cargo.toml`) | TOML inline tables cannot span lines | none |

## Ungrounded items

- FRET export file layouts and operator spellings (FR-087, FR-088).
- The C2PO unsupported-construct list against the C2PO grammar (tool 4.1.0 with R2U2 4.2-release, FR-086).
- Identity and versions of the LTLf reference tools (ADR-044).
- The literature's own named finite-versus-infinite counterexample (ADR-044 item 3).
