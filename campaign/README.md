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
V10 has two explicit live-target commands. They require the caller to opt in
with a clean local checkout of R2U2 at `336a2453`, with the pinned C2PO entry
and monitor executable bytes. The normal manifest never invokes that foreign
runtime. Its explicit Cargo example retains fresh compiler/monitor raw files,
compares eight reviewed bounded cells and eighteen past cells with the
independent oracle, classifies the known unsafe `S[0,2]` origin mismatch as
unsupported mapping, and checks one finite bad-prefix refutation against the
oracle. It also exports the `G[0,)q` safety body, compiles and executes the
exported `q` expression, replays its first false target step as a provider bad
prefix, and classifies a later true step as inconclusive. The second command
generates the TL-217 past grid: 75
operator/interval/depth shapes, three trace classes, six positions, 21
C2PO/R2U2 executions and 1,350 per-step cells. Its gate checks source/lock
and target pins, reconstructs all 21 spec/trace inputs, checks 147 raw artifact
digests, requires 360 admitted agreements and 990 mapping refusals, and refuses
missing or mismatched admitted target rows. Both commands must pass for V10.
This is a bounded reviewed population, not general R2U2 parity.

V4 has a fixed native gate that reconciles five checked-in libFuzzer reports
across the four production crates, including both mlTL mapping and evaluation.
It checks their measured source ancestors, unchanged targets and lock
files, checked corpus digests, lossless raw streams, actual 1,000-execution
`DONE` markers, and crash-artifact state. A clean outcome is a bounded
observation for those five named targets. The manifest builder adds this lane
automatically with seed 181; stale or missing per-crate evidence leaves V4
incomplete.

V6 has a live native Kani gate. It runs the exact syntax interval and mlTL
checked-horizon arithmetic harnesses with unwind bound 2, CaDiCaL, and Kani's
default memory, overflow, assertion-reachability, and unwinding checks. The
symbolic domains are two unconstrained `u32` endpoints and an unconstrained
`u32`/`u64` operand pair, with no assumptions. It parses individual check
results, verifier and solver identities, and one complete harness summary per
claim. In a temporary archive of the syntax source commit it adds one
verifier-only false assertion, requires Kani's concrete playback bytes, and
runs an ordinary Rust replay of those exact bytes against the unchanged
production interval body. All raw stdout/stderr streams and digests are
retained outside the source tree. A timeout, missing tool, disabled check,
incomplete proof, absent counterexample, or failed replay leaves V6 incomplete.

Every required gate has an exact invocation or native verifier. The campaign
reports the measured status of each lane; registered execution alone does not
close an incomplete population.
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
faults. A second native `cargo_full_domain_census` lane expands the completed
partition to all 807 depth-one formulas with intervals through 4 and every
one-atom word position through length 6 (518,094 oracle comparisons). Its
depth-three census uses the declared ordered-tree grammar, no symmetry
reductions, and the same 642 word positions. The literal target has
2,093,484,708,816,033 formulas and 1,344,017,183,059,893,186 cells;
1,344,017,183,059,375,092 remain unvisited. FR-044 makes the 518,094-cell
depth-one partition the V2 exit gate. The runner retains the depth-three
unvisited count and does not claim literal all-trees exhaustion.

To deliberately include both V10 lanes, generate a fresh manifest with
`--live-r2u2-source /absolute/path/to/pinned/r2u2`. Use a new empty
`--raw-dir` for each campaign run. The runner checks the foreign source
revision and binary digests before invoking it; it checks all 22 original raw
artifacts and all 147 grid artifacts against the examples' reported hashes.
An absent source leaves V10 not run; a missing or changed pinned source cannot
pass. The examples run only through these explicit campaign lanes.

V7 is also opt in. Add `--v7-cargo-home /absolute/path/to/provisioned/home`
when creating the manifest and use a fresh empty campaign raw directory. The
registered command runs `v7_native.py` against the manifest's five current,
clean source revisions. The campaign independently checks all eleven exact
probe commands and feature selections, three target builds, eight Miri tests,
34 paired limit edges, three refusal-only edges, six explicit Miri exclusions,
and each raw log digest. The retained `campaign/evidence/v7-native.json`
measures its listed earlier commits; importing that report cannot pass V7 for
a later source graph.

V3 uses the native `cargo_properties` lane from `tests/property.rs`. A fixed
ChaCha seed drives 64 accepted production-evaluator cases for duality,
bounded embedding, loop unrolling, fairness weakening, partial-information
monotonicity, and finite-prefix refutation. The same test checks 24 strict
owner-wire round trips and classifies all 61 checked-in criteria as property,
existing example, or justified exclusion. The parser requires the exact
seed, counts, criteria, example test names, and clean Cargo summaries; its
fault tests reject missing or duplicate evidence. Parser-owned wire editions
and rewrite equivalence remain in their owning crates, so this lane is a
bounded V3 result, not a claim that those external obligations ran here.

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

Each lane names an `id`, `milestone`, and `mode`. V4 uses the fixed `native`
mode with `{ "kind": "fixed", "value": 181 }` as its seed identity; it accepts
no caller-selected command or parser. A `command` lane also names
`repo`, argument-vector `argv`, `parser` (`cargo_test`, `cargo_population`,
`cargo_properties`, `cargo_live_target`, `cargo_v11_population`, `v7_native`, or
`population_json`),
an explicit fixed or deterministic `seed`, and an optional timeout. `record`
lanes name a receipt JSON with exact source
revisions, input digests, parser, exit code, and paths and SHA-256 digests for
captured stdout and stderr. `not_run` and `blocked` lanes keep explicit
reasons. Unspecified required lanes are automatically `not_run`.

The fixed V1–V11 gate table in the runner determines each milestone. A
zero-exit command without a nonvacuous cargo test summary, a validated live
target marker, or a reconciled population cannot pass. Missing, failed, stale, timed-out and partially
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

V8 lists every uncovered critical branch even when a reviewer concludes that
the branch cannot be reached through the public owner boundary. The standalone
`v8_coverage.py` producer accepts an optional `--reviews /path/reviews.json`.
For a V1 aggregate, pass the same file with `make_manifest.py --v8-reviews`;
the manifest declares its exact path and SHA-256, and the runner checks the
bytes before and after native measurement and against the emitted V8 report.
The V8 native report uses `tl-mltl.v8-coverage/v2`. That JSON review file is a
list of two review record types. A source-location record
has exactly `run`, `file`, `line`, `column`, `true_count`, `false_count`,
`source_file_sha256`, `reason`, and `reviewer`. The location and counts must
match one measured gap in that run. A file-summary record has exactly `kind`
(`file_summary`), `run`, `file`, `summary_missing_sides`,
`source_file_sha256`, `raw_export_sha256`, `reason`, and `reviewer`. It reviews
all `branches.count - branches.covered` missing sides in that critical file's
LLVM summary and binds the decision to the exact raw export. The summary's
missing sides require this file-level review even when a named source gap is
also present: LLVM does not identify which detail record contributes to the
summary. `0/0` or one-sided detail omitted from the summary still needs its
separate source-location review.
Both types require the exact source-file digest, a substantive infeasibility
reason, and the name of the person who checked it. An empty list records no
decisions. Unknown, duplicate, stale, or cursory records are rejected. The
native gate recomputes all deficits and reviews against the raw LLVM export
and source bytes. Any unreviewed named gap or file-summary deficit, or a
missing critical file, keeps V8 incomplete. Reviewed deficits remain in
`critical_uncovered` or `critical_summary_missing` and in the measured count;
review never masquerades as
executed coverage. Do not add a review record without an actual human review
of that exact source and measurement.

The retained campaign manifest/report, when present, identify their measured
source commits. A report committed after the run is an artifact of those
commits; it does not retroactively claim to have measured the artifact commit.

## Mechanical Stage 1 Campaign

`stage1-campaign-definition.json` is the external control-plane definition
for 117 direct native EA measurements. Its `tl-mltl` source is the clean
`536a04748ec9cba18e04909246748e17af0cecdd` checkout. Keep this
control-plane checkout separate from that measured checkout: the definition
and generated machine config are later artifacts that name the measured tree,
so they cannot be part of their own source digest. Pass the clean measured
checkout as Quoin's `--repo`, and pass the definition and config by their
external paths. Every other source alias needs its own clean checkout at the
definition's exact revision.

`tl_campaign_config --definition FILE --machine FILE --output FILE` derives
the Quoin configuration from authored procedures. It checks each selected
checkout's Git revision, cleanliness, and full-tree inventory digest; selected
Cargo manifests, locks, and historical V9 benchmark harness bytes; and exact
plan, procedure, and checker source equality between the control-plane and
measured tl-mltl checkouts. It checks executable file bytes against the
machine table. A generated config is only an execution selection, never a
measurement result. A member passes only after Quoin retains the bounded EA
producer and checker results and independently reconciles their identities.

The direct C2PO compile procedures bind Python 3.13.11. A bounded smoke on
Linux host `cave` used R2U2 commit
`336a2453dd2bd89bd26e9e45fb772a4bf77e4a6a` with C2PO 4.1.0
(`compiler/c2po.py` SHA-256
`f978a32f667a8247c387a66bce35371c97b7d8f7b730035a8ee40cdfc428ce12`),
`PYTHONPATH=compiler`, and the tracked bounded specification and map.
The exact Python executable was `/usr/bin/python3.13`, SHA-256
`bf9c0f7057bc1a4c9169ed54ce941a86c07ede3ba3793186a074887dd36af137`.
The isolated Python 3.13 venv contained only `pip` 25.3; C2PO used its vendored
compiler code. Invocation
`python compiler/c2po.py --spec .../formulas.c2po --map .../signals.map --output .../bounded.bin`
exited 0 and produced a binary with SHA-256
`234c5f0a1fb827c1ef10cab4ed4ae9ce8ffdb07e6863c6fa9522730e49ca0da8`.
The same pinned R2U2 source built its C monitor with `make -C monitors/c all`
on `cave`; the Linux executable SHA-256 was
`6b98ee5cfcad7073eef49a333b00be1e5b512ed9d3bed6b4e07418357a87ab92`.
It exited 0 on that compiled binary and the tracked bounded trace, emitting
14 target rows; retained stdout SHA-256 was
`567306aaf08c6d4603f770c91a5cdcd56a2ab3214ada5e6ae68661ac518501c3`.
This smoke proves only that the pinned compiler starts and compiles one
tracked input and that the native monitor runs on that result. It is not a
V10 Campaign pass.
