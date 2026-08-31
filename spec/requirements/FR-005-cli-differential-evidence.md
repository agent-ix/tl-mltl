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

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-005-AC-1 | CLI evaluation and analysis match the library API and reject unknown schema identities. | Test (TC-014) |
| FR-005-AC-2 | Supported differential cases compare truth value and verdict time; unsupported/tool errors remain non-conclusive. | Test (TC-015) |
| FR-005-AC-3 | Retained reports and checksums identify every input, tool, output, limitation, and requirement reference. | Test (TC-016) |

## Dependencies

Packages FR-001 through FR-004 without changing their semantic results.
