# tl-syntax shared temporal corpus

`manifest.json` is the authoritative index for corpus revision
`tl-syntax-corpus/v1`. Consumers must pin and report that exact revision in
their own conformance evidence rather than following a mutable branch.

Formula files conform to `schema/formula-v1.schema.json`; proposition names
conform to `schema/proposition-map-v1.schema.json`. The Rust serde model is the
executable validator. Fixture identities, class names, paths, traces, horizons,
and expected closed-trace outcomes are stable data in v1.

A trace is an array of instants, each containing the numeric identities of the
propositions true at that instant. `expected_horizon` is the formula's maximum
look-ahead from the current instant. `expected_closed_trace` is present for
cases whose manifest supplies an evaluation oracle. Syntax validation does not
compute either value; downstream reference evaluators consume them.

Malformed files deliberately violate either checked interval decoding or
formula graph validation. They are corpus inputs, not examples of accepted
documents.

`SHA256SUMS` pins the exact bytes of every JSON artifact in the revision and is
verified by `make check-corpus` and `make ci`.
