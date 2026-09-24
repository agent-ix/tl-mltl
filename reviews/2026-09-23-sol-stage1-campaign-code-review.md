---
id: SR-056
title: "SOL code review — mechanical Stage 1 campaign"
type: SpecReview
analysis: code-review
scope: "Draft PR 94 at 5219153; 117-member Stage 1 Campaign, config, source closure and checker"
review_set: subset
relationships:
  - target: "ix://agent-ix/tl-mltl/FR-055"
    type: "reviews"
---

# SR-056: SOL code review — mechanical Stage 1 campaign

## Summary

Reviewed the 117-member definition and authored procedure selectors, the source-closure data, and the TL config/checker paths. The V10 compile selectors choose tracked TL source inputs for the static cases and declared `V10.inputs` artifacts for generated cases. The V8 parse selector takes its executable from `V8.parse_example_prep`. The checker compares those selected digests to source or retained dependency bytes, and focused substitutions are refused.

## Verdict

**CONDITIONAL** for the reviewed code path; complete Campaign acceptance remains unmeasured.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Provenance substitution tests stop at helper functions; no full retained Campaign tamper replay checks the aggregate verdict | src/bin/tl_campaign_check.rs:1670 |

## Finding detail

### FND-001

The helper tests correctly reject changed selected digests. They do not run the TL checker from a Quoin-staged bundle, then alter the retained request/dependency evidence and verify that `quoin measurement campaign verify` rejects it. Add those two end-to-end cases for V10 generated input and V8 executable provenance before calling the fix a full Campaign gate.

## Coverage

The definition contains 117 distinct required members. Focused Rust 1.98.1 gate with `TMPDIR=/private/tmp`: checker and config binary tests passed 12/12. The full Linux `cave` Campaign run, package authentication and hosted CI were not run. No source code edits were made.
