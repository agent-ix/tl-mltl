---
id: PLAN-020
title: Numeric evaluation, C2PO export and contracts plan (tl-mltl)
type: Plan
status: draft
relationships:
  - target: ix://agent-ix/tl-mltl/FR-053
    type: references
  - target: ix://agent-ix/tl-mltl/FR-054
    type: references
  - target: ix://agent-ix/tl-mltl/FR-055
    type: references
  - target: ix://agent-ix/tl-mltl/FR-056
    type: references
  - target: ix://agent-ix/tl-mltl/FR-057
    type: references
  - target: ix://agent-ix/tl-mltl/FR-058
    type: references
  - target: ix://agent-ix/tl-mltl/FR-059
    type: references
  - target: ix://agent-ix/tl-mltl/FR-060
    type: references
  - target: ix://agent-ix/tl-mltl/NFR-017
    type: references
---

# Implementation Plan: numeric evaluation, C2PO export and contracts (tl-mltl)

## Requirements Summary

- [ ] **FR-053, FR-054, FR-055, NFR-017** (TL-422): trace document, exact evaluation, missing rules.
- [ ] **FR-056, FR-057, FR-058** (TL-423): numeric export, admission rule, retained differential.
- [ ] **FR-059, FR-060** (TL-425): contract evaluation and export.
- [ ] **FR-054-AC-4** (TL-424): aggregation equivalence run.

## Dependency Graph

Layers follow `ix-board project-chunk P-TL-150`:

| Layer | Tickets | tl-mltl tasks |
|---|---|---|
| 0 | TL-419 | this specification |
| 1 | TL-420, TL-425 | upstream tl-syntax model and contract set (inputs) |
| 2 | TL-422 | Task-001, Task-002, Task-003 |
| 3 | TL-423, TL-424 | Task-004, Task-005; the equivalence run of FR-054-AC-4 needs tl-syntax aggregation |
| 4 | TL-426 | tl-solve re-evaluates witnesses through Task-001/002 |

- `Task-001 -> Task-002 -> Task-003` and `Task-002 -> Task-004 -> Task-005`.
- Task-003 may start once the tl-syntax contract set exists (layer 1), independent of Task-002 beyond the evaluator hook.

## Quality Gates

**Gate 1**: legacy trace and mapping bytes unchanged (TC-1572, TC-1582).
**Gate 2**: reduction-preserving results on numeric-free traces (TC-1575).
**Gate 3**: no R2U2 or C2PO execution anywhere in ci (TC-1591) and campaign inventory passes.

## Execution Tracks

- **Track A**: Task-001 -> Task-002 -> Task-004 -> Task-005.
- **Track B**: Task-003 (contracts), joining Track A at Task-005.

## Test Plan

`spec/numeric-test-matrix.md` is authoritative; no ids are added here.

## Deferred verification

| Item | Why deferred | Evidence needed to close |
|---|---|---|
| Fresh R2U2/C2PO output for the 216-formula numeric and 6-contract populations (TC-1589, TC-1590, TC-1592, TC-1598, TC-1601, TC-1602); TL-423 AC1 and TL-425 AC2 | BLOCKED: nothing in this repository executes R2U2 or C2PO, no R2U2 checkout was available, and no task in this plan can produce the exchange. The only deliverable toward these ACs is the refusal-until-pinned path (Task-004, TC-1587) | A human runs `make r2u2-numeric-live` locally against the pinned revision, pins the numeric profile from that run, and retains the exchange under `corpus/r2u2-numeric-v1/` |
| Pinned target numeric profile (integer width, float width, tolerance, division behaviour) | Ungrounded in retained evidence | Profile recorded in the retained manifest with evidence |
| Native C2PO contract syntax and R2U2 contract output encoding | Not retained (UNGROUNDED) | Pinned from the R2U2 checkout at the recorded revision |
| C2PO numeric input grammar (typed INPUT, arithmetic, abs, comparison), float tolerance and rounding behaviour | Not retained (UNGROUNDED); the corpus has only bool INPUT and FTSPEC/PTSPEC | The same retained exchange |
| tl-oracle numeric extension | Out of scope; composition used instead | Not required |
| Build, test and `make guarded-ci` | No build capacity | Gate at implementation head |
