# R2U2 4.2 differential report

The exact C monitor built from canonical `R2U2/r2u2` tag `4.2-release`, commit
`336a2453dd2bd89bd26e9e45fb772a4bf77e4a6a`, was run with C2PO 4.1.0.
Eight declared supported formula/time cases agree with tl-mltl in truth value
and external verdict index. The raw aggregated verdict stream is retained in
`r2u2.stdout`; the executable, compiled specification, source inputs, and tool
script are pinned by SHA-256 in `manifest.json`.

| Case | tl-mltl | R2U2 | Verdict time | Status |
|---|---:|---:|---:|---|
| `r2u2-future-witness-v1` | true | true | 0 | agreement |
| `r2u2-globally-counterexample-v1` | false | false | 0 | agreement |
| `r2u2-future-deadline-v1` | false | false | 0 | agreement |
| `r2u2-until-lower-bound-v1` | true | true | 0 | agreement |
| `r2u2-release-lower-bound-v1` | false | false | 0 | agreement |
| `r2u2-nested-until-v1` | true | true | 0 | agreement |
| `r2u2-future-at-one-v1` | false | false | 1 | agreement |
| `r2u2-globally-at-one-v1` | true | true | 1 | agreement |
| `closed-profile-not-mapped-v1` | n/a | not run | n/a | unsupported profile difference |

R2U2 aggregates adjacent identical time-indexed verdicts; the comparison now
retains each selected external time index. This evidence neither measures
external resource use nor qualifies R2U2 or a consuming monitor. The source
release decision remains pending independent human review.
