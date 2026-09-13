---
id: SR-051
title: "Risk-complexity review of the verification-effectiveness campaign"
type: SpecReview
analysis: risk-complexity
scope: "FR-011 through FR-015 and NFR-005"
review_set: all
---

## Summary

**PASS after remediation.** The dominant hazards are denominator gaming,
correlated oracles, stochastic overclaiming, unstable mutation baselines,
bounded-proof widening, and premature shared intake. Each has an explicit
identity, refusal, non-claim, and admission control before tasking.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-5101 | high | Reclassification or omission could improve property, mutation, or proof results. Fixed with frozen complete populations, exclusion visibility, successor ledgers, and mutation probes. | FR-011, FR-013, FR-014, NFR-005-AC-2 |
| FND-5102 | high | A bounded primitive proof could be widened to callers or unbounded MLTL. Fixed with exact proposition/non-claims, finite domains, assumptions, call subject, and no aggregate proof ratio. | FR-014, MP-006 |
| FND-5103 | high | Tool output could enter assurance with missing identities or a false release profile. Fixed with exact wrapper bindings and capability-gated refusal. | FR-015-AC-3 through FR-015-AC-5 |
| FND-5104 | medium | Pending semantic profiles and native bridge contracts are volatile. Fixed by drafting identity-bearing blocked rows now while prohibiting implementation or credit before exact accepted revisions land. | MRS-003, FR-011 |

## Risk register

| Requirement | Technical risk | Volatility | Primary mitigation |
|---|---|---|---|
| FR-011 | high | medium | closed ledger, independent oracle control, bounded domains |
| FR-012 | high | medium | exact feature identities, dual caps, replay, bounded non-claims |
| FR-013 | high | medium | frozen populations, interleaved controls, conservative score |
| FR-014 | high | high | candidate digest, exact claims, cover/unwind controls |
| FR-015 | high | high | shared-capability admission gates and no local substitute |
| NFR-005 | medium | medium | mutation probes and raw populations before summaries |
