---
id: SR-005
title: Code review of the tl-mltl shared assurance migration
type: SpecReview
analysis: code-review
scope: assurance/**, scripts/**, examples/**, tests/**/*.rs, tests/fixtures/**, schemas/README.md, Makefile, Cargo.toml, .github/workflows/ci.yml
review_set: all
---

# Code review of the tl-mltl shared assurance migration

## Summary

This is the opening review of the migration under issue #13, written before an
independent adversarial review of the same head. It is a self-review, and every
wave of this campaign has shown that self-review misses high-severity false
greens, so it is written to record what was actually measured rather than what
the design intends.

Six findings were found and fixed while building the change. Two were high, and
both were false greens in checks this change itself introduced — a
producer-isolation test that could not see one of its producers, and a corpus
producer that would have reported a deleted fixture as a correctly rejected one.
One finding is accepted with rationale rather than fixed, because the honest
statement is the record of a divergence rather than its removal.

The mutation-probe table below shows the FIRST result and the CURRENT result for
each probe, because a probe table that shows only the final state is a table
written after the fix.

## What changed

| Disposition | Items |
|---|---|
| KEEP | Bounded closed-trace evaluation, checked horizon and required-buffer analysis, pending-aware open prefixes, profile refusal, work and recursion limits, the C2PO mapping manifest, the external-verdict comparison layer, the shared temporal corpus, the retained R2U2 4.2 exchange, the CLI and its wire schemas, `check_unsafe_comments.sh`, `deny.toml`, `clippy.toml`, `rustfmt.toml`, `rust-toolchain.toml` |
| REPLACE | Local evidence envelope, manifest, anchor verifier, retraction registry interpretation, host-scoped tool identity, failure-propagation policing and the finalizer — all now upstream in Quoin, Quire and Engineering Assurance |
| DELETE | 15 files, 2,769 lines, in a separate commit after the dual run: `build_evidence_envelope.py`, `finalize_collection.py`, `test_evidence_tool.py`, `check_failure_propagation.py`, `test_failure_propagation.py`, `tool_identity.py`, `collect_evidence.sh`, `tests/evidence_contract.rs`, `test_tool_identity.py`, `evidence_profile.py`, `tools.lock`, `run_policy_tests.py`, `validate_json_schema.py`, `verify_evidence.sh`, `parameter_identity.py` |
| FREEZE | The two evidence schemas under `schemas/`, not deleted, and not in the same position as each other — see FND-506. They are referred to here as the input schema and the manifest schema rather than by filename, because TC-024's source census forbids naming them outside `schemas/README.md`, `assurance/change-assurance.json` and the test that pins their digests — and it caught this document doing so. |
| DEFER | The Make execution-control class, recorded and tracked as `agent-ix/tl-mltl#14`; the upstream PGM-01 mapping gap, `agent-ix/engineering-assurance#21`; the acceptance-state release gap, `agent-ix/engineering-assurance#20` |

Three domain producers were added, each emitting declared structured rows and
each exiting non-zero on a failing row: `reference_conformance` over the shared
temporal corpus, `r2u2_differential` over the retained external exchange, and
`cli_conformance` over the built CLI.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-501 | high | The producer-isolation test could not see `quire coverage`. Injecting `cargo build` into the driver was detected; injecting `quire coverage` was not, because `quire` was absent from the shim list, so a driver that regenerated the static export it is supposed to consume would have kept the test green. FIXED. | tests/shared_assurance.rs | correct-requirement-no-evidence |
| FND-502 | medium | `producer_shims` topped the shim directory up instead of recreating it, so a shim written by an earlier version of the test survived in `target/` and silently changed what the next run measured. FIXED. | tests/shared_assurance.rs | implementation-bug-despite-evidence |
| FND-503 | high | `reference_conformance.rs` read declared corpus fixtures with `unwrap_or_default()`. A deleted malformed fixture yields empty bytes, empty bytes fail to deserialize, and a deserialize failure is exactly what a malformed fixture is supposed to produce, so a deleted fixture would have been reported as a correctly rejected one. FIXED. | examples/reference_conformance.rs | correct-requirement-no-evidence |
| FND-504 | medium | The declared-stage table mapped `operand_not_preceding` to the `validate` stage. Measured: `tl_syntax::FormulaDocument`'s deserializer enforces operand order, so all three malformed fixtures are refused at the wire boundary. FIXED against the measurement rather than the assumption. | examples/reference_conformance.rs | implementation-bug-despite-evidence |
| FND-505 | medium | The CLI refusal markers were `parse request` and `unknown variant`, each matching more than one case, so four different refusals were separated by nothing. FIXED, with a `marker-discrimination` row measuring the cross-product. | tests/fixtures/cli-requests/manifest.json | correct-requirement-no-evidence |
| FND-506 | medium | The input evidence schema on disk is not the bytes any retained envelope names: four records name `808fd9f3`, two name `d763369e`, the file is `7b7e4725`. ACCEPTED with the divergence pinned by TC-024. | schemas/README.md | implementation-bug-despite-evidence |

### FND-501 — the producer-isolation test could not see one of its producers

The migration contract requires the isolation test to be verified by injecting
real producer calls into the driver rather than by reasoning about it. Both were
injected. `cargo build` was detected. `quire coverage` was not.

Adding `quire` to the shim list does not fix it, and the reason is now recorded
in the test: `quoin evidence record` invokes `quire coverage` itself, inside the
store it is writing, which is Quoin using the static exporter exactly as the
architecture intends. A PATH shim cannot separate that from the driver
regenerating its own input, and shimming `quire` makes the clean-tree run fail —
measured, not assumed.

The replacement is a direct measurement of the actual property. Each of the seven
declared inputs is moved aside in turn, the driver is run, and it must exit 2
naming `make assurance-inputs` with the file still absent afterwards. Verified
able to fail: a driver patched to run `quire coverage` when its export is missing
exits 0 and the probe catches it.

### FND-506 — the two frozen schemas are not in the same position

Accepted rather than fixed. The manifest schema's current bytes are exactly what
all six retained envelopes name. The input schema's are not: the records name two
earlier revisions of it, both still reachable in Git history at the revisions
each record binds itself to.

Restoring the file to one of those digests would break the other four records'
reference just as surely, and editing an immutable record is prohibited. So the
freeze on that file preserves an identity the records name rather than the bytes
they name, and that weaker claim is written down in `schemas/README.md`, stated
in the `PRESERVE-frozen-schemas` constraint, and pinned by TC-024 — which asserts
all three digests, including that the on-disk digest is *not* one the records
name, so the divergence cannot quietly close or widen.

## Mutation probes

Thirty-five probes. Each applies one named mutation, verified to have changed
the bytes, and runs the gate that is supposed to notice. A probe that crashes is
a broken probe, not a detection: there is no `except` clause turning a traceback
into a green tick.

Full results are recorded in SR-007, the closing review, with FIRST and CURRENT
columns. The one non-detection this opening review already has to record is:

**P01 — every attested result hardcoded to `passed`: NOT DETECTED (exit 0).**

This is Wave 1's first high finding reproducing in this chain. On a tree where
every producer genuinely passes, a `derive_result` that ignored its input and
returned the constant `passed` produces an identical report, and no honest-path
assertion can see it. P02 — all three producer streams rewritten to fail —
*was* detected at exit 2, which proves the function does read the bytes today;
what was missing was anything that would notice if it stopped.

The fix is a control inside the chain: the same `derive_result` is handed a
stream derived from the real one with its first passing row flipped to fail, and
it has to return `failed` while still returning `passed` for the real bytes. A
constant cannot satisfy both halves.

## The dual run

Recorded as observed, not as parity. At candidate `a90ddae`, with both paths
present in the tree:

| Path | Result |
|---|---|
| New | `make assurance-inputs` exit 0; chain matched 21 scenarios, 9 controls, 7 adapter probes; twelve states demonstrated; 9 of 11 shared-assurance tests pass, the two failures being the assertions that the deletion had not happened yet |
| Old, at candidate `a90ddae` | red |
| Old, at its own unmodified base `0e17649` | red |

The old path was already red before this migration touched it, for two
independent structural reasons. `verify_evidence.sh` exits 1 with *"no active
non-retracted evidence record supports qualification"*, because all six retained
records are retracted. `run_policy_tests.py` and `tool_identity.py --verify-live`
exit 1 and 2 because `tools.lock` pins
`/home/peter/dev/tl-mltl/.qualification-target` and `validate_lock` compares it
against the checkout's own path, so the gate cannot pass in any checkout except
one directory on one machine — the open H2 finding on PR #12.
`check_failure_propagation.py` is the only old gate that passes anywhere.
`make ci` at `0e17649` exits 2 both with and without an ambient
`CARGO_TARGET_DIR`.

No green baseline was manufactured for it.

## Verdict

CONDITIONAL. The six findings above are dispositioned, but this is a self-review
and the campaign record says that is not sufficient. An independent adversarial
review of the same head, with the single instruction *find false greens*, is
required before this can be considered reviewed, and its findings are recorded
in SR-007.
