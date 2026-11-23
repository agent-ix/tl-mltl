---
id: FR-005
title: Produce deterministic CLI and differential evidence
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-001
    type: implements
---

# FR-005: Produce deterministic CLI and differential evidence

## Description

A thin CLI shall evaluate formula/trace JSON, analyze horizons, emit adapter
manifests, and compare identified external-monitor results using versioned JSON.

## Behavior

- Machine output is deterministic for identical input bytes and excludes wall
  clock fields from semantic records.
- Differential comparison retains truth value, verdict time, unsupported state,
  tool error, and mismatch classification separately.
- Every mismatch is retained as a fixture or a documented profile difference.
- Human-readable summaries derive from the machine record.
- The `evaluation_record` example emits one versioned evaluation record per
  evaluate request, with trace, window and position read from the request file
  and the verdict taken from the evaluator.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-005-AC-1 | CLI evaluation and analysis match the library API and reject unknown schema identities. | integration-testing (TC-014) |
| FR-005-AC-2 | Supported differential cases compare truth value and verdict time; unsupported/tool errors remain non-conclusive. | integration-testing (TC-015) |
| FR-005-AC-3 | Retained reports identify every input, tool, output, limitation, and requirement reference. | integration-testing (TC-016) |
| FR-005-AC-4 | The evaluation-record example reads one command file and writes one `tl-mltl.evaluation-record/v1` JSON record carrying the trace and window read from that file, the evaluation position, the verdict (`True`, `False` or `Pending`), the witness when reported, the toolchain version and the semantic profile; the checked-in record schema lists exactly the fields the record carries, so removing or renaming a field fails a test. | integration-testing (TC-187) |

## Dependencies

Packages FR-001 through FR-004 without changing their semantic results.
