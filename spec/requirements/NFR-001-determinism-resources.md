---
id: NFR-001
title: Preserve determinism and bounded resource claims
type: NFR
---

# NFR-001: Preserve determinism and bounded resource claims

## Statement

All library results and serialized records shall depend only on identified
inputs. Algorithms shall check arithmetic, configured evaluation recursion and
work limits, and the mapping recursion boundary before performing impractical
temporal expansion. Each declared resource limit shall admit an input exactly at
the limit and refuse an input one past it with its own typed error, and the test
suite shall produce every variant of every public error enumeration at least once.

## Scope

All public library functions, CLI records, mapping manifests, context identities
and digests, and retained differential artifacts are in scope. The crate depends on `std`
(`std::error::Error`, `std::io`, `std::time::Instant`) and makes no `no_std`
claim (FR-027, ADR-003); a `no_std` build is out of scope for this requirement.

## Rationale

Reference-oracle and resource-selection evidence must be reproducible and must
not understate bounds after arithmetic or work-budget failure.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Nondeterministic output mismatches | 0 | 0 | Test |
| Wrapped or partial resource claims | 0 | 0 | Test |

## Verification

Requirement-tagged unit and integration tests repeat semantic and serialized
operations and exercise checked arithmetic and configured work limits. Limit-edge and error-variant tests are written against
the typed values, never message text. The unit and property suites are also run
under Miri on a pinned nightly to show the absence of undefined behavior.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-001-AC-1 | Repeated evaluation, analysis, mapping, context binding, digesting, comparison, and serialization are byte-for-byte stable. | integration-testing (TC-003, TC-013, TC-014, TC-025, TC-028, TC-029) |
| NFR-001-AC-2 | Overflow and work-limit exhaustion are explicit errors with no partial success claim. | unit-testing (TC-004, TC-006) |
| NFR-001-AC-3 | Each declared resource limit (the recursion ceiling, the future, past and owner-wire evaluation limits, the infinite-trace work limits, and the past identity, unit, history and proposition size constants) admits an input exactly at the limit and refuses an input one past it with the typed error that names that limit. | unit-testing (TC-175) |
| NFR-001-AC-4 | Every variant of every public error and error-code enumeration is produced by at least one test that asserts the variant itself, not its message. | unit-testing (TC-176) |
| NFR-001-AC-5 | The unit and property suites complete under Miri with no undefined-behavior report, using the recorded flags and reduced case counts. | unit-testing (TC-177) |

## Dependencies

Constrains FR-001 through FR-005 and FR-007.
