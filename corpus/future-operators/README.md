# Paired W/M source and canonical-graph corpus

`tl-syntax.future-operator-corpus/v1`, revision 1, is the paired corpus
FR-010 names and TC-074 replays with `tests/future_operator_corpus.rs`. It is
separate from `tl-syntax-corpus/v1` in the parent directory, which is
unchanged.

The corpus is evidence input. It is not an evaluator, not an editable source
language, and it adds no derived formula-v1 node. The replay builds every graph
through the tl-syntax lowering API (`Formula::new` and
`FutureLoweringRequest::lower`).

## Files

- `manifest.json` names the corpus, operator-profile, request, formula-schema,
  derived-dialect, and primitive-dialect identities and the tl-parse revision
  the source cases were cross-checked against, and pins every replayed file by
  SHA-256. The test pins the manifest digest.
- `cases.json` holds every case.
- `expected/*.json` are span-free canonical formula-v1 documents. Each one is
  shared by one source case (derived or primitive) and one directly constructed
  case.
- `SHA256SUMS` repeats the digests for `make check-corpus`.

## Case classes

- `derived` binds `tl-parse.clean-ascii/v2`, the operator profile, a semantic
  profile, and source text to ordered `append` (primitive formula-v1 node) and
  `lower` (W or M request fields) steps. The replay checks that every span reads
  the bound source text, that each report matches `expected_lowerings`, and
  that the built graph equals the expected document without spans. W and M each
  have `[0,0]` and `[4294967295,4294967295]` pairs under both semantic profiles.
- `primitive` binds `tl-parse.clean-ascii/v1` source to append-only steps. It
  is the compatibility pair: old-dialect source spelling primitive U, G, and
  `|` yields a canonical graph a direct case shares.
- `direct` appends only span-free primitive nodes and must equal the same
  expected document exactly.
- `refused` ends in a lower step that must refuse with the recorded code;
  it produces no document. The code determines the admission axis, so no axis
  name is recorded. The refused cases are a sample; TC-046 owns the full
  refusal list. Only refused cases may override the request,
  operator-profile, or semantic-profile identity.
- `malformed` wraps an entry that must fail replay with the recorded error
  code, such as a derived wire node or a derived step under
  `tl-parse.clean-ascii/v1`.

Source spans and lowering records were checked against `tl-parse` `parse`
and `parse_clean_ascii_v2` at `9ca856b`, the revision `manifest.json` records.
tl-syntax does not depend on tl-parse, so the replay does not re-run the parser.
It binds every span to the source bytes and operator spellings, but precedence,
associativity, and grouping remain tl-parse grammar rules owned by TC-043.

## Changing the corpus

Edit the case or document, update its digest in `manifest.json` and
`SHA256SUMS`, then update `MANIFEST_SHA256` in the test. The mutation controls
in the test show that a changed expectation without a matching graph turns the
replay red.
