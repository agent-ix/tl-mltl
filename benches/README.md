# V9 evaluator and C2PO workloads

`v9_workloads` uses the public tl-mltl APIs for closed evaluation, a declared
closed prefix, lasso evaluation, fair lasso admission, and online-prefix C2PO
rendering. Each family has small (2 positions or operators), median (24),
and near-cap (96) cases. Evaluation limits are set to the input's temporal
span or materialized graph/trace dimensions where applicable. The C2PO
renderer receives exactly its graph's node count as its work limit.

The benchmark checks real successful outcomes before timing. Each generated
input's length-prefixed canonical wire parts must match `input-digests.json`.
The six additional closed cases vary trace length with width fixed at two,
then vary interval width with trace length fixed at 96. Run `cargo bench
--locked --features infinite-trace --bench v9_workloads -- --test` to exercise
all 21 cases without treating a smoke run as a
performance comparison.

Criterion 0.5.1 uses 20 samples, a 500 ms warmup, and a one second minimum
measurement period. The V9 comparison report requires two same-host paired
baseline/current runs; a missing or incomparable baseline remains incomplete.
