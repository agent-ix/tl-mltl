---
id: SR-078
title: TL-246 acceptance and evidence gap review
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/tl-mltl@6de52325ebecdebf2e71692f728cd4f43e3c6285; campaign/source-closure.json, campaign/stage1-campaign-definition.json, campaign/README.md, Cargo.toml, spec/requirements/FR-043-independent-tl-oracle.md, spec/requirements/FR-054-reproducible-v1-report.md, spec/requirements/FR-055-verification-claim-boundary.md, spec/v1-verification-test-matrix.md, src/bin/tl_campaign_config.rs, src/bin/tl_campaign_check.rs, campaign/test_v1_campaign.py"
review_set: subset
---

## Summary

Ticket: TL-246. Compared the ticket's full Campaign acceptance criteria and FR-054/055 to the retained definition, exact source pins, trace matrix and executable tests. The repo-wide coverage census has preexisting unbacked rows; this review focuses on TL-246's acceptance.

## Verdict

FAIL for campaign acceptance. The implementation remains a draft until the source graph is repinned, the entire Campaign executes, and the receipts reconcile.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The retained CampaignDefinition and source closure pin historical tl-mltl 29cea002, syntax 6e2fc17, and oracle 9bf3994 while this candidate is 6de5232 and Cargo pins syntax 6aa9b11 and oracle 2391e5b. The config correctly rejects a current-main measurement, so no full 120-member V1–V11 run at this candidate exists and TL-246's first acceptance criterion is unproved. | campaign/source-closure.json:5; Cargo.toml:31; src/bin/tl_campaign_config.rs:361 |

## Coverage

The definition declares 120 members across V1–V11, including 51 V10 members; the nine source aliases include five live TL/R2U2 identities and baseline aliases. Manual acceptance-to-tests reconciliation inspected TC-175 and TC-195 through TC-200 tags, focused checker/config tests, the V10 306 admitted/1,044 refused partition, and the retained Campaign claims. No full guarded CI or complete Campaign was run. Optional per-criterion semantic review was not invoked.
