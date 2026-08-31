---
id: PLAN-001
title: tl-mltl v0.1 implementation plan
type: Plan
status: active
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
  -> requirements, matrix, assurance packet, composite review
  -> closed evaluator
  -> checked horizon/delay/buffer analysis
  -> pending prefix evaluator
  -> R2U2/C2PO mapping
  -> CLI + differential corpus
  -> retained conformance evidence + human review
```

## Work Packages

1. Validate the specification, matrix, composite review, and assurance packet.
2. Implement pure closed evaluation with explicit limits and identities.
3. Implement checked horizon analysis and pending prefix semantics.
4. Implement deterministic mapping, CLI schemas, and differential comparison.
5. Pin the shared corpus, run requirement-tagged tests, and retain reports.
6. Perform code and gap reviews, then present the exact source candidate to the
   human release authority without publishing a crate.

## Exit Criteria

All matrix rows are backed, all local and remote gates pass, retained evidence
validates under merged PGM-01, unsupported external-tool cases remain explicit,
and the human source-release claim remains open pending independent approval.
