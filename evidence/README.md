# Retained evidence

Run `bash scripts/collect_evidence.sh` from a clean repository root. Each run
creates a revision-and-UTC-time-scoped directory and refuses overwrite. It
retains separate stdout/stderr, exit codes, tool/source identities, limitations,
canonical `quire.derivation-evidence/v1`, and an external SHA-256 file.

`evidence/ANCHORS` binds every retained outer manifest. `make verify-evidence`
first checks those committed anchors, requires every retained manifest to have
one, and then checks its contents and re-derives the post-seal summary.

Set `PGM01_SCHEMA` to the merged PGM-01 Draft 7 schema and `PGM01_VALIDATOR` to
its exact validator. Missing external gates are recorded as unavailable, never
as successful. The input record pins tl-syntax, both corpora, R2U2/C2PO source
and executable identities, dependency lockfile, schemas, and PGM-01.

The collector architecture is adapted under MIT OR Apache-2.0 from the
same-program tl-syntax collector at revision
`740182f13b84858008d6f176f75136737d405c1b`. It is tailored to tl-mltl's
semantic, resource, CLI, corpus, and external differential gates.

Evidence informs the open human source-release decision. It does not approve,
publish, validate, accredit, or qualify a consuming monitor or project.
