---
id: SR-017
title: "Base review — tracked SpecReview identity traceability"
type: SpecReview
analysis: base
scope: "NFR-002, TM-001, TC-033, tests/shared_assurance.rs"
review_set: base
---

# Base review — tracked SpecReview identity traceability

## Summary

Tracked SpecReview identity uniqueness is a distinct governance invariant. This
review adds an atomic NFR-002 acceptance criterion and TC-033, then traces the
tracked-tree collision test solely to that criterion. The implementation decodes
the YAML frontmatter value, so quoted and plain spellings identify the same
review, and refuses an empty census rather than passing vacuously.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-1701 | medium | The collision test was traced to TC-024, FR-006-AC-7, and NFR-003-AC-1 even though those govern legacy evidence removal and producer-boundary serialization, not review identities. Fixed by NFR-002-AC-5 and TC-033. | NFR-002-AC-5, TC-033, tests/shared_assurance.rs |
| FND-1702 | low | A uniqueness control must enumerate the tracked review tree rather than an allow-list, so newly added review artifacts are included and every colliding path is reported. The existing test has those properties. | TC-033, tests/shared_assurance.rs |
| FND-1703 | medium | Reading only a literal `id: ` line lets quoted YAML identities evade semantic duplicate detection. Fixed by deserializing the frontmatter `id` as YAML and proving quoted/plain collision. | NFR-002-AC-5, TC-033, tests/shared_assurance.rs |
| FND-1704 | medium | An empty review census makes uniqueness pass without evaluating an artifact. Fixed by a non-empty census floor and a direct negative control. | NFR-002-AC-5, TC-033, tests/shared_assurance.rs |
