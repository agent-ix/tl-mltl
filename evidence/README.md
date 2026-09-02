# Retained evidence

**These bytes are immutable and nothing in this repository writes them.**

Six records, 283 files, unchanged by the shared-assurance migration. They were
produced by a repository-local evidence collector that the migration removed;
what went away is the verifier, not the record.

## What still reads them

`scripts/legacy_evidence_view.py` opens every file here for reading, digests the
whole tree before and after the run, and fails if one byte moved. Read-only is a
claim, so it is measured. Whether the retained bytes are the bytes that were
*committed* is a separate and stronger claim, and Git is asked rather than a
second local manifest being invented.

Every envelope is interpreted by
`engineering_assurance.verification_semantics.map_pgm01_bytes` from the pinned
release. This repository implements no mapping of its own.

All six declare `quire.derivation-evidence/v1`, which is a schema family the
PGM-01 program governed but did not define, so the mapping answers
`incompatible` with the reason `unknown PGM-01 schema version` for every one of
them. That is the mapping declining to interpret a shape it has never seen. It is
reported as it stands: not a pass, not a defect of these records, and not a
licence to write a local mapper. Filed upstream as
`agent-ix/engineering-assurance#21`.

## What no longer reads them

`evidence/ANCHORS` and `evidence/RETRACTIONS.json` are retained bytes like any
others. Nothing interprets them any more — `verify_evidence.sh`,
`finalize_collection.py` and `evidence_profile.py` went with the rest of the
local framework. The dispositions `RETRACTIONS.json` records still stand as
history: none of the six records supports an active qualification claim, and
`spec/assurance/AA-001.md` says so in its own voice rather than by delegating to
a registry a script parses.

There is no collector. `scripts/collect_evidence.sh`, `tools.lock`, the
host-scoped executable census, the envelope builder and the manifest verifier are
gone. New assurance for a candidate revision is produced by
`make assurance-inputs` and sealed through Quoin by
`scripts/assurance_chain.py`, into a store under `target/`, which is ignored.
Nothing writes here.

## Provenance of the removed collector

The collector architecture was adapted under MIT OR Apache-2.0 from the
same-program tl-syntax collector at revision
`740182f13b84858008d6f176f75136737d405c1b`. That attribution is kept because the
records it produced are kept.

Evidence informs the open human source-release decision. It does not approve,
publish, validate, accredit, or qualify a consuming monitor or project.
