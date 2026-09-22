---
type: log
title: "PLAN-007 — Update Log"
description: "Chronological changes to the MLTL corpus campaign plan."
---

# PLAN-007 — Update Log

## History

* **2026-09-13** — Created from MRS-002, FR-008 through FR-010, NFR-004, MP-002, and TM-002; routed eight blocked tasks across tl-syntax, tl-parse, tl-rewrite, tl-mltl, and a quire-contract-ir #63/#64 producer boundary. First authored on branch `issue/38-corpus-interop-spec` (PR #44), which was never merged.
* **2026-09-21** — Re-derived against current `main` and renumbered. The abandoned PR #44 revision is reference material only: its `PLAN-006` identity is now held by `plan/PLAN-006-qobs-c00-consumer`, its `TC-048`..`TC-053`/`TC-056` and `SR-038`..`SR-052` identities collide with identities `main` has since allocated to other work, and its pinned source and coverage censuses predate TL-170, TL-171, TL-179 and TL-180. Plan identity is now `PLAN-007`; tasks are `Task-010` through `Task-016`, continuing after `PLAN-006`'s `Task-009`; test cases are `TC-091` through `TC-103`.
* **2026-09-21** — Dropped the native-predicate/native-temporal bridge lane (PR #44's `Task-023`, GitHub tl-mltl#55). Under the TL-175 architect ruling every TL-* crate stays independent of the agent-ix/Quire ecosystem, so `tl-mltl` cannot own a campaign lane whose producer is `quire-contract-ir`. That dimension is specified and routed in `quire-mltl`, the ruling's one deliberate bridge crate. Recorded as ADR-002. Seven TL-owned tasks remain.
