---
id: SR-035
title: "Base specification review — contextual semantic formula identity"
type: SpecReview
analysis: base
scope: "agent-ix/tl-mltl#28; FR-007-AC-8; TC-034; StR-003; NFR-001; NFR-002"
review_set: base
relationships:
  - target: ix://agent-ix/tl-mltl/FR-007
    type: reviews
---

# SR-035: Base specification review — contextual semantic formula identity

## Summary

The base checklist reviewed the span-independent contextual identity increment
for identifier integrity, functional-requirement quality, cross-references, and
all six test-coverage rules. Four issues were corrected before
implementation: optional span presence is explicit, mapping identity holds its
separately supplied formula bytes fixed, and conversion from a larger borrowed
formula to the bounded canonical document has a typed refusal. Existing
TC-025 through TC-031 statuses now reflect their compiled Rust evidence.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-3501 | medium | “Different spans” did not explicitly cover the absent-versus-present boundary even though optional-span serialization is one way diagnostic provenance can leak into identity. | FR-007-AC-8, TC-034 |
| FND-3502 | medium | Mapping also binds exact formula bytes, so an unconditional equality claim across differently spelled source inputs would conflict with the existing content-identity contract. | FR-007 digest allocation, FR-007-AC-8, TC-034 |
| FND-3503 | medium | The shared semantic view is constructed through the bounded v1 FormulaDocument, but a borrowed Formula can exceed that node limit; an undocumented conversion failure would invite a panic or mislabeled error. | FR-007 inputs and shared context behavior |
| FND-3504 | medium | TC-025 through TC-031 remained marked planned after their requirement-tagged Rust evidence landed, obscuring the actual remaining FR-007 gap. | TM-001, TC-025 through TC-031 |

## Dispositions

| Finding | Disposition | Evidence |
|---|---|---|
| FND-3501 | **FIXED** | FR-007-AC-8 and TC-034 now cover absent, empty, and distinct valid diagnostic spans with every other operation input fixed. |
| FND-3502 | **FIXED** | FR-007-AC-8 and TC-034 explicitly retain exact mapping formula bytes as a separately bound input. |
| FND-3503 | **FIXED** | FR-007 now requires a typed contextual identity failure before work, with FR-007-AC-9 and TC-035 owning the document-limit boundary. |
| FND-3504 | **FIXED** | Each existing tag was resolved to compiled Rust source and TC-025 through TC-031 are now marked implemented; the FR-007 summary remains planned until TC-034 and TC-035 land. |

## Base checklist result

- Identifier formats remain valid and collision-free in the reviewed scope:
  FR-007-AC-8 and AC-9 are sequential, TC-034 and TC-035 are sequential, and
  SR-035 follows the current maximum tracked review identity.
- FR-007 defines the input distinction, digest behavior, compatibility
  boundary, output identity effect, typed document-limit refusal, and dependency
  on the already-landed tl-syntax semantic view. No new user story, option,
  performance target, or security boundary is introduced by this correction.
- Cross-references resolve from FR-007-AC-8/AC-9 to TC-034/TC-035 and onward to
  the applicable stakeholder and non-functional criteria. Terminology
  consistently distinguishes diagnostic node spans from requirement-context
  spans.

## Six-rule coverage result

- **Coverage:** FR-007-AC-8 and AC-9 have owning regressions TC-034 and TC-035.
- **Option permutation:** no configurable option is introduced; the applicable
  presence permutation is absent versus present diagnostic spans.
- **Constraint boundary:** TC-034 names absent, empty, and distinct valid spans;
  TC-035 exercises one node above the canonical formula-document limit.
- **Error path:** TC-035 requires the canonical-view conversion failure to use a
  typed identity error before hashing or operation work.
- **State transition:** the requirement is a pure identity invariant and adds
  no state machine.
- **Edge case:** diagnostic span presence and offset variation are explicit,
  structural documents must remain distinct, and mapping formula bytes remain
  independently bound.

## Review conclusion

The corrected requirements and planned tests are sufficiently precise for
implementation. Strict coverage is expected to remain red for FR-007-AC-8/AC-9
and TC-034/TC-035 until the Rust regressions are implemented; none of those rows
may be marked implemented before that evidence exists.
