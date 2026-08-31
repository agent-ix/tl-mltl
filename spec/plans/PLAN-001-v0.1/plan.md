---
id: PLAN-001
title: tl-mltl v0.1 implementation plan
type: Plan
relationships:
  - target: ix://agent-ix/tl-mltl/FR-001
    type: references
  - target: ix://agent-ix/tl-mltl/FR-005
    type: references
---

# tl-mltl v0.1 implementation plan

## Dependency DAG

```text
PGM-01 + exact tl-syntax revision
  -> specification and assurance foundation
  -> closed and caller-time evaluation semantics
  -> checked horizon and prefix semantics
  -> R2U2/C2PO mapping and differential corpus
  -> review remediation and complete local gates
  -> exact-candidate retained evidence
  -> human v0.1 source-release decision
```

## Task File Mapping

| Task | Scope | Exit evidence |
|---|---|---|
| Task-001 | Specification and assurance foundation | Validated requirements, matrix, reviews, and assurance packet |
| Task-002 | Reference semantics | Requirement-tagged Boolean and temporal tests |
| Task-003 | Horizon and prefix behavior | Checked resource and pending-decision tests |
| Task-004 | Mapping and differential interoperability | Exact tool identities and retained formula/time comparisons |
| Task-005 | Verification and review remediation | Complete local gate and resolved actionable review findings |
| Task-006 | Exact-candidate evidence | Sealed PGM-01 validations and checksummed retained record |
| Task-007 | Human source-release decision | Maintainer review and explicit release decision |

## Exit Criteria

All matrix rows are backed by executable or retained inspection evidence, the
complete CI gate passes, no blocking gap remains, and the Assurance Argument
stays open until a human release owner records the source-release decision.
