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

## New findings (disposition pass 1)

Reviewed at `58ff72e181521e2a3600359bcc9f613b5b57f5fd`.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | PR #101's updated description says the current source graph cannot run because parse/rewrite main lacks three targets. Those assets exist at the exact parse and rewrite PR #52 heads pinned by the historical closure; the actual current-candidate blocker is incompatible Cargo source locks (parse/rewrite 0.4 and syntax 6e2/oracle 9bf versus mlTL 0.3 and syntax 6aa/oracle 239), stale control/measurement pins, and absent full Campaign evidence. Correct the PR description so the next source-graph decision uses the real dependency constraint. | PR #101 body; campaign/source-closure.json:5; Cargo.toml:31 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | still-open | The definition and closure still pin historical mlTL 29cea00/syntax 6e2/oracle 9bf, and no 120-member receipt exists for candidate 58ff72e. Parse/rewrite PR #52 heads contain the named targets, but their locked dependencies are incompatible with this candidate. |
| FND-002 | still-open | PR #101 description still presents missing-on-main assets as the reason the graph cannot be rebound; it omits the actual lock incompatibility and the pinned PR-head assets. |

## Disposition verdict — round 1

**FAIL** — FND-001 and FND-002 remain open. The MLTL-only imported targets and migrated plans improve readiness, but they do not satisfy TL-246 full Campaign acceptance.

## Dispositions — round 2

Reviewed at `9c68ef0e33b42726ace1ca420d20fb2a8f4b60d4`. The only changed product prose is `campaign/README.md`; the PR description now states the same corrected source-graph constraint. CampaignDefinition and closure pins are unchanged, and no complete Campaign receipt was added.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | still-open | The historical closure still pins measured mlTL 29cea00, syntax 6e2fc17 and oracle 9bf3994; candidate 9c68ef0 and its 0.3 dependency graph have no coherent repin or full 120-member EA/Quoin receipt. |
| FND-002 | fixed | 9c68ef0e33b42726ace1ca420d20fb2a8f4b60d4 — `campaign/README.md` and PR #101 now explicitly identify the pinned parse/rewrite PR-head assets and historical 0.4 lock incompatibility with candidate mlTL 0.3/syntax 6aa9/oracle 2391. |

## Disposition verdict — round 2

**FAIL** — only high FND-001 remains open. FND-002's source-graph explanation is corrected. GitHub reports PR #101 mergeable, but TL-246's full Campaign acceptance gate remains unproved.

## Dispositions — round 3

Reviewed at `09951830acf7ba0d6fa10845e8cf61ff5fe0dc50`. The published definition and closure remain historical. The README and latest Linear comment distinguish a clean scratch preflight, unpublished parse/rewrite baseline harness commits, and a Mac-only missing Linux Python executable from measured Campaign evidence.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | still-open | No coherent current-candidate published source closure or full 120-member EA/Quoin receipt exists. Local-only parse 02e167e/rewrite ac48980 baseline harness commits pass smoke and strict source/lock/equalHarness preflight, but are unpublished and unmeasured. The later Darwin refusal of `/usr/bin/python3.13` is an environment limit, not a Campaign verdict. |

## Disposition verdict — round 3

**FAIL** — high FND-001 remains open at `09951830acf7ba0d6fa10845e8cf61ff5fe0dc50`. The 120-plan parser fix does not establish Campaign execution or acceptance.

## Dispositions — round 4

Reviewed at `5143f02d8eeb614fb94a163cc96eca107758bf28`. The published CampaignDefinition and source closure are unchanged. The code fix and copied review files add no measured run, and the PR body continues to disclose the local-only scratch baseline trials and host-specific Python limitation.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | still-open | Candidate 5143f02 still has historical definition/closure pins and no full current-source 120-member EA/Quoin receipt. The local scratch preflight and smoke results remain prerequisites, not Campaign acceptance evidence. |

## Disposition verdict — round 4

**FAIL** — high FND-001 remains open. GitHub reports mergeable mechanics, but TL-246 acceptance remains unproved.

## Dispositions — round 5

Reviewed at control `91f826c7e9aae3881ba38c3e573a0c5810a13fe2` and clean measured `dfa3522d597bc1882128cefe1fe52b76b4610d4d`. The source closure and definition now bind the current 0.3 graph with nine exact aliases and reviewed V9 baseline refs. Independent static checks reconciled eight locally available Git trees, their Cargo manifests/locks and nested TL dependencies, all 12 equal-harness file controls, 120 plans/procedures/checker versions and 244 control/measured file bytes. The partial Darwin preflight stopped at missing authored cargo-fuzz 0.13.2 (host has 0.13.1), before a complete machine config or EA/Quoin run. No Campaign receipt or paired V9 measurement for this graph exists.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | still-open | The source repin portion is fixed by control 91f826c and measured dfa3522, but TL-246's acceptance criterion still lacks a full 120-member EA/Quoin Campaign receipt and independent replay. The Darwin tool refusal is an honest preflight stop, not a measured member outcome. |

## Disposition verdict — round 5

**FAIL** for TL-246 acceptance: FND-001 remains open solely on complete current-source Campaign execution and evidence reconciliation. GitHub reports PR #101 mergeable mechanically.
