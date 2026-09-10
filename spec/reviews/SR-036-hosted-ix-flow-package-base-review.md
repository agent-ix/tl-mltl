---
id: SR-036
title: "Base specification review — hosted ix-flow package identity"
type: SpecReview
analysis: base
scope: "agent-ix/tl-syntax#35; NFR-003-AC-5; TC-036; TM-001"
review_set: base
relationships:
  - target: ix://agent-ix/tl-mltl/NFR-003
    type: reviews
---

# SR-036: Base specification review — hosted ix-flow package identity

## Summary

The owner-selected base review checked the tl-mltl slice of the cross-repository
hosted-package correction for identifier integrity, requirement quality,
cross-references, and all six coverage rules. Two boundary gaps were corrected
before implementation: the package scan no longer depends on one npm command
spelling, and the manual-trigger/runtime-version requirements are explicit.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-3601 | medium | A control scoped only to `npm install --global` would miss accepted aliases such as `npm i -g`, allowing a later ix-flow install to override the reviewed package. | NFR-003-AC-5, TC-036, tl-syntax#36 reviewer carry-forward |
| FND-3602 | low | Naming only the package token would leave the manual-only trigger and the executable actually exercised by the local gate implicit. | NFR-003-AC-5, TC-036, `.github/workflows/ci.yml` |

## Dispositions

| Finding | Disposition | Evidence |
|---|---|---|
| FND-3601 | **FIXED** | NFR-003-AC-5 and TC-036 require inspection of every `ix-flow@` package token regardless of accepted npm install command or global-option spelling. |
| FND-3602 | **FIXED** | NFR-003-AC-5 and TC-036 separately require `workflow_dispatch` as the only trigger and exact `ix-flow --version` output `0.0.4`. |

## Base checklist result

- NFR-003-AC-5 does not reuse retired AC-4; TC-036 and SR-036 follow the
  current maximum active identities and the tracked review uniqueness control
  will reject a collision.
- The criterion names the exact package identity, version, cardinality, trigger
  boundary, and observed executable. It adds no workflow trigger, release
  decision, tool upgrade, registry substitution, or production behavior.
- NFR-003-AC-5 traces directly to TC-036, and TM-001 records the planned test
  without claiming implementation evidence early.

## Six-rule coverage result

- **Coverage:** TC-036 owns every clause added by NFR-003-AC-5.
- **Option permutation:** npm's `install`/`i` and `--global`/`-g` spellings
  cannot create a second unobserved package path because the oracle scans every
  token containing `ix-flow@`, independent of the command spelling.
- **Constraint boundary:** exactly one scoped 0.0.4 package token is allowed;
  zero, two, an unscoped token, or another version is refused.
- **Error path:** an absent executable, a non-zero version command, or any
  output other than `0.0.4` fails the test with a named diagnostic.
- **State transition:** hosted CI remains manually dispatched; adding any
  automatic trigger changes the reviewed workflow bytes and fails the control.
- **Edge case:** a later npm alias-form install carrying a different ix-flow
  version remains visible even when the original install line is unchanged.

## Review conclusion

The corrected NFR and planned test are precise enough for implementation. The
NFR-003 matrix row and TC-036 must remain planned until the Rust control exists
and has been demonstrated red against a conflicting alias-form install.
