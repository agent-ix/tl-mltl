---
id: SR-059
title: "Independent risk-complexity review of the current corpus campaign"
type: SpecReview
analysis: risk-complexity
scope: "FR-008 through FR-010, NFR-004, PLAN-006"
review_set: all
---

## Summary

**PASS after remediation.** The highest risks are denominator drift, identity
ambiguity, hostile manifest resources, self-oracling, and external-state
overclaim. Each has a bounded contract, mutation control, and owner-routed task.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-5901 | high | An unencoded tuple and unsealed initial registry made cell identity and denominator retention technically indeterminate. Fixed with exact preimages and a pinned ordered set. | FR-008-AC-1, Task-001 |
| FND-5902 | high | Manifest nesting, sizes, counts, and platform paths crossed an untrusted filesystem boundary without stable maxima. Fixed with a closed limit profile, checked preflight, and hostile-boundary tests. | FR-009-AC-6, Task-007 |
| FND-5903 | high | External failures and identity defects could be normalized into a plausible comparison state. Fixed with pre-admission refusal and a closed valid-combination table. | FR-010-AC-1, FR-010-AC-3, Task-006 |
| FND-5904 | medium | Upstream past/native contracts remain volatile. Kept as successor rows and Task-008 so current future/W/M implementation does not depend on their shape. | MRS-002, Task-008 |

## Risk register

| Requirement | Technical risk | Volatility | Drivers | Mitigation |
|---|---|---|---|---|
| FR-008 | high | low | closed registry and cryptographic identity | exact preimages, required-class property checks, mutation-pinned set |
| FR-009 | high | medium | filesystem boundary and distributed family ownership | fixed caps, checked conversions, one owner, digest-pinned replay |
| FR-010 | medium | high | external profile and availability evolution | closed axes, successor profiles, no runtime qualification dependency |
| NFR-004 | medium | medium | measurement interpretation and tooling module drift | raw populations, byte reproduction, explicit limitation, Task-007 gate |

Top hazards are FND-5901 through FND-5903. No unresolved failure-domain gap is
hidden by the passing scoped verdict; the Quire module-pin limitation remains
explicit evidence context rather than a campaign pass claim.
