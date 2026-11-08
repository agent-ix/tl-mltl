# tl-mltl conformance corpus

The MIT OR Apache-2.0 shared temporal corpus (formula fixtures, malformed
cases, and their manifest), the future-operator corpus (`future-operators`
under `tl_syntax::CORPUS_DIR`), and the past-history corpus (`past-history`
under `tl_syntax::CORPUS_DIR`) are all read straight out of the compiled
`tl-syntax` dependency via `tl_syntax::CORPUS_DIR`. There is no
retained copy of any of them in this repository.

tl-mltl neither depends on nor runs tl-parse.

tl-mltl consumes the formula, profile, trace, horizon, and closed-verdict fields
without changing their meaning. Evaluator-specific and external-monitor cases
live in separate versioned manifests.

## Boundary corpus (`boundary-v1`)

`corpus/boundary-v1/cases.json` is a hand-authored corpus of 200-odd cases: for
every operator x interval-shape cell (point `[0,0]`, `[0,b]`, `[a,a]`, `[a,b]`,
and the `u32::MAX` extremes) at least one positive, one negative and one
boundary case, plus the named edge conditions (empty trace, horizon equal to the
prefix length, nested windows that only just fit, windows reaching before the
origin). Each case stores the formula, trace, evaluation position, profile and
expected verdict. The verdicts are derived by hand from `docs/semantics.md`; no
evaluator produces them.

`tests/boundary_corpus.rs` replays the closed-trace and origin-complete cases
against both `tl-oracle` and production, replays the online-prefix cases against
production (the oracle does not model that profile), and checks that every cell
keeps its three cases. A case tagged `work-limit` or `oracle-abstains` is checked
by production only, because the oracle's resource and empty-trace behaviour
differs by design; the case's `rationale` says why.

The formula JSON shape (`{"op":"until","interval":[a,b],"left":..,"right":..}`)
is shared with the minimised regression fixtures under
`tests/fixtures/small-scope-regressions/`.
