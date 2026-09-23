# V9 Criterion lane (FR-051)

`v9_criterion.py` measures 28 pinned parser, rewrite, and mlTL cases. Each crate's
benchmark verifies its input digests before timing. The runner requires clean,
exact baseline and candidate Git revisions and copies the candidate harness and
lockfile into a fresh baseline source staging area. Baseline and candidate use
separate Cargo target directories, so a cached candidate binary cannot stand in
for baseline code. Each pair retains the complete Criterion `sample.json` and
`estimates.json` files and raw command logs, plus source, harness, toolchain,
and host identity in `pair.json`.

Run at least two full pairs on the same idle host, with Rust 1.98.1 and a
provisioned Cargo home. All six paths below are clean worktrees, and pair output
paths must be fresh. Repeat any case above the 20% threshold or overlapping it
in a third pair before accepting a regression decision.

```sh
python3 campaign/v9_criterion.py measure \
  --baseline-parse /path/to/parse-baseline --candidate-parse /path/to/parse-current \
  --baseline-rewrite /path/to/rewrite-baseline --candidate-rewrite /path/to/rewrite-current \
  --baseline-mltl /path/to/mltl-baseline --candidate-mltl /path/to/mltl-current \
  --cargo-home /path/to/cargo-home --pair-dir /path/to/v9-pair-1
python3 campaign/v9_criterion.py report \
  --pair-dir /path/to/v9-pair-1 --pair-dir /path/to/v9-pair-2 \
  --output /path/to/v9-report.json
```

The report includes all per-case sample distributions, medians, sample
variances, median change, and bootstrap confidence interval. A confirmed
above-20% result requires two paired runs whose lower 95% confidence bounds
exceed 20%. One such run requires a third pair. Host/toolchain mismatch,
threshold overlap, a failed benchmark, and missing pairs are nonpassing and
reported distinctly. An empty report is `incomplete`; it does not claim a
performance result.
