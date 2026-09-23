# V1 verification campaign runner

`v1_campaign.py` is a local, standard-library-only runner for FR-054 and
FR-055. It runs each selected command in an exact Git source graph and writes
one JSON report plus raw stdout/stderr files. A recorded external lane is
admitted only when its receipt names the same five source commits and input
hashes and its captured bytes still match their digests. The runner derives
status from those bytes and exit code; a claimed status in a receipt is ignored.
Each source label must resolve to a distinct clean repository with a matching
Cargo package name, so one repository cannot stand in for another crate.
Imported records remain incomplete as milestone gates even when the captured
bytes show a passing test. The registered executable commands cover the
finite/past oracle comparison, oracle seeded-fault and dependency-boundary
tests, both finite and infinite rewrite wrong-rule controls, the infinite
oracle (also required for V1 lasso coverage), infinite behavior, and oracle
semantic-law tests; an
arbitrary command with a plausible test summary cannot complete their gates.
The other required gates have named `unsupported` contracts in the
report, with a specific missing native output/parser. Adding one requires an
exact invocation, a parser for its actual population, and fault tests. This
runner presently reports those milestones open; it is not a full V1–V11
completion gate.
V11 now has a native `cargo_v11_population` gate over a declared small
lasso/fairness/partial partition: 30 formula graphs, 372 distinct words, three
fairness modes, and four selected positions. Its production comparison reports
133,920 cases, with 108,720 admitted and 25,200 empty-fair refusals. The
parser checks the exact axes, source command, complete ledger counts, and raw
test result; malformed or missing cases cannot pass. The marker explicitly
sets `full_target_complete` to false because longer lassos, deeper formulas,
and other intervals remain outside this partition. A V11 milestone pass means
all four named local gates passed for this declared scope.
V2 runs `tests/v1_finite_partition.rs` through a native `cargo_population`
parser. That producer enumerates 375 one-level formulas over `{false,true,p0}`
and 34 word positions (12,750 comparisons) with all closed intervals through
2, then prints one JSON census after reconciling visits and seeded ledger
faults. Its small partition can pass independently. V2 remains incomplete
because the full depth-three, interval-through-4, length-through-6 domain has
not run.

Run `python3 -m unittest discover -s campaign -p 'test_*.py'` for the
TC-195–199 fault tests. Generate a manifest with
`python3 campaign/make_manifest.py --repos-root /Users/peter/dev
--output /private/tmp/tl-v1-manifest.json`. The helper records all tracked
`corpus/` and `fuzz/corpus/` files across the five source repositories, plus
parser benchmark inputs, with their exact digests. Review and add any
lane-specific untracked live-target raw inputs before running the campaign.
The manifest contains
`schema`, the five `sources` (`path` and 40-character `revision`), `inputs`
(`path` and `sha256`), and `lanes`, then run:

```sh
python3 campaign/v1_campaign.py \
  --manifest /private/tmp/tl-v1-manifest.json \
  --raw-dir /private/tmp/tl-v1-raw \
  --output /private/tmp/tl-v1-report.json
```

Each lane names an `id`, `milestone`, and `mode`. A `command` lane also names
`repo`, argument-vector `argv`, `parser` (`cargo_test`, `cargo_population`,
`cargo_v11_population`, or `population_json`),
an explicit fixed or deterministic `seed`, and an optional timeout. `record`
lanes name a receipt JSON with exact source
revisions, input digests, parser, exit code, and paths and SHA-256 digests for
captured stdout and stderr. `not_run` and `blocked` lanes keep explicit
reasons. Unspecified required lanes are automatically `not_run`.

The fixed V1–V11 gate table in the runner determines each milestone. A
zero-exit command without a nonvacuous cargo test summary or reconciled
population cannot pass. Missing, failed, stale, timed-out and partially
visited lanes retain separate statuses. The semantic payload excludes raw
artifact paths and volatile timing text so identical deterministic runs can
be compared by `semantic_sha256`; the report separately retains raw artifact
paths and digests. The report states automated evidence only. TL-215 review,
release, native parity, and certification are outside the runner.
The semantic payload reports tool versions and paths, each source's Cargo
manifest and lock digests, direct dependency pins and feature declarations,
and named measurement slots for domain counts, fuzz and mutation populations,
proof bounds, coverage, performance, and the live target. Unrun measurement
slots remain explicitly `not_run`.

The retained campaign manifest/report, when present, identify their measured
source commits. A report committed after the run is an artifact of those
commits; it does not retroactively claim to have measured the artifact commit.
