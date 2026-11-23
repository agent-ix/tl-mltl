---
id: ADR-046
title: "Import FRET MLTL and C2PO and WEST in tl-mltl, and leave FRETish out of scope"
type: ADR
status: proposed
owner: tl-mltl-maintainer
relationships:
  - target: ix://agent-ix/tl-mltl/FR-086
    type: realizes_requirement
  - target: ix://agent-ix/tl-mltl/FR-087
    type: realizes_requirement
  - target: ix://agent-ix/tl-mltl/FR-091
    type: realizes_requirement
  - target: ix://agent-ix/tl-mltl/FR-092
    type: realizes_requirement
  - target: ix://agent-ix/tl-parse/ADR-041
    type: relates_to
---

# ADR-046: Import FRET MLTL and C2PO and WEST in tl-mltl, and leave FRETish out of scope

## Status

Proposed (TL-427). Ready for human acceptance; not accepted.

## Context

The crates export to C2PO but cannot read it. FRET exports MLTL and C2PO files
for R2U2. WEST has a one-line formula text. tl-syntax and tl-parse already state
that FRETish is out of scope (tl-syntax spec.md; export-only FRETish mapping).
tl-mltl owns the C2PO export mapping and its refusal partition (FR-041), and
neither depends on nor runs tl-parse.

## Decision

1. **Scope.** FRET import covers exactly FRET's exported MLTL and C2PO files,
   plus the variable map and trace header that accompany an export. FRETish
   source text, FRET project files and requirement-editing metadata are out of
   scope and are refused by name, not parsed.
2. **Home.** The three readers (C2PO, FRET MLTL, WEST) and the writer for WEST
   live in a new tl-mltl module `interop`, each with its own small bounded
   lexer. They are not added to tl-parse: tl-parse is the clean-room parser of
   the project's own dialects with a provenance boundary (tl-parse NFR-002), and
   foreign grammars would blur it. Cost: three small parsers in tl-mltl; benefit:
   the C2PO reader sits next to its exporter, so export-then-import identity is
   testable in one crate.
3. **Never silent.** A construct the reader does not map is a typed loss record
   or a typed refusal naming it; it is never dropped, defaulted or approximated.
4. **Attribution.** Every imported FRET or WEST artifact carries its upstream
   source, path, revision and licence into the result, and retained fixtures keep
   their licence files. The retained WEST corpus records no upstream revision in
   tl-rewrite's manifest, so its revision is the explicit marker `UNKNOWN`
   (never invented) and no content hash is recorded (FR-089).
5. **Bare next.** A bare next in foreign text is a loss record `AmbiguousNext`;
   the importer does not choose strong or weak (tl-syntax ADR-045).

## Alternatives rejected

- Put readers in tl-parse: provenance boundary; two crates for one round trip.
- Parse FRETish: out of scope; semantic ambiguity is the reason it is export-only.
- A separate interop crate: a sixth production crate for about three parsers.

## Consequences

tl-mltl gains no dependency on tl-parse.

## Open decisions (Peter)

1. **WEST corpus: copy or move.** Option copy: tl-mltl holds a byte-identical copy
   of `formulas_d1.txt` and its licence file next to a retained-file list, and a
   test compares it with tl-rewrite's copy; two owners of the bytes exist, with no
   recorded hash. Option move: the corpus moves to one shared home (for example the
   tl-syntax corpus) and both repositories read it there; this edits tl-rewrite.
   **Recommendation: copy.**
2. **Importers in tl-mltl.** This ADR places them in tl-mltl. **Recommendation:
   keep** (no tl-parse dependency; the C2PO reader sits next to its exporter).
