---
id: SR-077
title: TL-246 code and Rust review
type: SpecReview
analysis: code-review
scope: "agent-ix/tl-mltl@6de52325ebecdebf2e71692f728cd4f43e3c6285; Cargo.toml, src/bin/tl_campaign_check.rs, src/bin/tl_campaign_check/v10_replay.rs, src/bin/tl_campaign_check/v10_static.rs, src/bin/tl_campaign_config.rs, campaign/procedures/v1-independent-oracle.json, campaign/source-closure.json, campaign/stage1-campaign-definition.json"
review_set: subset
---

## Summary

Ticket: TL-246. Reviewed the source identity, sealed result and dependency binding, V10 semantic replay, oracle dependency seam, and the config source closure against current main. The frozen candidate is a prequalification implementation.

## Verdict

PASS for the inspected code paths. The known source pin and execution gaps are recorded in the separate gap review. Full guarded CI and a real 120-member Campaign were not run in this review.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Inspected Cargo.toml, the checker and V10 replay modules, the config generator, one generated procedure, source closure, and the CampaignDefinition. Checked wrong-source and duplicate-alias tests and V10's current mapper partition. Focused checker/config Rust tests were run separately.

## Disposition pass 1 recheck

Reviewed `58ff72e181521e2a3600359bcc9f613b5b57f5fd`. The prior placeholder records no actual finding. Added oracle, finite-partition, lasso and benchmark targets use production APIs and independently selected oracle expectations; the previously reviewed checker and config paths are unchanged. `cargo bench --locked --offline --features infinite-trace --bench v9_workloads -- --test` passed all 21 named smoke cases with digest checks. No new code finding from the inspected additions. Full Campaign execution remains in SR-078.

## New findings (disposition pass 3)

Reviewed at `09951830acf7ba0d6fa10845e8cf61ff5fe0dc50`. The changed `validate_plan` parser and its 120-plan and wrong-path tests passed (11 config tests total).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | `validate_plan` claims to bind a MeasurementPlan's frontmatter `definition_version`, but searches all lines of the file. A plan with a wrong or missing YAML version and the expected `definition_version: ...` line added in its body passes this preflight, so the version check is not tied to the authoritative frontmatter. The new tests mutate only the procedure path and do not cover this case. | src/bin/tl_campaign_config.rs:549 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-002 | still-open | Round 3 new finding: config still searches all lines for definition_version instead of reading the plan frontmatter. Quire validation is a separate gate and does not make this preflight check accurate. |

## Disposition verdict — round 3

**CONDITIONAL** for inspected code at `09951830acf7ba0d6fa10845e8cf61ff5fe0dc50`: one medium plan-version binding defect remains. The prior FND-001 is a no-finding placeholder, so it needs no disposition.
