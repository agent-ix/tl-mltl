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
all six test-coverage rules. Two boundary ambiguities were corrected before
implementation: optional span presence is now explicit, and mapping identity
holds its separately supplied formula bytes fixed while excluding diagnostic
node spans.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-3501 | medium | “Different spans” did not explicitly cover the absent-versus-present boundary even though optional-span serialization is one way diagnostic provenance can leak into identity. | FR-007-AC-8, TC-034 |
| FND-3502 | medium | Mapping also binds exact formula bytes, so an unconditional equality claim across differently spelled source inputs would conflict with the existing content-identity contract. | FR-007 digest allocation, FR-007-AC-8, TC-034 |

## Dispositions

| Finding | Disposition | Evidence |
|---|---|---|
| FND-3501 | **FIXED** | FR-007-AC-8 and TC-034 now cover absent, empty, and distinct valid diagnostic spans with every other operation input fixed. |
| FND-3502 | **FIXED** | FR-007-AC-8 and TC-034 explicitly retain exact mapping formula bytes as a separately bound input. |

## Base checklist result

- Identifier formats remain valid and collision-free in the reviewed scope:
  FR-007-AC-8 is the next FR-007 criterion, TC-034 is the next test case, and
  SR-035 follows the current maximum tracked review identity.
- FR-007 defines the input distinction, digest behavior, compatibility
  boundary, output identity effect, and dependency on the already-landed
  tl-syntax semantic view. No new user story, option, constraint, error code,
  performance target, or security boundary is introduced by this correction.
- Cross-references resolve from FR-007-AC-8 to TC-034 and from TC-034 to
  StR-003-VC-1, NFR-001-AC-1, and NFR-002-AC-4. Terminology consistently
  distinguishes diagnostic node spans from requirement-context spans.

## Six-rule coverage result

- **Coverage:** FR-007-AC-8 has the single owning regression TC-034.
- **Option permutation:** no configurable option is introduced; the applicable
  presence permutation is absent versus present diagnostic spans.
- **Constraint boundary:** TC-034 names absent, empty, and distinct valid spans
  while SourceSpan construction continues to reject inverted spans upstream.
- **Error path:** no new fallible public operation or error state is introduced;
  existing contextual failures remain governed by FR-007-AC-1 through AC-7.
- **State transition:** the requirement is a pure identity invariant and adds
  no state machine.
- **Edge case:** diagnostic span presence and offset variation are explicit,
  structural documents must remain distinct, and mapping formula bytes remain
  independently bound.

## Review conclusion

The corrected requirement and planned test are sufficiently precise for
implementation. Strict coverage is expected to remain red for FR-007-AC-8 and
TC-034 until the Rust regression is implemented; neither row may be marked
implemented before that evidence exists.
