---
id: FR-022
title: "Measure the mutation kill rate against thresholds"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-003
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-020
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-009
    type: depends_on
---

# FR-022: Measure the mutation kill rate against thresholds

## Description

When the mutation campaign runs, tl-mltl shall run cargo-mutants only against a green
baseline, compute the kill rate, compare it with the declared thresholds, and list
every surviving mutant with a reason.

## Inputs

- The source revision, locked dependencies, compiler, and cargo-mutants version
  and configuration.
- The module scope: the evaluator core (evaluation, horizon, and interval
  arithmetic) and the crate as a whole, excluding CI-plumbing modules
  (`ci_guard`, `campaign`) because their tests do not exercise MLTL logic.
- The test selection that runs against each mutant.

## Outputs

- A campaign record with the tool version, the mutant counts by outcome, the kill
  rate for the evaluator core and for the crate, and the survivor list.
- A pass or a failure of the threshold gate.
- A typed refusal when the baseline is not green.

## Behavior

The campaign refuses to run, and emits no kill rate, unless the unmutated test
suite passes at the same revision with the same test selection. A baseline that
passes only intermittently is treated as not green.

Each mutant is run alone against the same test selection under a finite timeout.
Outcomes are `caught`, `missed`, `timeout`, `unviable`, and tool error, and they
are reported separately. A mutant that does not compile (`unviable`) exercised no
test and is outside the rate. A timeout counts against the rate, conservatively,
because the suite did not demonstrate that it noticed the change. The kill rate
is `caught / (caught + missed + timeout)`, and the record prints it beside the
raw counts.

The thresholds are:

| Scope | Minimum kill rate |
|---|---|
| evaluator core (evaluation, horizon, interval arithmetic) | 95% |
| the crate as a whole | 85% |

A kill rate below either threshold fails the gate. On a pull request the run is
limited to the mutants in changed code (`cargo mutants --in-diff`) so a change
cannot add untested logic; a weekly run covers every mutant.

Every missed or timed-out mutant appears in the survivor list with a reason, one
of: a new test that kills it, equivalent with a one-line reason recorded in the
configuration, or routed to a Kani harness under FR-023 with the harness named. A
survivor without a reason keeps the campaign open. A kill rate is a measurement of
the test suite, not a claim about the correctness of the code it did not reach.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-022-AC-1 | The campaign mutates the evaluator core and the crate as a whole, and excludes the CI-plumbing modules by a stated configuration, so plumbing tests do not inflate the rate. | Test (TC-113) |
| FR-022-AC-2 | A baseline that fails or is unstable refuses the campaign and emits no kill rate. | Test (TC-114) |
| FR-022-AC-3 | Caught, missed, timeout, unviable, and tool-error outcomes are reported separately, and the kill rate is caught over caught plus missed plus timeout, printed beside every raw count. | Test (TC-115, TC-116) |
| FR-022-AC-4 | A kill rate below 95% for the evaluator core or below 85% for the crate fails the gate, and a deliberately weakened test selection is detected by the gate. | Test (TC-116, TC-118) |
| FR-022-AC-5 | Every surviving mutant is listed with exactly one reason (a killing test, equivalence with a stated reason, or a named Kani harness), and a survivor without a reason fails the check. | Test (TC-117) |

## Dependencies

Depends on the FR-020 property laws as the baseline suite under test, and on FR-023
for survivors that tests cannot reach. Each source repository owns its own
mutation configuration and remediation.
