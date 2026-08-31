---
id: StR-002
title: Support bounded monitor interoperability
type: StR
---

# StR-002: Support bounded monitor interoperability

## Stakeholder Need

Embedded and assurance consumers need checked resource estimates, explicit
pending behavior, and reproducible mappings to R2U2/C2PO without silently
changing unsupported syntax or inventing external-tool evidence.

## Rationale

Finite memory and propagation delay are selection constraints for embedded
monitors, while external syntax drift must be visible before execution.

## Validation Criteria

| ID | Criteria | Validation |
|---|---|---|
| StR-002-VC-1 | Resource and prefix APIs distinguish conclusive, pending, unsupported, and resource-failure outcomes. | Test |
| StR-002-VC-2 | Adapter and differential records pin formula, proposition, source, output, and external-tool identities. | Test |

## Stakeholders

Embedded monitor developers, R2U2/C2PO integrators, and assurance reviewers.

## Context and Assumptions

External monitor execution is optional and never simulated when unavailable.

## Traceability

This need is realized by FR-002 through FR-004 and verified by TM-001.
