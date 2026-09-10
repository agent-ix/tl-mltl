---
id: SR-046
title: "Base review of the verification-effectiveness campaign"
type: SpecReview
analysis: base
scope: "MRS-003, FR-011 through FR-015, NFR-005, MP-003 through MP-006, TM-003"
review_set: all
---

## Summary

**PASS after remediation.** The issue #39 artifacts define distinct property,
fuzz, mutation, and bounded-proof observations, retain their finite populations
and limitations, and keep release authority human. Native Quire remains the
sole authored clause language. This author-run review does not replace an
independent exact-head PR review or admit blocked implementation.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-4601 | high | Early wording let counts, plateaus, mutation ratios, and bounded proofs drift toward semantic or release authority. Fixed with domain-specific non-claims, raw populations before summaries, and a zero automated-authority gate. | MRS-003, FR-012-AC-5, FR-015-AC-6, NFR-005 |
| FND-4602 | high | The first shared-intake draft assumed Quoin retained arbitrary binary artifacts and represented instrumented build profiles. Fixed: fuzz/proof attachment and non-release build-profile intake are explicit shared-capability blockers tracked by quoin#363/#364 with no local substitute. | FR-015-AC-4, FR-015-AC-5 |
| FND-4603 | medium | Initial local MeasurementPlans appeared to govern all four repositories despite repository-local Quoin plan resolution. Fixed: MP-003 through MP-006 govern tl-mltl only; siblings require independently accepted local plans. | FR-015, MP-003 through MP-006 |
| FND-4604 | medium | Matrix priority was initially a standalone Kani trigger, making nearly every criterion a proof candidate. Fixed: priority orders candidates admitted by a property gap, fuzz result, mutation disposition, or exact reviewed bounded proposition. | MRS-003, FR-014 |
