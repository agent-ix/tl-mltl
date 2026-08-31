---
id: FR-004
title: Emit versioned R2U2 and C2PO mappings
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-002
    type: implements
---

# FR-004: Emit versioned R2U2 and C2PO mappings

## Description

The adapter shall translate the supported bounded future-time subset into a
deterministic C2PO expression and a versioned mapping manifest without running
or impersonating an external monitor.

## Behavior

- Proposition identities map to stable `p<ID>` aliases.
- Every output records input/output SHA-256 digests, exact source revision,
  clean/modified source state, formula identity, profile, adapter version, and
  optional external-tool identity.
- Unsupported profiles, bounds, or constructs are reported before execution.
- The manifest makes no unmeasured timing or memory claim.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-004-AC-1 | Supported mappings are stable and preserve formula/proposition identities. | Test (TC-011) |
| FR-004-AC-2 | Unsupported mappings return a typed reason and no executable artifact. | Test (TC-012) |
| FR-004-AC-3 | Manifest digests and external-tool identities detect substitution. | Test (TC-013) |

## Dependencies

Depends on the tl-syntax wire identity and FR-002 resource report.
