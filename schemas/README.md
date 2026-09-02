# Frozen evidence schemas

These two files are **frozen, not live**. Nothing validates against them, and no
script in this repository may reference them.

| File | SHA-256 of the current bytes |
|---|---|
| `tl-mltl-evidence-input-v1.schema.json` | `7b7e4725bc05d1aafdda7af1586449dbaec6dae2e0893d204acf188347daff24` |
| `tl-mltl-evidence-manifest-v1.schema.json` | `8744bfe233f10f2dd6fe3a9d2948d2424802eda0489e4874b79428e6bf73cca1` |

They were the schemas of the local evidence framework the shared-assurance
migration removed. They are kept rather than deleted because the six retained
envelopes under `evidence/` name the identities `tl-mltl.evidence-input v1` and
`tl-mltl.evidence-manifest v1` by path and by SHA-256, and those bytes are
immutable.

## The two files are not in the same position, and saying otherwise would be false

`tl-mltl-evidence-manifest-v1.schema.json` is exactly what the records name. All
six retained envelopes carry the output schema digest
`8744bfe233f10f2dd6fe3a9d2948d2424802eda0489e4874b79428e6bf73cca1`, which is the
digest of the file as it stands.

`tl-mltl-evidence-input-v1.schema.json` is **not**. The six records name the input
schema by two different digests, and neither is the current file:

| Digest named by a record | Records | Present in the working tree |
|---|---|---|
| `808fd9f33720066e136188722daf0d4ce254fb846fd16f1e9073d1d3175138e2` | 4 | no |
| `d763369e194bc9b908b456f6da0f39266720cc4bb77a5102d21c62b51c0b2d3a` | 2 | no |
| `7b7e4725bc05d1aafdda7af1586449dbaec6dae2e0893d204acf188347daff24` | 0 | yes |

The file was edited twice after the last record was sealed, in `5d717ab` and
`2bb5e40`, while the collector those records came from was being hardened. Both
of the digests the records actually name remain reachable in Git history at the
revisions each record binds itself to, which is where a reader has to go to
resolve them.

So the freeze on this file preserves an *identity* the records name, not the
*bytes* they name. That is a weaker thing than the manifest schema's freeze and
it is written down here rather than smoothed over, because a blanket claim that
"the retained envelopes name these bytes by digest" would have been untrue for
one of the two files.

`tests/shared_assurance.rs` pins all three digests — the two current files and
the fact that the two record-named input digests are not the current one — so the
divergence cannot quietly close or quietly widen.

## Nothing validates against them

`tests/shared_assurance.rs` censuses the whole source tree — every directory
except `evidence/`, `target/` and the assurance virtualenv — to prove nothing
references them.

Three files name them on purpose and are allow-listed by that census: the test
itself (which pins the digests), this README (which documents the freeze), and
`assurance/change-assurance.json` (which states the preservation constraint).
Anything else naming them fails TC-024.
