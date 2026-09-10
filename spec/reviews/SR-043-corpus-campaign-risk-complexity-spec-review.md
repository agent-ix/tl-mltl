---
id: SR-043
title: "Risk-complexity review of the corpus campaign"
type: SpecReview
analysis: risk-complexity
scope: "FR-008 through FR-010 and NFR-004"
review_set: all
---

## Summary

**PASS after remediation.** The dominant hazards are denominator gaming,
self-oracling, profile churn, unsafe artifact resolution, and overclaiming an
external observation. Exact populations, isolated oracles, hard gates, safe
paths, and orthogonal states mitigate each before tasking.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-4301 | high | A ratio can improve when a failing or blocked cell silently disappears. Fixed with immutable raw strata, tombstones, successor manifests, and denominator mutation controls. | FR-008-AC-1, FR-009-AC-3, NFR-004-AC-2 |
| FND-4302 | high | Retained external agreement can be generalized beyond its eight exact cases or mistaken for target qualification. Fixed with exact identity/case bounds and three independent state axes. | FR-010-AC-1, FR-010-AC-2 |
| FND-4303 | high | Recursive manifests and hostile paths can escape the corpus root or exhaust resources. Fixed with regular tracked-file confinement, digest-before-decode, and hard size/count/depth caps. | FR-009-AC-6 |
| FND-4304 | medium | Pending upstream profiles are volatile and could force deep rework. Fixed by retaining them as blocked identity-bearing cells that enter only through successor manifests after landing. | MRS-002, FR-008-AC-4 |

## Risk register

| Requirement | Technical risk | Volatility | Driver | Mitigation |
|---|---|---|---|---|
| FR-008 | high | medium | closed cross-dimensional census and independent semantic oracle | schema/property/mutation controls; implement current profiles first |
| FR-009 | high | low | filesystem trust boundary, immutable identities, distributed ownership | bounded safe-path loader; one owner; exact digest contract |
| FR-010 | medium | high | external profile/runtime policy and mapping evolution | orthogonal states; output-only mapping; successor profiles |
| NFR-004 | medium | medium | measurement interpretation and retained human authority | raw populations before ratios; automated-claim zero gate |

The top hazards are FND-4301 through FND-4303. The failure-domain review SR-039
contains no open gap after remediation.
