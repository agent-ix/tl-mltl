---
id: SR-067
title: "Independent risk-complexity review of the M5 verification campaign"
type: SpecReview
analysis: risk-complexity
scope: "FR-011 through FR-015, NFR-005, PLAN-004"
review_set: all
---

## Summary

**PASS after remediation.** The largest risks are identity/denominator drift,
stochastic plateau overclaim, unstable mutation execution, proof widening, and
false shared-intake claims. Each now has a closed input population, exact
identity, bounded outcome, and tracked gate.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-6701 | high | Hash prose without byte preimages made all four domain records volatile at implementation time. Fixed with exact arrays, domains, orders, and external digest carriage. | FR-011 through FR-014 |
| FND-6702 | high | Fuzz time/input windows and feature counts could overstate effectiveness. Fixed with dual anchors, exact features, monotonic time, and non-conclusive short runs. | FR-012, MP-004 |
| FND-6703 | high | Mutation equivalence and duplicate/risk dispositions could conceal unresolved survivors. Fixed with raw-outcome proof routing, acyclic duplicate roots, expiry, and immutable original outcomes. | FR-013, MP-005 |
| FND-6704 | high | Shared binary retention/build labels remain externally volatile. Kept behind open Quoin #363/#364 with domain work and unsupported states separable. | FR-015, PLAN-004 |

## Risk register

| Requirement | Technical risk | Volatility | Primary mitigation |
|---|---|---|---|
| FR-011 | high | medium | complete criterion census, exact ledger identity, discriminating oracle |
| FR-012 | high | medium | dual caps, cadence-valid feature identities, replay |
| FR-013 | high | medium | frozen population, isolated batches, non-circular routing |
| FR-014 | high | high | finite source registry, exact proposition, cover/unwind/non-claim checks |
| FR-015 | high | high | FR-009 resource boundary and shared capability gates |
| NFR-005 | medium | medium | raw populations and mutation controls before summaries |
