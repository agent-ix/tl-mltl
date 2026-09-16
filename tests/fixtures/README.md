# Test fixture provenance

`fcd-static-bundle-1.2.json` is an exact test-only copy of
`agent-ix/filament-core-data:fixtures/baseline-1-2/static-bundle-a.json` at
commit `404288282402d60de007295ccbafa960532b955e` (Producer interface 1.2,
PR #99). The source repository and fixture are AGPL-3.0-only. This repository
uses it only to exercise the exact QObs admission boundary; production code
contains no copied producer artifact or decoder.
