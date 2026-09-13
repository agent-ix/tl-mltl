---
type: log
title: "PLAN-005 update log"
description: "What changed in the derived future-operator parity plan, and when."
---

# PLAN-005 update log

## History

- **2026-09-12** - Opened the plan for `agent-ix/tl-mltl#47` after tl-syntax#40
  landed as main `8dc18eec`. Advanced the compiled tl-syntax pin from
  `26b801d4`; the delta adds the future-lowering module and relocates
  `MAX_FORMULA_DOCUMENT_NODES` with an unchanged re-export.
- **Matrix header aligned.** The installed TestMatrix archetype asserts a
  single `Status` column; the functional and stakeholder coverage tables were
  renamed from `Coverage Status` so `make spec` validates.
- **Prefix reference bounded.** Kleene prefix evaluation is exact over
  continuations only for negation-free operands. The open-prefix parity sweep
  is restricted to that class rather than inventing a second prefix semantics.
- **Coverage totals re-pinned to 93/93.** spec-artifacts-process 737987b
  (quire-rs#363) makes the suite registry reference-only, so the 8 SUITE rows
  left the totals; the suite bindings are now asserted directly, and the
  unpinned module revision is recorded as known drift in `assurance/pins.json`.
- **PR-time review (PR #49) fixes.** Closed-prefix parity now covers every
  operand shape. The evaluation-record horizon is checked for both operators
  under both profiles. Exact resource outcomes are asserted on the lowered and
  the direct construction. The no-derived-branch scan walks `src/` recursively.
  The Quire export must also name FR-016.
