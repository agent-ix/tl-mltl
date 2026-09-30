---
id: SR-077
title: "tl-mltl#115 drop dangling PGM-01 citations review"
type: SpecReview
analysis: base
scope: "agent-ix/tl-mltl@0c0fe0ee63f228ec5ef558bf68cb7b6916a3a42f; diff against origin/main a53a104; assurance/change-assurance.json; spec/assurance/MP-001.md; spec/requirements/NFR-004-reproducible-corpus-retention.md; spec/reviews/SR-004-pgm01-reconciliation.md (deleted); src/lib.rs"
review_set: subset
---

# tl-mltl#115 drop dangling PGM-01 citations review

## Summary

Ticket: TL-272. PR: agent-ix/tl-mltl#115, branch `chore/drop-pgm01-citations`,
reviewed at `0c0fe0e`. Methods run in this one document, scoped to the diff
against `origin/main` (`a53a104`, which is also the merge base):
- spec-review, the base checklist over the changed spec text;
- gap-analysis, planless;
- rust-review, over the removal of `pub const PGM01_POLICY_REVISION` from
  `src/lib.rs`.

TL-180 (ADR-001) had already removed tl-mltl's frontmatter edges to PGM-01.
This PR removes the remaining prose and the constant.

## Verdict

**PASS with two low findings.** It can merge as is. Both findings are optional
wording fixes.

- **No dangling live references.** No `quire-contract-ir/PGM-01` edge is left in
  any frontmatter. The remaining `PGM-01` text is of three kinds:
  - the decision records that document the retirement: ADR-001, ADR-002, SR-052,
    and FR-019's retirement note. The `spec/spec.md:102` link goes to ADR-001 by
    file name;
  - archival plans and reviews;
  - literal identifiers such as `map_pgm01_bytes` in `spec/assurance/AA-001.md:136`
    and the `pgm01-compatibility-view` file name.
- **The SR-004 deletion was ceremony only.** The file held a PGM-01-R01..R10
  mapping plus a restated review gate. It had no FR, AC or test. No file in the
  tree names SR-004 or `pgm01-reconciliation`.
- **No AC lost meaning.** The edits are in the MP-001 Collection Procedure and
  the NFR-004 Dependencies section. NFR-004-AC-1 and NFR-004-AC-2, including the
  digest-omission control, are untouched.
- **Rust review of the `pub const PGM01_POLICY_REVISION` removal.**
  - Nothing reads the constant. On origin/main it appears only at its definition,
    `src/lib.rs:98`. A search of every `.rs` and `.py` file under `~/dev` finds
    only definition sites.
  - `scripts/check_shared_pins.py:157` checks only `TL_SYNTAX_REVISION`.
  - The crate is `publish = false` at 0.3.0, and CHANGELOG entries are written
    at release time.
  - `cargo check --all-targets` passes with 0 warnings. It ran in a dedicated
    target dir, which was deleted afterwards.
- **Validation matches origin/main.** The Makefile's `quire validate` output,
  `'spec/**/*.md'` with no extra flags, is byte-identical on origin/main and the
  PR head once paths are normalised: the same 9 pre-existing failing documents
  and no new failure.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The rewrite removed the words "SHA-256 checksums" as well as "canonical PGM-01 envelope", so the collection procedure no longer says retained evidence is bound by digest. Integrity binding was never specific to PGM-01; Quoin binds each retained input by digest (AA-001:107). Fix: "... limitation, and status; Quoin binds each retained input by digest (AA-001)." | spec/assurance/MP-001.md:43-44 |
| FND-002 | low | "Human authority is retained." has no subject and no scope, and it repeats the NFR-004 text at line 54 ("An independent reviewer and the human release owner..."). Fix: delete the sentence, or say "Source-release authority stays with the human release owner." | spec/requirements/NFR-004-reproducible-corpus-retention.md:68 |

## Scope examined

| Unit | Role | Result |
| --- | --- | --- |
| spec/assurance/MP-001.md Collection Procedure | examined | FND-001 |
| spec/requirements/NFR-004 Dependencies | examined | FND-002 |
| spec/requirements/NFR-004-AC-1, NFR-004-AC-2 | context_only | unchanged |
| assurance/change-assurance.json:254 | examined | clean |
| src/lib.rs PGM01_POLICY_REVISION removal | examined | clean; no readers |
| scripts/check_shared_pins.py:146-157 | context_only | reads TL_SYNTAX_REVISION only |
| spec/reviews/SR-004-pgm01-reconciliation.md (deleted) | examined | ceremony only; no inbound links |
| spec/decisions/ADR-001, spec/reviews/SR-052, FR-019 retirement note | context_only | historical records, left in place on purpose |

## Gap analysis

Plan completion: not assessed.

- No requirement, AC, or TC was added, removed, or reworded.
- Removing the constant takes away public code that had no owning requirement
  and no test.

## Remaining pin/digest machinery (out of this PR's scope: blocked on permission)

The PR does not touch this machinery, and it is not a must-fix for this PR:

- `assurance/pins.json`
- `assurance/change-assurance.json`. This includes the open item
  `UNKNOWN-consumed-artifact-digests-now-vacuous`, which records that
  `artifact_digest_mismatches` now checks an empty population.
- `assurance/README.md`
- `scripts/check_shared_pins.py`
- `scripts/assurance_chain.py`
- `scripts/rust_test_census.py`
- `requirements-assurance.txt`
- `Makefile` targets `assurance-env`, `assurance-inputs`, `pins`,
  `assurance-chain`, `assurance` and `assurance-record`. `test` depends on
  `assurance-inputs`.
- `.github/workflows/ci.yml`
- `tests/shared_assurance.rs`
- `CLAUDE.md` contains the pin table.
- `examples/emit_shared_corpus_manifest.rs`
- Corpus checksum files:
  - `corpus/r2u2-v4.2/SHA256SUMS`
  - `corpus/past-c2po-v1/SHA256SUMS`
  - `fuzz/corpus/c2po_map/SHA256SUMS`

  These are read by `tests/infinite_corpus.rs`, `tests/past_c2po_corpus.rs` and
  `tests/c2po_map_fuzz.rs`. They are content digests, so they may be
  load-bearing.
