---
id: SR-073
title: TL-218 ears-conformance review
type: SpecReview
analysis: ears-conformance
scope: agent-ix/tl-mltl@c4cdfc56feb6d8a7b0e2e784b6b301d72ca71db5; spec/requirements/FR-040-infinite-safety-export.md,
  spec/requirements/FR-041-c2po-refusal-partition.md
review_set: subset
---

## Summary

EARS review of the two new functional requirement statements; targeted Quire grammar check was clean, with one semantic modal defect. Ticket: TL-218. Provisional base: 4bcea84bbe3fa7360076ea51edaf7f23a094500f.

## Verdict

**CONDITIONAL** — Substantive finding(s) require a fix.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-040 uses “may export” for the feature’s trigger and response while FR-040-AC-1 says every admitted export has a defined result; the normative obligation is optional in the statement and cannot establish when the provider shall export or refuse. | spec/requirements/FR-040-infinite-safety-export.md:16 |

## Coverage

Examined: spec/requirements/FR-040-infinite-safety-export.md, spec/requirements/FR-041-c2po-refusal-partition.md. Targeted Quire validation was 3/3 grammar-clean; no aggregate gate was run.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 4643c76d4d46730a5de9f50f47ebfc178b8f9e0d — FR-040 now says that, when export is requested, the provider shall export an admitted graph or return a typed pre-output refusal. |

## Disposition verdict

**PASS** — normative trigger and response are explicit on this head.
